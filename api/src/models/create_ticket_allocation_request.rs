#[derive(Deserialize)]
pub struct CreateTicketTypeRequest {
    pub name: String,
    pub capacity: u32,
}
