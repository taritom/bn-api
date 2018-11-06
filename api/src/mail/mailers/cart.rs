use bigneon_db::models::DisplayOrder;
use config::Config;
use errors::*;
use utils::communication::*;

pub fn purchase_completed(
    user_first_name: &String,
    user_email: &String,
    display_order: DisplayOrder,
    config: &Config,
) -> Result<(), BigNeonError> {
    let source = CommAddress::from(&config.communication_default_source_email);
    let destinations = CommAddress::from(&user_email);
    let title = "BigNeon Purchase Completed".to_string();
    let template_id = config.sendgrid_template_bn_purchase_completed.clone();
    let mut template_data = TemplateData::new();
    template_data.insert(String::from("name"), user_first_name.clone());
    //Construct an itemised breakdown using a HTML table
    let mut item_breakdown = r#"<table style="width:100%"><tbody>"#.to_string();
    item_breakdown
        .push_str("<tr><th>Units</th><th>Description</th><th>Unit Price</th><th>Total</th></tr>");
    for oi in &display_order.items {
        item_breakdown.push_str("<tr><th>");
        item_breakdown.push_str(&oi.quantity.to_string());
        item_breakdown.push_str("</th><th>");
        item_breakdown.push_str(&oi.description);
        item_breakdown.push_str("</th><th>$");
        item_breakdown.push_str(&(oi.unit_price_in_cents as f64 / 100.0).to_string());
        item_breakdown.push_str("</th><th>$");
        item_breakdown
            .push_str(&((oi.quantity * oi.unit_price_in_cents) as f64 / 100.0).to_string());
        item_breakdown.push_str("</th></tr>");
    }
    item_breakdown.push_str("</tbody></table>");

    template_data.insert(
        "ticket_count".to_string(),
        display_order.items.len().to_string(),
    );
    template_data.insert(
        "total_price".to_string(),
        (display_order.total_in_cents as f64 / 100.0).to_string(),
    );
    template_data.insert("item_breakdown".to_string(), item_breakdown);

    Communication::new(
        CommunicationType::EmailTemplate,
        title,
        None,
        Some(source),
        destinations,
        Some(template_id),
        Some(vec![template_data]),
    ).send(&config)
}
