use chrono::prelude::*;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::*;
use schema::broadcasts;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Default, Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "broadcasts"]
pub struct NewBroadcast {
    pub event_id: Uuid,
    pub notification_type: BroadcastType,
    pub channel: BroadcastChannel,
    pub name: String,
    pub message: Option<String>,
    pub send_at: Option<NaiveDateTime>,
    pub status: BroadcastStatus,
    pub progress: i32,
}

#[derive(Queryable, Identifiable, Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "broadcasts"]
pub struct Broadcast {
    pub id: Uuid,
    pub event_id: Uuid,
    pub notification_type: BroadcastType,
    pub channel: BroadcastChannel,
    pub name: String,
    pub message: Option<String>,
    pub send_at: Option<NaiveDateTime>,
    pub status: BroadcastStatus,
    pub progress: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Default, Deserialize)]
#[table_name = "broadcasts"]
pub struct BroadcastEditableAttributes {
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub notification_type: Option<BroadcastType>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub channel: Option<BroadcastChannel>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "double_option_deserialize_unless_blank")]
    pub send_at: Option<Option<NaiveDateTime>>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub status: Option<BroadcastStatus>,
}

impl Broadcast {
    pub fn create(
        event_id: Uuid,
        notification_type: BroadcastType,
        channel: BroadcastChannel,
        name: String,
        message: Option<String>,
        send_at: Option<NaiveDateTime>,
        status: Option<BroadcastStatus>,
    ) -> NewBroadcast {
        NewBroadcast {
            event_id,
            notification_type,
            channel,
            name,
            message,
            send_at,
            status: status.unwrap_or(BroadcastStatus::Pending),
            progress: 0,
        }
    }

    pub fn find(id: Uuid, connection: &PgConnection) -> Result<Broadcast, DatabaseError> {
        broadcasts::table
            .filter(broadcasts::id.eq(id))
            .get_result(connection)
            .to_db_error(ErrorCode::QueryError, "Unable to load push notification")
    }

    pub fn find_by_event_id(
        event_id: Uuid,
        page: u32,
        limit: u32,
        connection: &PgConnection,
    ) -> Result<Payload<Broadcast>, DatabaseError> {
        let total: i64 = broadcasts::table
            .filter(broadcasts::event_id.eq(event_id))
            .count()
            .first(connection)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not get total push notifications for event",
            )?;

        let notifications = broadcasts::table
            .filter(broadcasts::event_id.eq(event_id))
            .limit(limit as i64)
            .offset((limit * page) as i64)
            .select(broadcasts::all_columns)
            .order_by(broadcasts::send_at.asc())
            .load(connection)
            .to_db_error(
                ErrorCode::QueryError,
                "Unable to load push notification by event",
            )?;

        let mut paging = Paging::new(page, limit);
        paging.total = total as u64;
        Ok(Payload {
            paging,
            data: notifications,
        })
    }

    pub fn cancel(&self, connection: &PgConnection) -> Result<Broadcast, DatabaseError> {
        let attributes: BroadcastEditableAttributes = BroadcastEditableAttributes {
            notification_type: None,
            channel: None,
            name: None,
            send_at: None,
            status: Some(BroadcastStatus::Cancelled),
        };

        self.update(attributes, connection)
    }

    pub fn update(
        &self,
        attributes: BroadcastEditableAttributes,
        connection: &PgConnection,
    ) -> Result<Broadcast, DatabaseError> {
        match self.status {
            BroadcastStatus::Cancelled => Err(DatabaseError::new(
                ErrorCode::UpdateError,
                Some("This broadcast has been cancelled, it cannot be modified.".to_string()),
            )),
            _ => DatabaseError::wrap(
                ErrorCode::UpdateError,
                "Could not update broadcast",
                diesel::update(self)
                    .set((attributes, broadcasts::updated_at.eq(dsl::now)))
                    .get_result(connection),
            ),
        }
    }

    pub fn set_in_progress(self, connection: &PgConnection) -> Result<Broadcast, DatabaseError> {
        let attributes = BroadcastEditableAttributes {
            status: Some(BroadcastStatus::InProgress),
            ..Default::default()
        };

        self.update(attributes, connection)
    }
}

impl NewBroadcast {
    pub fn commit(&self, connection: &PgConnection) -> Result<Broadcast, DatabaseError> {
        if self.notification_type == BroadcastType::Custom
            && self
                .message
                .clone()
                .map_or_else(|| false, |x| x.trim().len() > 0)
                == false
        {
            return DatabaseError::business_process_error(
                "Message cannot be blank if broadcast type is custom",
            );
        }

        let result: Broadcast = DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new push notification",
            diesel::insert_into(broadcasts::table)
                .values(self)
                .get_result(connection),
        )?;

        let mut action = DomainAction::create(
            None,
            DomainActionTypes::BroadcastPushNotification,
            None,
            json!(BroadcastPushNotificationAction {
                event_id: self.event_id,
            }),
            Some(Tables::Broadcasts.to_string()),
            Some(result.id),
        );
        if let Some(send_at) = self.send_at {
            action.schedule_at(send_at);
        }

        action.commit(connection)?;

        Ok(result)
    }
}

#[derive(Serialize, Deserialize)]
pub struct BroadcastPushNotificationAction {
    pub event_id: Uuid,
}
