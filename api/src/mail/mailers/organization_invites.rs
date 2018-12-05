use bigneon_db::models::{Organization, OrganizationInvite};
use config::Config;
use diesel::pg::PgConnection;
use errors::*;
use utils::communication::CommAddress;
use utils::communication::Communication;
use utils::communication::CommunicationType;
use utils::communication::TemplateData;

pub fn invite_user_to_organization_email(
    config: &Config,
    invite: &OrganizationInvite,
    org: &Organization,
    recipient_name: &str,
    conn: &PgConnection,
) -> Result<(), BigNeonError> {
    let invite_link_accept = format!(
        "{}/invites/accept?token={}",
        config.front_end_url.clone(),
        invite.security_token.expect("Security token is not set")
    );
    let invite_link_decline = format!(
        "{}/invites/decline?token={}",
        config.front_end_url.clone(),
        invite.security_token.expect("Security token is not set")
    );

    let source = CommAddress::from(&config.communication_default_source_email);
    let destinations = CommAddress::from(&invite.user_email);
    let title = "BigNeon Invites".to_string();
    let template_id = config.sendgrid_template_bn_org_invite.clone();
    let mut template_data = TemplateData::new();
    template_data.insert("name".to_string(), recipient_name.into());
    template_data.insert("org".to_string(), org.name.clone());
    template_data.insert("invite_link_accept".to_string(), invite_link_accept);
    template_data.insert("invite_link_decline".to_string(), invite_link_decline);
    Communication::new(
        CommunicationType::EmailTemplate,
        title,
        None,
        Some(source),
        destinations,
        Some(template_id),
        Some(vec![template_data]),
    ).queue(conn)
}
