use bigneon_api::models::UserContactAttributes;
use validator::Validate;

#[test]
fn user_contact_attributes_validate() {
    let mut user_parameters: UserContactAttributes = Default::default();
    user_parameters.phone = Some("abc".into());
    user_parameters.email = Some("abc".into());

    let result = user_parameters.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err().inner();

    assert!(errors.contains_key("email"));
    assert_eq!(errors["email"].len(), 1);
    assert_eq!(errors["email"][0].code, "email");
}
