use std::fmt;

macro_rules! string_enum {
    ($name:ident [$($value:ident),+]) => {

            #[derive(Serialize, Deserialize, PartialEq, Debug)]
            pub enum $name {
                $(
                    $value,
                )*
            }

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                 let s = match self {
                      $(
                        $name::$value => stringify!($value),
                       )*
                    };
                    write!(f, "{}", s)
                }
            }

            impl $name {
                #[allow(dead_code)]
                pub fn parse(s: &str) -> Result<$name, &'static str> {
                  match s {
                      $(
                        stringify!($value) => Ok($name::$value),
                       )*
                        _ => Err("Could not parse value")
                    }
                }
            }
        }
}

string_enum! { AssetStatus [Unsynced] }
string_enum! { EventStatus [Draft,Closed,Published,Offline]}
string_enum! { OrderStatus [Draft, PendingPayment, Paid, Cancelled] }
// Potentially there will also be shipping or other items on an order
string_enum! { OrderItemTypes [Tickets]}
string_enum! { OrderTypes [Cart, BackOffice] }
string_enum! { TicketPricingStatus [Published] }
string_enum! { TicketTypeStatus [Published] }
