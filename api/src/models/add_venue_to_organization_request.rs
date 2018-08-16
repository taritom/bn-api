use uuid::Uuid;

#[derive(Deserialize)]
pub struct AddVenueToOrganizationRequest {
    pub organization_id: Uuid,
}
