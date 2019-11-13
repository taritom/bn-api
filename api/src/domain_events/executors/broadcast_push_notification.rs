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
use chrono::Utc;

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
            template_id: Some(config.email_templates.custom_broadcast.template_id.clone()),
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
        let (audience_type, message) = match broadcast.notification_type {
            BroadcastType::LastCall => (
                BroadcastAudience::PeopleAtTheEvent,
                "🗣LAST CALL! 🍻The bar is closing soon, grab something now before it's too late!",
            ),
            BroadcastType::Custom => (BroadcastAudience::PeopleAtTheEvent, message.as_str()),
        };

        let mut audience: Vec<User> = match audience_type {
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

        // if broadcast is a preview, use the email that where the preview should be sent to
        if let Some(preview_address) = broadcast.preview.clone() {
            let user = User {
                id: Default::default(),
                first_name: None,
                last_name: None,
                email: Some(preview_address),
                phone: None,
                profile_pic_url: None,
                thumb_profile_pic_url: None,
                cover_photo_url: None,
                hashed_pw: "".to_string(),
                password_modified_at: Utc::now().naive_utc(),
                created_at: Utc::now().naive_utc(),
                last_used: None,
                active: false,
                role: vec![],
                password_reset_token: None,
                password_reset_requested_at: None,
                updated_at: Utc::now().naive_utc(),
                last_cart_id: None,
                accepted_terms_date: None,
                invited_at: None,
            };
            audience = vec![user];
        }

        for user in audience {
            match broadcast.channel {
                BroadcastChannel::PushNotification => {
                    queue_push_notification(&broadcast, message.to_string(), &user, conn)?;
                }
                BroadcastChannel::Email => {
                    queue_email_notification(&broadcast, conn, self.template_id.clone(), message.to_string(), &user)?
                }
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
) -> Result<(), BigNeonError> {
    if user.email.is_none() {
        return Ok(());
    }

    DomainAction::create(
        None,
        DomainActionTypes::Communication,
        Some(CommunicationChannelType::Email),
        serde_json::to_value(Communication::new(
            CommunicationType::EmailTemplate,
            broadcast.subject.as_ref().unwrap_or(&broadcast.name).to_string(),
            Some(message),
            None,
            CommAddress::from(user.email.clone().unwrap()),
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
