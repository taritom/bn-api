#[derive(Serialize, Deserialize)]
pub enum CommunicationType {
    Email,
    EmailTemplate,
    Sms,
    Push,
}
