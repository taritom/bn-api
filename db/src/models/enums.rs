use serde_json;
use std::fmt;

macro_rules! string_enum {
    ($name:ident [$($value:ident),+]) => {

            #[derive(Serialize, Deserialize, PartialEq, Debug)]
            #[serde(rename_all = "kebab-case")]
            pub enum $name {
                $(
                    $value,
                )*
            }


            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                    write!(f, "{}", serde_json::to_string(self).unwrap())
                }
            }

            impl $name {
                #[allow(dead_code)]
                pub fn parse(s: &str) -> Result<$name, &'static str> {
                    serde_json::from_str(s).map_err(|_| "Could not parse value")
                }
            }
        }
}

string_enum! { CartStatus [Open, Completed] }
string_enum! { EventStatus [Draft,Closed,Published,Offline]}
string_enum! { OrderStatus [Unpaid, Paid, Cancelled] }
string_enum! { PricePointStatus [Published] }
string_enum! { TicketTypeStatus [Published] }
