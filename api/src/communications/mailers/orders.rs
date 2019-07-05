use bigneon_db::models::enums::OrderItemTypes;
use bigneon_db::prelude::{OrderItem, Refund};
use config::Config;
use diesel::PgConnection;
use errors::*;
use itertools::Itertools;
use utils::communication::*;

pub fn refund_email(
    user_first_name: &String,
    user_email: String,
    refund: &Refund,
    config: &Config,
    conn: &PgConnection,
) -> Result<(), BigNeonError> {
    let source = CommAddress::from(config.communication_default_source_email.clone());
    let destinations = CommAddress::from(user_email);
    let title = "BigNeon Refund".to_string();
    let template_id = config.sendgrid_template_bn_refund.clone();
    let mut template_data = TemplateData::new();
    template_data.insert(String::from("name"), user_first_name.clone());
    //Construct an itemised breakdown using a HTML table
    let mut item_breakdown = r#"<table style="width:100%"><tbody>"#.to_string();
    item_breakdown.push_str("<tr><th>Units Refunded</th><th>Description</th><th>Total</th></tr>");

    let items = refund
        .items(conn)?
        .into_iter()
        .map(|i| {
            let item = OrderItem::find(i.order_item_id, conn);
            return (i, item);
        })
        .collect_vec();

    let mut new_items = vec![];
    // unwrap results
    for (item, res) in items {
        new_items.push((item, res?));
    }
    let items = new_items;

    for (item, oi) in items.iter() {
        item_breakdown.push_str(r#"<tr><th align="center">"#);
        item_breakdown.push_str(&item.quantity.to_string());
        item_breakdown.push_str("</th><th>");
        item_breakdown.push_str(&oi.description(conn)?);
        item_breakdown.push_str(r#"</th><th align="right">$"#);
        item_breakdown.push_str(&format!("{:.*}", 2, item.amount as f64 / 100.0));

        item_breakdown.push_str("</th></tr>");
    }

    item_breakdown.push_str("</tbody></table>");
    let amount = items.iter().map(|i| i.0.amount).sum::<i64>() as f64 / 100.0;
    template_data.insert("amount_refunded".to_string(), amount.to_string());
    template_data.insert(
        "ticket_count".to_string(),
        items
            .iter()
            .filter(|i| i.1.item_type == OrderItemTypes::Tickets)
            .map(|i| i.0.quantity)
            .sum::<i64>()
            .to_string(),
    );
    let total_fees = items
        .iter()
        .filter(|i| i.1.item_type.is_fee())
        .map(|i| i.0.amount)
        .sum::<i64>();

    template_data.insert(
        "total_fees".to_string(),
        format!("{:.*}", 2, total_fees as f64 / 100.0),
    );
    template_data.insert("total_price".to_string(), format!("{:.*}", 2, amount));
    template_data.insert("item_breakdown".to_string(), item_breakdown);
    template_data.insert(
        "tickets_link".to_string(),
        format!("{}/orders", config.front_end_url),
    );

    // TODO: Perhaps move this to an event subscription
    Communication::new(
        CommunicationType::EmailTemplate,
        title,
        None,
        Some(source),
        destinations,
        Some(template_id),
        Some(vec![template_data]),
        Some(vec!["refund"]),
        None,
    )
    .queue(conn)
}
