use bigneon_db::dev::TestProject;
use bigneon_db::models::{EventInterest, User};
use rand::prelude::*;
use uuid::Uuid;

#[test]
fn create() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let event = project.create_event().finish();

    let event_interest = EventInterest::create(event.id, user.id)
        .commit(project.get_connection())
        .unwrap();

    assert_eq!(event_interest.user_id, user.id);
    assert_eq!(event_interest.event_id, event.id);
}

#[test]
fn total_interest() {
    let project = TestProject::new();
    let user1 = project.create_user().finish();
    let user2 = project.create_user().finish();
    let event = project.create_event().finish();

    let _event_interest1 = EventInterest::create(event.id, user1.id)
        .commit(project.get_connection())
        .unwrap();

    let _event_interest2 = EventInterest::create(event.id, user2.id)
        .commit(project.get_connection())
        .unwrap();

    assert_eq!(
        EventInterest::total_interest(event.id, project.get_connection()).unwrap(),
        2
    );
}

#[test]
fn user_interest() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let event = project.create_event().finish();

    let _event_interest1 = EventInterest::create(event.id, user.id)
        .commit(project.get_connection())
        .unwrap();

    assert_eq!(
        EventInterest::user_interest(event.id, user.id, project.get_connection()).unwrap(),
        true
    );
}

#[test]
fn list_interested_users() {
    let project = TestProject::new();
    let primary_event = project.create_event().finish();
    let secondary_event = project.create_event().finish();
    let primary_user = project.create_user().finish();
    //Create set of secondary users with interest in the primary and secondary event
    let n_secondary_users = 15;
    let mut rng = thread_rng();
    let p_event_interest_flag_list: Vec<u8> = (0..n_secondary_users)
        .map(|_| rng.gen_range(0, 2))
        .collect(); //[0,1]
    let s_event_interest_flag_list: Vec<u8> = (0..n_secondary_users)
        .map(|_| rng.gen_range(0, 2))
        .collect(); //[0,1]
    let mut secondary_user_list: Vec<User> = Vec::new();
    for _u_id in 0..n_secondary_users {
        secondary_user_list.push(project.create_user().finish());
    }
    //Let primary user show interest in primary event
    EventInterest::create(primary_event.id, primary_user.id)
        .commit(project.get_connection())
        .unwrap();
    //Let secondary users show interest in primary or/and secondary event
    let mut desired_user_id_completelist: Vec<Uuid> = Vec::new();
    for u_id in 0..n_secondary_users {
        if p_event_interest_flag_list[u_id] == 1 {
            //Set interest for primary event
            desired_user_id_completelist.push(secondary_user_list[u_id].id);
            let _secondary_event_interest =
                EventInterest::create(primary_event.id, secondary_user_list[u_id].id)
                    .commit(project.get_connection())
                    .unwrap();
        }
        if s_event_interest_flag_list[u_id] == 1 {
            //Set interest for secondary event
            let _secondary_event_interest =
                EventInterest::create(secondary_event.id, secondary_user_list[u_id].id)
                    .commit(project.get_connection())
                    .unwrap();
        }
    }
    desired_user_id_completelist.sort(); //Sort results for testing purposes
    if desired_user_id_completelist.len() > 0 {
        //Test1 - Normal Query of list of interested users for event, excluding primary user
        let request_from_page: usize = 0;
        let request_limit: usize = 100;
        let result = EventInterest::list_interested_users(
            primary_event.id,
            primary_user.id,
            request_from_page as u32,
            request_limit as u32,
            project.get_connection(),
        ).unwrap();

        let n_sublist_entries = desired_user_id_completelist.len();
        for u_id in 0..n_sublist_entries {
            assert_eq!(
                desired_user_id_completelist[u_id],
                result.data[u_id].user_id
            );
        }
    }
}
