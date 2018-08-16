use bigneon_db::models::Roles;
use bigneon_db::models::User;
use bigneon_db::utils::errors;
use bigneon_db::utils::errors::ErrorCode;
use support::project::TestProject;
use uuid::Uuid;

#[test]
fn commit() {
    let project = TestProject::new();
    let first_name = "Jeff";
    let last_name = "Wilco";
    let email = "jeff@tari.com";
    let phone_number = "555-555-5555";
    let password = "examplePassword";
    let user = User::create(first_name, last_name, email, phone_number, password)
        .commit(&project)
        .unwrap();

    assert_eq!(user.first_name, first_name);
    assert_eq!(user.last_name, last_name);
    assert_eq!(user.email, Some(email.to_string()));
    assert_eq!(user.phone, Some(phone_number.to_string()));
    assert_ne!(user.hashed_pw, password);
    assert_eq!(user.hashed_pw.is_empty(), false);
    assert_eq!(user.id.to_string().is_empty(), false);
}

#[test]
fn commit_duplicate_email() {
    let project = TestProject::new();
    let user1 = project.create_user().finish();
    let first_name = "Jeff";
    let last_name = "Wilco";
    let email = &user1.email.unwrap();
    let phone_number = "555-555-5555";
    let password = "examplePassword";
    let result =
        User::create(first_name, last_name, email, phone_number, password).commit(&project);

    assert_eq!(result.is_err(), true);
    assert_eq!(
        result.err().unwrap().code,
        errors::get_error_message(ErrorCode::DuplicateKeyError).0
    );
}

#[test]
fn find() {
    let project = TestProject::new();
    let user = project.create_user().finish();

    let found_user = User::find(&user.id, &project).expect("User was not found");
    assert_eq!(found_user.id, user.id);
    assert_eq!(found_user.email, user.email);

    assert!(
        match User::find(&Uuid::new_v4(), &project) {
            Ok(_user) => false,
            Err(_e) => true,
        },
        "User incorrectly returned when id invalid"
    );
}

#[test]
fn full_name() {
    let project = TestProject::new();

    let first_name = "Bob".to_string();
    let last_name = "Jones".to_string();

    let user = project
        .create_user()
        .with_first_name(first_name.clone())
        .with_last_name(last_name.clone())
        .finish();
    assert_eq!(user.full_name(), format!("{} {}", first_name, last_name));
}

#[test]
fn find_by_email() {
    let project = TestProject::new();
    let user = project.create_user().finish();

    let found_user =
        User::find_by_email(&user.email.clone().unwrap(), &project).expect("User was not found");
    assert_eq!(found_user.unwrap(), user);

    assert!(
        User::find_by_email("not@real.com", &project)
            .unwrap()
            .is_none(),
        "User incorrectly returned when email invalid"
    );
}

#[test]
fn for_display() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let user_id = user.id.clone();
    let display_user = user.for_display();

    assert_eq!(display_user.id, user_id);
}

#[test]
fn add_role() {
    let project = TestProject::new();
    let user = project.create_user().finish();

    user.add_role(Roles::Admin, &project).unwrap();

    let user2 = User::find(&user.id, &project).unwrap();
    assert_eq!(user2.role, vec!["Guest", "Admin"]);
}
