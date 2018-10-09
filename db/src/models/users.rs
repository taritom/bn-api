use chrono::NaiveDateTime;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::*;
use schema::{organization_users, organizations, users};
use std::collections::HashMap;
use utils::errors::{ConvertToDatabaseError, DatabaseError, ErrorCode};
use utils::passwords::PasswordHash;
use uuid::Uuid;
use validator::Validate;

#[derive(Insertable, PartialEq, Debug, Validate)]
#[table_name = "users"]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    #[validate(email)]
    pub email: Option<String>,
    pub phone: Option<String>,
    pub hashed_pw: String,
    role: Vec<String>,
}

#[derive(Queryable, Identifiable, PartialEq, Debug, Clone, QueryableByName)]
#[table_name = "users"]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub profile_pic_url: Option<String>,
    pub thumb_profile_pic_url: Option<String>,
    pub cover_photo_url: Option<String>,
    pub hashed_pw: String,
    pub password_modified_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub last_used: Option<NaiveDateTime>,
    pub active: bool,
    pub role: Vec<String>,
    pub password_reset_token: Option<Uuid>,
    pub password_reset_requested_at: Option<NaiveDateTime>,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct DisplayUser {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub profile_pic_url: Option<String>,
    pub thumb_profile_pic_url: Option<String>,
    pub cover_photo_url: Option<String>,
}

#[derive(AsChangeset, Default, Deserialize, Validate)]
#[table_name = "users"]
pub struct UserEditableAttributes {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub phone: Option<String>,
    pub active: Option<bool>,
    pub role: Option<Vec<String>>,
    #[validate(url)]
    pub profile_pic_url: Option<String>,
    #[validate(url)]
    pub thumb_profile_pic_url: Option<String>,
    #[validate(url)]
    pub cover_photo_url: Option<String>,
}

impl NewUser {
    pub fn commit(&self, conn: &PgConnection) -> Result<User, DatabaseError> {
        let user: User = diesel::insert_into(users::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create new user")?;

        Wallet::create_for_user(user.id, "Default".to_string()).commit(conn)?;

        Ok(user)
    }
}

impl User {
    pub fn create(
        first_name: &str,
        last_name: &str,
        email: &str,
        phone: &str,
        password: &str,
    ) -> NewUser {
        let hash = PasswordHash::generate(password, None);
        NewUser {
            first_name: String::from(first_name),
            last_name: String::from(last_name),
            email: Some(String::from(email)),
            phone: Some(String::from(phone)),
            hashed_pw: hash.to_string(),
            role: vec![Roles::User.to_string()],
        }
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<User, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading user",
            users::table.find(id).first::<User>(conn),
        )
    }

