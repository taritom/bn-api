#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub password: String,
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
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            email: email.to_string(),
            phone: phone.to_string(),
            password: password.to_string(),
        }
    }
}
