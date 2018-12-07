use bigneon_db::dev::TestProject;
use bigneon_db::models::*;

#[test]
fn create_and_find_by_user_id() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    //Create two push notification tokens for user
    let pnt1_request = NewPushNotificationToken {
        user_id: user.id,
        token_source: "example_token1_source".to_string(),
        token: "example_token1".to_string(),
    };
    let pnt2_request = NewPushNotificationToken {
        user_id: user.id,
        token_source: "example_token2_source".to_string(),
        token: "example_token2".to_string(),
    };
    pnt1_request.commit(connection).unwrap();
    pnt2_request.commit(connection).unwrap();
    //Check stored push notification tokens
    let push_notification_tokens =
        PushNotificationToken::find_by_user_id(user.id, connection).unwrap();
    assert_eq!(push_notification_tokens.len(), 2);
    assert_eq!(push_notification_tokens[0].user_id, pnt1_request.user_id);
    assert_eq!(
        push_notification_tokens[0].token_source,
        pnt1_request.token_source
    );
    assert_eq!(push_notification_tokens[0].token, pnt1_request.token);
    assert_eq!(push_notification_tokens[1].user_id, pnt2_request.user_id);
    assert_eq!(
        push_notification_tokens[1].token_source,
        pnt2_request.token_source
    );
    assert_eq!(push_notification_tokens[1].token, pnt2_request.token);
}

#[test]
fn remove_push_notification_token() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    //Create two push notification tokens for user
    let pnt1_request = NewPushNotificationToken {
        user_id: user.id,
        token_source: "example_token1_source".to_string(),
        token: "example_token1".to_string(),
    };
    let pnt2_request = NewPushNotificationToken {
        user_id: user.id,
        token_source: "example_token2_source".to_string(),
        token: "example_token2".to_string(),
    };
    pnt1_request.commit(connection).unwrap();
    pnt2_request.commit(connection).unwrap();
    let stored_push_notification_tokens =
        PushNotificationToken::find_by_user_id(user.id, connection).unwrap();
    //Remove first created token
    PushNotificationToken::remove(user.id, stored_push_notification_tokens[0].id, connection)
        .unwrap();
    //Check stored push notification tokens
    let updated_push_notification_tokens =
        PushNotificationToken::find_by_user_id(user.id, connection).unwrap();
    assert_eq!(updated_push_notification_tokens.len(), 1);
    assert_eq!(
        updated_push_notification_tokens[0].user_id,
        stored_push_notification_tokens[1].user_id
    );
    assert_eq!(
        updated_push_notification_tokens[0].token_source,
        stored_push_notification_tokens[1].token_source
    );
    assert_eq!(
        updated_push_notification_tokens[0].token,
        stored_push_notification_tokens[1].token
    );
}
