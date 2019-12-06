use serde_json;
use std::borrow::Borrow;
use std::cmp;
use std::cmp::Ordering;
use std::collections::HashMap;
use utils::errors::DatabaseError;
use utils::errors::ParseError;

// Limit lookup to only check a certain number of fragments for matches when parsing
pub const MAXIMUM_STATE_FRAGMENTS: usize = 4;
pub const MAXIMUM_COUNTRY_FRAGMENTS: usize = 6;

#[derive(Clone, Deserialize)]
pub struct CountryLookup {
    pub country_data: Vec<CountryDatum>,
}

impl CountryLookup {
    pub fn new() -> Result<CountryLookup, DatabaseError> {
        Ok(CountryLookup {
            country_data: CountryDatum::load()?,
        })
    }

    pub fn find(&self, input: &str) -> Option<CountryDatum> {
        let input = input.to_lowercase();
        self.country_data
            .iter()
            .find(|c| c.code.to_lowercase() == input || c.name.to_lowercase() == input)
            .map(|c| c.clone())
    }

    pub fn parse_city_state_country(
        &self,
        input: &str,
    ) -> Result<Vec<(Option<String>, Option<StateDatum>, Option<CountryDatum>)>, DatabaseError> {
        let mut city_state_countries = Vec::new();
        let query_fragments: Vec<String> = input
            .split(|c| c == ',' || c == ' ')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let mut found_countries = Vec::new();
        let query_fragment_size = query_fragments.len();
        for i in 0..cmp::min(query_fragment_size, MAXIMUM_COUNTRY_FRAGMENTS) {
            let test_fragment = query_fragments[(query_fragment_size - 1 - i)..query_fragment_size].join(" ");
            if let Some(country_datum) = self.find(&test_fragment) {
                found_countries.push((
                    country_datum,
                    if query_fragment_size - i - 1 == 0 {
                        "".to_string()
                    } else {
                        query_fragments[..query_fragment_size - i - 1].join(" ")
                    },
                ));
            }
        }

        for (country, remaining_input) in found_countries {
            if remaining_input.len() == 0 {
                city_state_countries.push((None, None, Some(country.clone())));
            } else if let Ok(city_states) = country.parse_city_state(&remaining_input) {
                for (city, state) in city_states {
                    city_state_countries.push((city, state, Some(country.clone())));
                }
            }
        }

        if city_state_countries.len() > 0 {
            return Ok(city_state_countries);
        }

        return Err(ParseError {
            message: "Could not parse city state countries".to_string(),
            input: input.to_string(),
        }
        .into());
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct StateDatum {
    pub name: String,
    pub label: Option<String>,
    pub code: Option<String>,
    pub alternate_names: Option<Vec<String>>,
}

impl PartialOrd for StateDatum {
    fn partial_cmp(&self, other: &StateDatum) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StateDatum {
    fn cmp(&self, other: &StateDatum) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct CountryDatum {
    pub name: String,
    pub code: String,
    pub label: Option<String>,
    pub zip_label: String,
    pub zip_required: bool,
    pub province_label: String,
    pub provinces: Option<Vec<String>>,
    pub province_labels: Option<HashMap<String, String>>,
    pub province_codes: HashMap<String, String>,
    pub province_alternate_names: HashMap<String, Option<Vec<String>>>,
}

impl PartialOrd for CountryDatum {
    fn partial_cmp(&self, other: &CountryDatum) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CountryDatum {
    fn cmp(&self, other: &CountryDatum) -> Ordering {
        self.name.cmp(&other.name)
    }
}

fn compare_str(state: &str, state_compare: &str) -> bool {
    let mod_state = state_compare.to_lowercase();
    let mod_state = mod_state.trim();
    state.to_lowercase() == mod_state
}

impl CountryDatum {
    // convert a long US state name to it's 2 letters abbreviations
    pub fn convert_state(&self, state_compare: &str) -> Option<String> {
        let found_state = self
            .province_codes
            .iter()
            .find(|&t| compare_str(t.0.borrow(), state_compare));
        if found_state.is_none() && state_compare.trim().len() == 2 {
            return Some(state_compare.trim().to_uppercase().to_string());
        }
        found_state.map_or(None, |(_key, value)| Some(value.to_owned()))
    }

    pub fn parse_city_state(&self, input: &str) -> Result<Vec<(Option<String>, Option<StateDatum>)>, DatabaseError> {
        let mut potential_city_states = Vec::new();
        let query_fragments: Vec<String> = input
            .split(|c| c == ',' || c == ' ')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let query_fragment_size = query_fragments.len();
        for i in 0..cmp::min(query_fragment_size, MAXIMUM_STATE_FRAGMENTS) {
            let test_fragment = query_fragments[(query_fragment_size - 1 - i)..query_fragment_size].join(" ");
            if let Some(state_datum) = self.state(&test_fragment) {
                let mut city: Option<String> = None;
                if query_fragment_size - i - 1 > 0 {
                    city = Some(query_fragments[..query_fragment_size - i - 1].join(" "));
                }
                potential_city_states.push((city, Some(state_datum)));
            }
        }

        if potential_city_states.len() > 0 {
            return Ok(potential_city_states);
        }

        return Err(ParseError {
            message: "Could not parse city state".to_string(),
            input: input.to_string(),
        }
        .into());
    }

    pub fn state(&self, input: &str) -> Option<StateDatum> {
        let input = input.to_lowercase();

        // Find by province name
        if let Some(provinces) = &self.provinces {
            if let Some(province) = provinces.iter().find(|p| p.to_lowercase() == input) {
                let mut label: Option<String> = None;
                if let Some(province_labels) = &self.province_labels {
                    label = province_labels.get(province).map(|l| l.clone());
                }

                return Some(StateDatum {
                    name: province.to_string(),
                    label,
                    code: self.province_codes.get(province).map(|c| c.clone()),
                    alternate_names: self
                        .province_alternate_names
                        .get(province)
                        .map(|p| p.clone())
                        .unwrap_or(None),
                });
            }
        }

        // Find by province code
        if let Some((province, code)) = self.province_codes.iter().find(|(_, v)| v.to_lowercase() == input) {
            let mut label: Option<String> = None;
            if let Some(province_labels) = &self.province_labels {
                label = province_labels.get(province).map(|l| l.clone());
            }

            return Some(StateDatum {
                name: province.clone(),
                label,
                code: Some(code.clone()),
                alternate_names: self
                    .province_alternate_names
                    .get(province)
                    .map(|p| p.clone())
                    .unwrap_or(None),
            });
        }
        None
    }

    pub fn load() -> Result<Vec<CountryDatum>, DatabaseError> {
        Ok(serde_json::from_str(include_str!(
            "../../external/country-province-data/countries.json"
        ))?)
    }
}
