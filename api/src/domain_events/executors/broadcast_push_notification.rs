use bigneon_db::prelude::*;
use config::Config;
use db::Connection;
use diesel::PgConnection;
use domain_events::executor_future::ExecutorFuture;
use domain_events::routing::DomainActionExecutor;
use errors::*;
use futures::future;
use itertools::Itertools;
use log::Level::Error;
use validator::HasLen;

pub struct BroadcastPushNotificationExecutor {
    template_id: Option<String>,
}

impl DomainActionExecutor for BroadcastPushNotificationExecutor {
    fn execute(&self, action: DomainAction, conn: Connection) -> ExecutorFuture {
        match self.perform_job(&action, &conn) {
            Ok(_) => ExecutorFuture::new(action, conn, Box::new(future::ok(()))),
            Err(e) => {
                jlog!(Error, "Send tickets mail action failed", {"action_id": action.id, "main_table_id":action.main_table_id,  "error": e.to_string()});
                ExecutorFuture::new(action, conn, Box::new(future::err(e)))
            }
        }
    }
}

impl BroadcastPushNotificationExecutor {
    pub fn new(config: &Config) -> BroadcastPushNotificationExecutor {
        BroadcastPushNotificationExecutor {
            template_id: Some(config.email_templates.custom_broadcast.to_string()),
        }
    }

    fn perform_job(&self, action: &DomainAction, conn: &Connection) -> Result<(), BigNeonError> {
        let conn = conn.get();
        let broadcast_id = action.main_table_id.ok_or(ApplicationError::new(
            "No broadcast id attached to domain action".to_string(),
        ))?;
        let broadcast = Broadcast::find(broadcast_id, conn)?;
        if broadcast.status == BroadcastStatus::Cancelled {
            return Ok(());
        }

        let broadcast = broadcast.set_in_progress(conn)?;
        let message = broadcast.message.clone();
        let message = message.unwrap_or("".to_string());
        let message = match broadcast.notification_type {
            BroadcastType::LastCall => {
                "🗣LAST CALL! 🍻The bar is closing soon, grab something now before it's too late!"
            }
            BroadcastType::Custom => message.as_str(),
        };

        let audience: Vec<User> = match broadcast.audience {
            BroadcastAudience::PeopleAtTheEvent => Event::checked_in_users(broadcast.event_id, conn)?
                .into_iter()
                .map(|u| (u, Vec::new(), None))
                .collect_vec(),
            BroadcastAudience::TicketHolders => Event::find_all_ticket_holders(broadcast.event_id, conn)?,
        }
        .into_iter()
        .map(|aud| aud.0)
        .collect_vec();

        Broadcast::set_sent_count(broadcast_id, audience.length() as i64, conn)?;

        for user in audience {
            match broadcast.channel {
                BroadcastChannel::PushNotification => {
                    queue_push_notification(&broadcast, message.to_string(), &user, conn)?;
                }
                BroadcastChannel::Email => queue_email_notification(
                    &broadcast,
                    conn,
                    self.template_id.clone(),
                    message.to_string(),
                    &user,
                    broadcast.preview_email.clone(),
                )?,
            }
            // if preview email, only send 1 email
            if broadcast.preview_email != None {
                break;
            }
        }

        Ok(())
    }
}

fn queue_push_notification(
    broadcast: &Broadcast,
    message: String,
    user: &User,
    conn: &PgConnection,
) -> Result<(), BigNeonError> {
    let tokens = user
        .push_notification_tokens(conn)?
        .into_iter()
        .map(|pt| pt.token)
        .collect_vec();

    if tokens.len() > 0 {
        DomainAction::create(
            None,
            DomainActionTypes::Communication,
            Some(CommunicationChannelType::Push),
            serde_json::to_value(Communication::new(
                CommunicationType::Push,
                message,
                None,
                None,
                CommAddress::from_vec(tokens),
                None,
                None,
                Some(vec!["broadcast"]),
                Some(
                    [("broadcast_id".to_string(), broadcast.id.to_string())]
                        .iter()
                        .cloned()
                        .collect(),
                ),
            ))?,
            Some(Tables::Events),
            Some(broadcast.event_id),
        )
        .commit(conn)?;
    }

    Ok(())
}

fn queue_email_notification(
    broadcast: &Broadcast,
    conn: &PgConnection,
    template_id: Option<String>,
    message: String,
    user: &User,
    preview_email: Option<String>,
) -> Result<(), BigNeonError> {
    if user.email.is_none() {
        return Ok(());
    }

    let email = match preview_email {
        None => CommAddress::from(user.email.clone().unwrap()),
        Some(e) => CommAddress::from(e),
    };

    DomainAction::create(
        None,
        DomainActionTypes::Communication,
        Some(CommunicationChannelType::Email),
        serde_json::to_value(Communication::new(
            CommunicationType::EmailTemplate,
            broadcast.subject.as_ref().unwrap_or(&broadcast.name).to_string(),
            Some(message),
            None,
            email,
            template_id,
            None,
            Some(vec!["broadcast"]),
            Some(
                [
                    ("broadcast_id".to_string(), broadcast.id.to_string()),
                    ("event_id".to_string(), broadcast.event_id.to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
            ),
        ))?,
        Some(Tables::Events),
        Some(broadcast.event_id),
    )
    .commit(conn)?;
    Ok(())
}
