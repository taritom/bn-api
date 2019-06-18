use bigneon_db::prelude::*;
use config::Config;
use db::Connection;
use diesel::PgConnection;
use domain_events::executor_future::ExecutorFuture;
use domain_events::routing::DomainActionExecutor;
use errors::BigNeonError;
use futures::future;
use log::Level::*;
use std::default::Default;
use utils::marketing_contacts::BulkEventFanListImportAction;
use utils::sendgrid::contacts::{SGContactList, SGContactListResponse};
use uuid::Uuid;

use super::bulk_event_fan_list_import::BulkEventFanListImportPayload;

const LOG_TARGET: &'static str = "bigneon::domain_actions::marketing_contacts";
const DATE_FORMAT: &'static str = "%b %e, %Y"; // Jun 1, 2019

pub struct CreateEventListExecutor {
    config: Config,
}

impl CreateEventListExecutor {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateEventListPayload {
    pub event_id: Uuid,
}

impl DomainActionExecutor for CreateEventListExecutor {
    fn execute(&self, action: DomainAction, conn: Connection) -> ExecutorFuture {
        match self.perform_job(&action, &conn) {
            Ok(_) => ExecutorFuture::new(action, conn, Box::new(future::ok(()))),
            Err(e) => {
                jlog!(Error, LOG_TARGET, "Error in CreateEventListExecutor", {
                    "innerError": format!("{}", e)
                });
                ExecutorFuture::new(action, conn, Box::new(future::err(e)))
            }
        }
    }
}

impl CreateEventListExecutor {
    fn perform_job(
        &self,
        action: &DomainAction,
        connection: &Connection,
    ) -> Result<(), BigNeonError> {
        let payload = serde_json::from_value::<CreateEventListPayload>(action.payload.clone())?;

        jlog!(
            Info,
            LOG_TARGET,
            &format!(
                "Create event list task starting for event_id={}",
                payload.event_id
            ), {"event_id" : payload.event_id, "action_id" : action.id}
        );

        let conn = connection.get();

        let event = Event::find(payload.event_id, conn)?;
        let mut org = event.organization(conn)?;

        org.decrypt(&self.config.api_keys_encryption_key)?;

        let api_key = match org.sendgrid_api_key {
            Some(v) => v,
            None => {
                jlog!(Info, LOG_TARGET, &format!("No sendgrid api key for org {}", event.organization_id), { "event_id": event.id });
                return Ok(());
            }
        };

        jlog!(Info, LOG_TARGET, &format!("Ensuring that event {} has a sendgrid list", event.id), {
            "action_id": action.id,
            "event_id": event.id,
            "organization_id": event.organization_id,
        });

        let sg_list = self.ensure_event_list(api_key.as_str(), &event)?;
        jlog!(Info, LOG_TARGET, &format!("Creating sendgrid list '{}'", sg_list.name), {
            "action_id": action.id,
            "event_id": event.id,
            "organization_id": event.organization_id,
        });

        if event.sendgrid_list_id.is_none() || event.sendgrid_list_id.unwrap() != sg_list.id as i64
        {
            event.update(
                None,
                EventEditableAttributes {
                    sendgrid_list_id: Some(sg_list.id as i64),
                    ..Default::default()
                },
                conn,
            )?;
        }

        jlog!(Info, LOG_TARGET, &format!("Enqueing MarketingContactCreateEventList domain action for event={}", event.id), {
            "action_id": action.id,
            "event_id": event.id,
            "organization_id": event.organization_id,
        });

        // DomainEvent should be triggered here: something like EventMarketingContactListCreated #DomainEvents
        if !DomainAction::has_pending_action(
            DomainActionTypes::MarketingContactsBulkEventFanListImport,
            Tables::Events.to_string(),
            payload.event_id,
            conn,
        )? {
            self.enqueue_import_domain_action(conn, payload)?;
        }

        Ok(())
    }

    fn enqueue_import_domain_action(
        &self,
        conn: &PgConnection,
        payload: CreateEventListPayload,
    ) -> Result<DomainAction, BigNeonError> {
        let import_payload = BulkEventFanListImportPayload::new(payload.event_id);
        BulkEventFanListImportAction::from_payload(import_payload).enqueue(conn)
    }

    fn ensure_event_list(
        &self,
        api_key: &str,
        event: &Event,
    ) -> Result<SGContactListResponse, BigNeonError> {
        let date_str = event
            .event_start
            .map(|date| date.format(DATE_FORMAT).to_string())
            .unwrap();
        let name = format!("{} ({})", event.name, date_str);
        SGContactList::new(name).create_or_return(api_key)
    }
}
