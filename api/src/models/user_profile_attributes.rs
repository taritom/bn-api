use bigneon_db::models::UserEditableAttributes;
use validator::Validate;

#[derive(Default, Deserialize, Validate)]
pub struct UserProfileAttributes {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[validate(email(message = "Email is invalid"))]
    pub email: Option<String>,
    pub phone: Option<String>,
    #[validate(url(message = "Profile pic URL is invalid"))]
    pub profile_pic_url: Option<String>,
    #[validate(url(message = "Thumb profile pic URL is invalid"))]
    pub thumb_profile_pic_url: Option<String>,
    #[validate(url(message = "Cover photo URL is invalid"))]
    pub cover_photo_url: Option<String>,
}

impl From<UserProfileAttributes> for UserEditableAttributes {
    fn from(attributes: UserProfileAttributes) -> Self {
        UserEditableAttributes {
            first_name: attributes.first_name,
            last_name: attributes.last_name,
            email: attributes.email,
            phone: attributes.phone,
            profile_pic_url: attributes.profile_pic_url,
            thumb_profile_pic_url: attributes.thumb_profile_pic_url,
            cover_photo_url: attributes.cover_photo_url,
            ..Default::default()
        }
    }
}
