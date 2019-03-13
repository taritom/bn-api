use bigneon_db::models::User;
use diesel::pg::PgConnection;
use errors::*;
use itertools::Itertools;
use utils::communication::CommAddress;
use utils::communication::Communication;
use utils::communication::CommunicationType;

pub fn tickets_received(
    to_user: &User,
    from_user: &User,
    conn: &PgConnection,
) -> Result<(), BigNeonError> {
    let tokens = to_user
        .push_notification_tokens(conn)?
        .into_iter()
        .map(|pt| pt.token)
        .collect_vec();

    if tokens.len() > 0 {
        let body = format!("{} has sent you some tickets.", from_user.full_name(),);

        Communication::new(
            CommunicationType::Push,
            body,
            None,
            None,
            CommAddress::from_vec(tokens),
            None,
            None,
        )
        .queue(conn)?;
    }
    Ok(())
}
