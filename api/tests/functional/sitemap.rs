use support::database::TestDatabase;
//use bigneon_db::prelude::*;
use bigneon_api::utils::gen_sitemap;

#[test]
fn index() {
    let database = TestDatabase::new();
    let connection = database.connection.get();
    let organization = database.create_organization().finish();
    let venue = database
        .create_venue()
        .with_city("San Francisco".to_string())
        .with_state("California".to_string())
        .with_country("US".to_string())
        .finish();

    let event = database
        .create_event()
        .with_name("NewEvent1".to_string())
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();

    let slug = database
        .create_slug()
        .for_event(&event)
        .with_slug("redirect-me")
        .finish();

    let urls: Vec<String> = Vec::new();
    println!("{:?}, {:?}, {:?}, ", venue, event, slug);
    let sitemap_xml_test = gen_sitemap::create_sitemap(&urls).unwrap();

    let sitemap_xml = gen_sitemap::create_sitemap_conn(connection, &"front_end_url".to_string()).unwrap();
    println!("{}", sitemap_xml);
    assert_eq!(sitemap_xml_test, sitemap_xml);
}

//fn check_if_url_included(url: &String, sitemap_xml: &String) -> bool {
// true
//}

