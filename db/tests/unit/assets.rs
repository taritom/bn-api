use bigneon_db::models::Asset;
use chrono::NaiveDate;
use support::project::TestProject;
use uuid::Uuid;

#[test]
fn update_asset_blockchain_id() {
    let db = TestProject::new();
    let event = db.create_event().finish();
    let sd = NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11);
    let ed = NaiveDate::from_ymd(2016, 7, 9).and_hms(4, 10, 11);
    let ticket_type = event
        .add_ticket_type("VIP".to_string(), 100, sd, ed, &db.get_connection())
        .unwrap();

    let asset = Asset::find_by_ticket_type(&ticket_type.id, &db.get_connection()).unwrap();
    let tari_asset_id = Uuid::new_v4().to_string();
    let asset = asset
        .update_blockchain_id(tari_asset_id.clone(), &db.get_connection())
        .unwrap();

    assert_eq!(asset.blockchain_asset_id, Some(tari_asset_id));
}
