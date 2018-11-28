use chrono::prelude::Utc;
use chrono::NaiveDateTime;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::*;
use schema::{events, organization_users, organizations, users};
use std::collections::HashMap;
use time::Duration;
use utils::errors::{ConvertToDatabaseError, DatabaseError, ErrorCode};
use utils::passwords::PasswordHash;
use uuid::Uuid;
use validator::Validate;

#[derive(Insertable, PartialEq, Debug, Validate)]
#[table_name = "users"]
pub struct NewUser {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[validate(email(message = "Email is invalid"))]
    pub email: Option<String>,
    pub phone: Option<String>,
    pub hashed_pw: String,
    role: Vec<String>,
}

#[derive(Queryable, Identifiable, PartialEq, Debug, Clone, QueryableByName)]
#[table_name = "users"]
pub struct User {
    pub id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
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
    pub last_cart_id: Option<Uuid>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct DisplayUser {
    pub id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub profile_pic_url: Option<String>,
    pub thumb_profile_pic_url: Option<String>,
    pub cover_photo_url: Option<String>,
    pub is_org_owner: bool,
}

#[derive(AsChangeset, Default, Deserialize, Validate)]
#[table_name = "users"]
pub struct UserEditableAttributes {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[validate(email(message = "Email is invalid"))]
    pub email: Option<String>,
    pub phone: Option<String>,
    pub active: Option<bool>,
    pub role: Option<Vec<String>>,
    #[validate(url(message = "Profile pic URL is invalid"))]
    pub profile_pic_url: Option<String>,
    #[validate(url(message = "Thumb profile pic URL is invalid"))]
    pub thumb_profile_pic_url: Option<String>,
    #[validate(url(message = "Cover photo URL is invalid"))]
    pub cover_photo_url: Option<String>,
}

impl NewUser {
    pub fn commit(&self, conn: &PgConnection) -> Result<User, DatabaseError> {
        self.validate()?;
        let user: User = diesel::insert_into(users::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create new user")?;

        Wallet::create_for_user(user.id, "Default".to_string(), true, conn)?;

        Ok(user)
    }
}

impl User {
    pub fn create(
        first_name: &Option<String>,
        last_name: &Option<String>,
        email: &Option<String>,
        phone: &Option<String>,
        password: &str,
    ) -> NewUser {
        let hash = PasswordHash::generate(password, None);
        NewUser {
            first_name: first_name.clone(),
            last_name: last_name.clone(),
            email: email.clone(),
            phone: phone.clone(),
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
        attributes.validate()?;
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

    pub fn has_role(&self, role: Roles) -> bool {
        self.role.contains(&role.to_string())
    }

    pub fn is_admin(&self) -> bool {
        self.has_role(Roles::Admin)
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

    pub fn payment_methods(
        &self,
        conn: &PgConnection,
    ) -> Result<Vec<PaymentMethod>, DatabaseError> {
        PaymentMethod::find_for_user(self.id, None, conn)
    }

    pub fn default_payment_method(
        &self,
        conn: &PgConnection,
    ) -> Result<PaymentMethod, DatabaseError> {
        PaymentMethod::find_default_for_user(self.id, conn)
    }

    pub fn payment_method(
        &self,
        name: String,
        conn: &PgConnection,
    ) -> Result<PaymentMethod, DatabaseError> {
        let mut payment_methods = PaymentMethod::find_for_user(self.id, Some(name), conn)?;
        if payment_methods.is_empty() {
            Err(DatabaseError::new(
                ErrorCode::NoResults,
                Some("No payment method found for user".to_string()),
            ))
        } else {
            Ok(payment_methods.remove(0))
        }
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

    pub fn find_events_with_access_to_scan(
        &self,
        conn: &PgConnection,
    ) -> Result<Vec<Event>, DatabaseError> {
        let event_start = NaiveDateTime::from(Utc::now().naive_utc() - Duration::days(1));

        if self.is_admin() {
            DatabaseError::wrap(
                ErrorCode::QueryError,
                "Error loading events to scan",
                events::table
                    .filter(events::status.eq(EventStatus::Published.to_string()))
                    .filter(events::event_start.ge(event_start))
                    .order_by(events::event_start.asc())
                    .load(conn),
            )
        } else {
            DatabaseError::wrap(
                ErrorCode::QueryError,
                "Error loading events to scan",
                events::table
                    .inner_join(
                        organization_users::table.on(organization_users::organization_id
                            .eq(events::organization_id)
                            .and(organization_users::user_id.eq(self.id))),
                    ).filter(events::status.eq(EventStatus::Published.to_string()))
                    .filter(events::event_start.ge(event_start))
                    .order_by(events::event_start.asc())
                    .select(events::all_columns)
                    .load(conn),
            )
        }
    }

    pub fn full_name(&self) -> String {
        vec![
            self.first_name.clone().unwrap_or("".to_string()),
            self.last_name.clone().unwrap_or("".to_string()),
        ].join(" ")
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
            first_name: Some(first_name.to_string()),
            last_name: Some(last_name.to_string()),
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

    pub fn update_last_cart(
        &self,
        new_cart_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        // diesel does not have any easy way of handling "last_cart_id is null OR last_cart_id = 'x'"
        let query = if self.last_cart_id.is_none() {
            diesel::update(
                users::table
                    .filter(users::id.eq(self.id))
                    .filter(users::updated_at.eq(self.updated_at))
                    .filter(users::last_cart_id.is_null()),
            ).into_boxed()
        } else {
            diesel::update(
                users::table
                    .filter(users::id.eq(self.id))
                    .filter(users::updated_at.eq(self.updated_at))
                    .filter(users::last_cart_id.eq(self.last_cart_id)),
            ).into_boxed()
        };
        let rows_affected = query
            .set((
                users::last_cart_id.eq(new_cart_id),
                users::updated_at.eq(dsl::now),
            )).execute(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update last cart on user")?;

        match rows_affected {
        1 => {
           Ok(())
        },

        _ => DatabaseError::concurrency_error("Could not update last cart on user because the row has been changed by another source")
    }
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
            is_org_owner: false,
        }
    }
}

impl ForDisplay<DisplayUser> for User {
    fn for_display(self) -> Result<DisplayUser, DatabaseError> {
        Ok(self.into())
    }
}
