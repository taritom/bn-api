use bigneon_db::models::*;
use diesel::pg::PgConnection;
use errors::*;
use itertools::Itertools;
use std::collections::HashMap;

pub fn tickets_received(
    to_user: &User,
    from_user: &User,
    conn: &PgConnection,
) -> Result<(), BigNeonError> {
    let tokens = to_user
        .push_notification_tokens(conn)?
        .into_iter()
        .sorted_by_key(|pt| pt.token_source.clone())
        .into_iter()
        .group_by(|pt| pt.token_source.clone());

    for (source, token_list) in tokens.into_iter() {
        let addresses = token_list.map(|pt| pt.token).collect_vec();

        if addresses.len() > 0 {
            let addresses = CommAddress::from_vec(addresses);
            let body = format!("{} has sent you some tickets.", from_user.full_name(),);

            let mut extra_data = HashMap::new();
            extra_data.insert("source".to_string(), source.to_string());

            Communication::new(
                CommunicationType::Push,
                body,
                None,
                None,
                addresses,
                None,
                None,
                Some(vec!["transfers"]),
                Some(extra_data),
            )
            .queue(conn)?;
        }
    }

    Ok(())
}
