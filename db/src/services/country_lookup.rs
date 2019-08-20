use serde_json;
use std::collections::HashMap;
use utils::errors::DatabaseError;

#[derive(Deserialize)]
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
}

#[derive(Clone, Deserialize)]
pub struct StateDatum {
    pub name: String,
    pub label: Option<String>,
    pub code: Option<String>,
    pub alternate_names: Option<Vec<String>>,
}

#[derive(Clone, Deserialize)]
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

impl CountryDatum {
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
        if let Some((province, code)) = self
            .province_codes
            .iter()
            .find(|(_, v)| v.to_lowercase() == input)
        {
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
