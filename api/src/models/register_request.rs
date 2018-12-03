use bigneon_db::models::{deserialize_unless_blank, NewUser, User};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub first_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub last_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub phone: Option<String>,
    pub password: String,
}

impl From<RegisterRequest> for NewUser {
    fn from(attributes: RegisterRequest) -> Self {
        User::create(
            attributes.first_name,
            attributes.last_name,
            Some(attributes.email),
            attributes.phone,
            &attributes.password,
        )
    }
}

impl RegisterRequest {
    pub fn new(
        first_name: &str,
        last_name: &str,
        email: &str,
        phone: &str,
        password: &str,
    ) -> RegisterRequest {
        RegisterRequest {
            first_name: Some(first_name.to_string()),
            last_name: Some(last_name.to_string()),
            email: email.to_string(),
            phone: Some(phone.to_string()),
            password: password.to_string(),
        }
    }
}
