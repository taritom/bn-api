#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub phone: String,
    pub password: String,
}

impl RegisterRequest {
    pub fn new(name: &str, email: &str, phone: &str, password: &str) -> RegisterRequest {
        RegisterRequest {
            name: name.to_string(),
            email: email.to_string(),
            phone: phone.to_string(),
            password: password.to_string(),
        }
    }
}
