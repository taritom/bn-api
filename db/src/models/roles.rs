use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

#[derive(Debug, PartialEq, Clone)]
pub enum Roles {
    Admin,
    Guest,
    OrgMember,
    OrgOwner,
    User,
}

impl Roles {
    pub fn parse(s: &str) -> Result<Roles, &'static str> {
        match s {
            "Guest" => Ok(Roles::Guest),
            "Admin" => Ok(Roles::Admin),
            "OrgMember" => Ok(Roles::OrgMember),
            "OrgOwner" => Ok(Roles::OrgOwner),
            "User" => Ok(Roles::User),
            _ => Err("Could not parse role. Unexpected value occurred"),
        }
    }
}

impl Display for Roles {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Roles::Guest => write!(f, "Guest"),
            Roles::User => write!(f, "User"),
            Roles::OrgMember => write!(f, "OrgMember"),
            Roles::OrgOwner => write!(f, "OrgOwner"),
            Roles::Admin => write!(f, "Admin"),
        }
    }
}

#[test]
fn display() {
    assert_eq!(Roles::Admin.to_string(), "Admin");
    assert_eq!(Roles::Guest.to_string(), "Guest");
    assert_eq!(Roles::OrgMember.to_string(), "OrgMember");
    assert_eq!(Roles::OrgOwner.to_string(), "OrgOwner");
    assert_eq!(Roles::User.to_string(), "User");
}

#[test]
fn parse() {
    assert_eq!(Roles::Admin, Roles::parse("Admin").unwrap());
    assert_eq!(Roles::Guest, Roles::parse("Guest").unwrap());
    assert_eq!(Roles::OrgMember, Roles::parse("OrgMember").unwrap());
    assert_eq!(Roles::OrgOwner, Roles::parse("OrgOwner").unwrap());
    assert!(Roles::parse("Not role").is_err());
}
