#[derive(Deserialize)]
pub struct CreateTicketAllocationRequest {
    pub name: String,
    pub tickets_delta: i64,
}
