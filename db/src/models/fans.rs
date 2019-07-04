use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Queryable, Serialize)]
pub struct DisplayFan {
    pub user_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub thumb_profile_pic_url: Option<String>,
    pub organization_id: Uuid,
    pub order_count: Option<i64>,
    pub created_at: NaiveDateTime,
    pub first_order_time: Option<NaiveDateTime>,
    pub last_order_time: Option<NaiveDateTime>,
    pub revenue_in_cents: Option<i64>,
}
