use bigneon_db::prelude::*;

#[test]
fn new() {
    let country_lookup = CountryLookup::new().unwrap();
    assert_eq!(country_lookup.country_data.len(), 242);
}

#[test]
fn find() {
    let country_lookup = CountryLookup::new().unwrap();
    assert!(country_lookup.find("FAKE").is_none());

    let country = country_lookup.find("US").unwrap();
    assert_eq!(country.code, "US");
    assert_eq!(country.name, "United States");

    let country = country_lookup.find("zA").unwrap();
    assert_eq!(country.code, "ZA");
    assert_eq!(country.name, "South Africa");

    let country = country_lookup.find("JapaN").unwrap();
    assert_eq!(country.code, "JP");
    assert_eq!(country.name, "Japan");
}

#[test]
fn state() {
    let country_lookup = CountryLookup::new().unwrap();
    let country = country_lookup.find("US").unwrap();
    assert!(country.state("FAKE").is_none());

    let state = country.state("MA").unwrap();
    assert_eq!(state.name, "Massachusetts");
    assert_eq!(state.code, Some("MA".to_string()));
    let state = country.state("MaSSaChuseTTs").unwrap();
    assert_eq!(state.name, "Massachusetts");
    assert_eq!(state.code, Some("MA".to_string()));

    let country = country_lookup.find("JapaN").unwrap();
    let state = country.state("JP-02").unwrap();
    assert_eq!(state.name, "Aomori");
    assert_eq!(state.code, Some("JP-02".to_string()));
}
