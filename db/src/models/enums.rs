use std::fmt;
use std::str::FromStr;

macro_rules! string_enum {
    ($name:ident [$($value:ident),+]) => {

            #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
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

            impl FromStr for $name {
                type Err = &'static str;

                fn from_str(s: &str) -> Result<Self, &'static str> {
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
string_enum! { DomainEventTypes [PaymentCreated, PaymentCompleted, PaymentMethodCreated, PaymentMethodUpdated]}
string_enum! { EventStatus [Draft,Closed,Published,Offline]}
string_enum! { HoldTypes [Discount, Comp] }
string_enum! { OrderStatus [Draft, PartiallyPaid, Paid, Cancelled] }
string_enum! { OrderItemTypes [Tickets, PerUnitFees, EventFees]}
string_enum! { OrderTypes [Cart, BackOffice] }
string_enum! { PaymentMethods [External, CreditCard] }
string_enum! { PaymentStatus [Authorized, Completed] }
string_enum! { Roles [Admin, OrgMember, OrgOwner, User] }
string_enum! { Tables [Payments, PaymentMethods] }
string_enum! { TicketInstanceStatus [Available, Reserved, Purchased, Redeemed, Nullified]}
string_enum! { TicketPricingStatus [Published, Deleted] }
string_enum! { TicketTypeStatus [NoActivePricing, Published, SoldOut] }

#[test]
fn display() {
    assert_eq!(Roles::Admin.to_string(), "Admin");
    assert_eq!(Roles::OrgMember.to_string(), "OrgMember");
    assert_eq!(Roles::OrgOwner.to_string(), "OrgOwner");
    assert_eq!(Roles::User.to_string(), "User");
}

#[test]
fn parse() {
    assert_eq!(Roles::Admin, "Admin".parse::<Roles>().unwrap());
    assert_eq!(Roles::OrgMember, "OrgMember".parse::<Roles>().unwrap());
    assert_eq!(Roles::OrgOwner, "OrgOwner".parse::<Roles>().unwrap());
    assert!("Invalid Role".parse::<Roles>().is_err());
}
