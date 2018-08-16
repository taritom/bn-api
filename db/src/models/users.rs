use chrono::NaiveDateTime;
use db::Connectable;
use diesel;
use diesel::prelude::*;
use models::ExternalLogin;
use models::Roles;
use schema::users;
use utils::errors::{DatabaseError, ErrorCode};
use utils::passwords::PasswordHash;
use uuid::Uuid;

#[derive(Insertable, PartialEq, Debug)]
#[table_name = "users"]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub hashed_pw: String,
    role: Vec<String>,
}

#[derive(Queryable, Identifiable, PartialEq, Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub hashed_pw: String,
    pub password_modified_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub last_used: Option<NaiveDateTime>,
    pub active: bool,
    pub role: Vec<String>,
    pub password_reset_token: Option<Uuid>,
    pub password_reset_requested_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DisplayUser {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub created_at: NaiveDateTime,
}

impl NewUser {
    pub fn commit(&self, conn: &Connectable) -> Result<User, DatabaseError> {
        let res = diesel::insert_into(users::table)
            .values(self)
            .get_result(conn.get_connection());
        DatabaseError::wrap(ErrorCode::InsertError, "Could not create new user", res)
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
            role: vec![Roles::Guest.to_string()],
        }
    }

    pub fn find(id: &Uuid, conn: &Connectable) -> Result<User, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading user",
            users::table.find(id).first::<User>(conn.get_connection()),
        )
    }

    pub fn find_by_email(email: &str, conn: &Connectable) -> Result<Option<User>, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading user",
            users::table
                .filter(users::email.eq(email))
                .first::<User>(conn.get_connection())
                .optional(),
        )
    }

    pub fn check_password(&self, password: &str) -> bool {
        let hash = match PasswordHash::from_str(&self.hashed_pw) {
            Ok(h) => h,
            Err(_) => return false,
        };
        hash.verify(password)
    }

    pub fn add_role(&self, r: Roles, conn: &Connectable) -> Result<User, DatabaseError> {
        let mut new_roles = self.role.clone();
        new_roles.push(r.to_string());
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not add role to user",
            diesel::update(self)
                .set(users::role.eq(&new_roles))
                .get_result(conn.get_connection()),
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
        conn: &Connectable,
    ) -> Result<ExternalLogin, DatabaseError> {
        ExternalLogin::create(external_user_id, site, self.id, access_token).commit(&*conn)
    }

    pub fn create_from_external_login(
        external_user_id: String,
        site: String,
        access_token: String,
        conn: &Connectable,
    ) -> Result<User, DatabaseError> {
        let hash = PasswordHash::generate("random", None);
        let new_user = NewUser {
            first_name: String::from("Unknown"),
            last_name: String::from("Unknown"),
            email: None,
            phone: None,
            hashed_pw: hash.to_string(),
            role: vec![Roles::Guest.to_string()],
        };
        new_user.commit(&*conn).and_then(|user| {
            user.add_external_login(external_user_id, site, access_token, conn)?;
            Ok(user)
        })
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
            created_at: user.created_at,
        }
    }
}