    pub fn find_by_email(email: &str, conn: &PgConnection) -> Result<User, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading user",
            users::table
                .filter(users::email.eq(email))
                .first::<User>(conn),
        )
    }

    pub fn update(
        &self,
        attributes: &UserEditableAttributes,
        conn: &PgConnection,
    ) -> Result<User, DatabaseError> {
        let query = diesel::update(self).set((attributes, users::updated_at.eq(dsl::now)));

        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Error updating user",
            query.get_result(conn),
        )
    }

    pub fn check_password(&self, password: &str) -> bool {
        let hash = match PasswordHash::from_str(&self.hashed_pw) {
            Ok(h) => h,
            Err(_) => return false,
        };
        hash.verify(password)
    }

    pub fn add_role(&self, r: Roles, conn: &PgConnection) -> Result<User, DatabaseError> {
        let mut new_roles = self.role.clone();
        if !new_roles.contains(&r.to_string()) {
            new_roles.push(r.to_string());
        }

        self.update_role(new_roles, conn)
    }

    pub fn remove_role(&self, r: Roles, conn: &PgConnection) -> Result<User, DatabaseError> {
        let mut current_roles = self.role.clone();

        current_roles.retain(|x| x.as_str() != &r.to_string());

        self.update_role(current_roles, conn)
    }

    pub fn get_global_scopes(&self) -> Vec<String> {
        scopes::get_scopes(self.role.clone())
    }

    pub fn get_roles_by_organization(
        &self,
        conn: &PgConnection,
    ) -> Result<HashMap<Uuid, Vec<String>>, DatabaseError> {
        let mut roles_by_organization = HashMap::new();
        for organization in self.organizations(conn)? {
            roles_by_organization.insert(
                organization.id.clone(),
                organization.get_roles_for_user(self, conn)?,
            );
        }
        Ok(roles_by_organization)
    }

    pub fn get_scopes_by_organization(
        &self,
        conn: &PgConnection,
    ) -> Result<HashMap<Uuid, Vec<String>>, DatabaseError> {
        let mut scopes_by_organization = HashMap::new();
        for organization in self.organizations(conn)? {
            scopes_by_organization.insert(
                organization.id.clone(),
                organization.get_scopes_for_user(self, conn)?,
            );
        }
        Ok(scopes_by_organization)
    }

    pub fn organizations(&self, conn: &PgConnection) -> Result<Vec<Organization>, DatabaseError> {
        organizations::table
            .left_join(organization_users::table)
            .filter(
                organization_users::user_id
                    .eq(self.id)
                    .or(organization_users::id
                        .is_null()
                        .and(organizations::owner_user_id.eq(self.id))),
            ).select(organizations::all_columns)
            .order_by(organizations::name.asc())
            .load::<Organization>(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not retrieve organization users",
            )
    }

    fn update_role(
        &self,
        new_roles: Vec<String>,
        conn: &PgConnection,
    ) -> Result<User, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update role for user",
            diesel::update(self)
                .set((users::role.eq(new_roles), users::updated_at.eq(dsl::now)))
                .get_result(conn),
        )
    }

    pub fn for_display(self) -> DisplayUser {
        self.into()
    }

    pub fn full_name(&self) -> String {
        vec![self.first_name.to_string(), self.last_name.to_string()].join(" ")
    }

    pub fn add_external_login(
        &self,
        external_user_id: String,
        site: String,
        access_token: String,
        conn: &PgConnection,
    ) -> Result<ExternalLogin, DatabaseError> {
        ExternalLogin::create(external_user_id, site, self.id, access_token).commit(&*conn)
    }

    pub fn create_from_external_login(
        external_user_id: String,
        first_name: String,
        last_name: String,
        email: String,
        site: String,
        access_token: String,
        conn: &PgConnection,
    ) -> Result<User, DatabaseError> {
        let hash = PasswordHash::generate("random", None);
        let new_user = NewUser {
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            email: Some(email.to_string()),
            phone: None,
            hashed_pw: hash.to_string(),
            role: vec![Roles::User.to_string()],
        };
        new_user.commit(&*conn).and_then(|user| {
            user.add_external_login(external_user_id, site, access_token, conn)?;
            Ok(user)
        })
    }

    pub fn has_role(&self, role: Roles) -> bool {
        self.role.contains(&role.to_string())
    }

    pub fn can_read_user(&self, user: &User, conn: &PgConnection) -> Result<bool, DatabaseError> {
        if self.has_role(Roles::Admin) || self == user {
            return Ok(true);
        }
        // TODO: Once OrgAdmin is moved to the organization_users table this logic will need to be adjusted

        let organizations = organizations::table
            .filter(organizations::owner_user_id.eq(self.id))
            .load::<Organization>(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not retrieve organizations owned by user",
            )?;
        let organizations2 = OrganizationUser::belonging_to(user)
            .inner_join(organizations::table)
            .select(organizations::all_columns)
            .load::<Organization>(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not retrieve organizations for user",
            )?;

        let mut can_read = false;
        for organization in organizations {
            can_read = organizations2.contains(&organization);
            if can_read {
                break;
            }
        }
        Ok(can_read)
    }

    pub fn wallets(&self, conn: &PgConnection) -> Result<Vec<Wallet>, DatabaseError> {
        Wallet::find_for_user(self.id, conn)
    }
}

impl From<User> for DisplayUser {
    fn from(user: User) -> Self {
        DisplayUser {
            id: user.id,
            first_name: user.first_name,
            last_name: user.last_name,
            email: user.email,
            phone: user.phone,
            profile_pic_url: user.profile_pic_url,
            thumb_profile_pic_url: user.thumb_profile_pic_url,
            cover_photo_url: user.cover_photo_url,
        }
    }
}

impl ForDisplay<DisplayUser> for User {
    fn for_display(self) -> Result<DisplayUser, DatabaseError> {
        Ok(self.into())
    }
}
