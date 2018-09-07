use bigneon_db::models::UserEditableAttributes;
use validator::Validate;

#[derive(Default, Deserialize, Validate)]
pub struct UserContactAttributes {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub phone: Option<String>,
}

impl From<UserContactAttributes> for UserEditableAttributes {
    fn from(attributes: UserContactAttributes) -> Self {
        UserEditableAttributes {
            first_name: attributes.first_name,
            last_name: attributes.last_name,
            email: attributes.email,
            phone: attributes.phone,
            ..Default::default()
        }
    }
}
