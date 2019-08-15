use bigneon_db::models::*;
use config::Config;
use diesel::pg::PgConnection;
use errors::*;
use uuid::Uuid;

pub fn checkin_instructions(
    config: &Config,
    phone: String,
    order_id: Uuid,
    conn: &PgConnection,
) -> Result<(), BigNeonError> {
    let source = CommAddress::from(config.communication_default_source_phone.clone());
    let destinations = CommAddress::from(phone);
    let body = format!(
        "Thank you for your purchase! Your order number is {}, Please head to the entry and let the door person know your first and last name so that they can check you in. Also be sure to check your inbox for a link to download the Big Neon app to access your tickets for quicker entry",
        Order::parse_order_number(order_id)

    );

    Communication::new(
        CommunicationType::Sms,
        body,
        None,
        Some(source),
        destinations,
        None,
        None,
        Some(vec!["box-office"]),
        None,
    )
    .queue(conn)?;

    Ok(())
}
