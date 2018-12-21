use models::Roles;
use std::fmt;

#[derive(PartialEq, Debug, Copy, Clone, Eq, Ord, PartialOrd, Serialize)]
pub enum Scopes {
    ArtistWrite,
    CodeRead,
    CodeWrite,
    CompRead,
    CompWrite,
    DashboardRead,
    EventWrite,
    EventInterest,
    EventScan,
    EventViewGuests,
    HoldRead,
    HoldWrite,
    OrderMakeExternalPayment,
    OrderRead,
    OrgAdmin,
    OrgRead,
    OrgReadFans,
    OrgWrite,
    OrgManageAdminUsers,
    OrgManageUsers,
    RedeemTicket,
    RegionWrite,
    TicketAdmin,
    TicketRead,
    TicketTransfer,
    UserRead,
    VenueWrite,
}

impl fmt::Display for Scopes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Scopes::ArtistWrite => "artist:write",
            Scopes::CodeRead => "code:read",
            Scopes::CodeWrite => "code:write",
            Scopes::CompRead => "comp:read",
            Scopes::CompWrite => "comp:write",
            Scopes::DashboardRead => "dashboard:read",
            Scopes::EventWrite => "event:write",
            Scopes::EventInterest => "event:interest",
            Scopes::EventScan => "event:scan",
            Scopes::EventViewGuests => "event:view-guests",
            Scopes::HoldRead => "hold:read",
            Scopes::HoldWrite => "hold:write",
            Scopes::OrderRead => "order:read",
            Scopes::OrderMakeExternalPayment => "order::make-external-payment",
            Scopes::OrgAdmin => "org:admin",
            Scopes::OrgRead => "org:read",
            Scopes::OrgReadFans => "org:fans",
            Scopes::OrgWrite => "org:write",
            Scopes::OrgManageAdminUsers => "org:admin-users",
            Scopes::OrgManageUsers => "org:users",
            Scopes::RedeemTicket => "redeem:ticket",
            Scopes::RegionWrite => "region:write",
            Scopes::UserRead => "user:read",
            Scopes::VenueWrite => "venue:write",
            Scopes::TicketAdmin => "ticket:admin",
            Scopes::TicketRead => "ticket:read",
            Scopes::TicketTransfer => "ticket:transfer",
        };
        write!(f, "{}", s)
    }
}

pub fn get_scopes(roles: Vec<Roles>) -> Vec<String> {
    let scopes: Vec<Scopes> = roles
        .into_iter()
        .flat_map(|r| get_scopes_for_role(r))
        .collect();
    let mut scopes: Vec<String> = scopes.iter().map(|s| s.to_string()).collect();
    scopes.sort();
    scopes.dedup();
    scopes
}

fn get_scopes_for_role(role: Roles) -> Vec<Scopes> {
    use models::Roles::*;
    let mut roles = match role {
        User => {
            let mut roles = vec![
                Scopes::EventInterest,
                Scopes::OrderRead,
                Scopes::TicketTransfer,
            ];
            roles
        }
        DoorPerson => {
            let mut roles = vec![
                Scopes::RedeemTicket,
                Scopes::HoldRead,
                Scopes::EventScan,
                Scopes::TicketRead,
            ];
            roles
        }
        OrgBoxOffice => {
            let mut roles = vec![
                Scopes::DashboardRead,
                Scopes::EventViewGuests,
                Scopes::OrderMakeExternalPayment,
            ];
            roles.extend(get_scopes_for_role(Roles::DoorPerson));
            roles
        }
        OrgMember => {
            let mut roles = vec![
                Scopes::ArtistWrite,
                Scopes::CodeRead,
                Scopes::CodeWrite,
                Scopes::CompRead,
                Scopes::CompWrite,
                Scopes::DashboardRead,
                Scopes::EventScan,
                Scopes::EventViewGuests,
                Scopes::EventWrite,
                Scopes::HoldRead,
                Scopes::HoldWrite,
                Scopes::OrgRead,
                Scopes::OrgReadFans,
                Scopes::RedeemTicket,
                Scopes::TicketAdmin,
                Scopes::TicketRead,
                Scopes::VenueWrite,
            ];
            roles.extend(get_scopes_for_role(Roles::User));
            roles
        }
        OrgAdmin => {
            let mut roles = vec![Scopes::OrgWrite, Scopes::UserRead, Scopes::OrgManageUsers];
            roles.extend(get_scopes_for_role(OrgMember));
            roles.extend(get_scopes_for_role(Roles::OrgBoxOffice));
            roles
        }
        OrgOwner => {
            let mut roles = vec![Scopes::OrgManageAdminUsers];
            roles.extend(get_scopes_for_role(Roles::OrgAdmin));
            roles
        }
        Admin => {
            let mut roles = vec![Scopes::OrgAdmin, Scopes::RegionWrite];
            roles.extend(get_scopes_for_role(OrgOwner));
            roles
        }
    };
    roles.sort();
    roles.dedup();

    roles
}

#[test]
fn get_scopes_for_role_test() {
    let res = get_scopes_for_role(Roles::OrgOwner);
    assert_eq!(
        vec![
            Scopes::ArtistWrite,
            Scopes::CodeRead,
            Scopes::CodeWrite,
            Scopes::CompRead,
            Scopes::CompWrite,
            Scopes::DashboardRead,
            Scopes::EventWrite,
            Scopes::EventInterest,
            Scopes::EventScan,
            Scopes::EventViewGuests,
            Scopes::HoldRead,
            Scopes::HoldWrite,
            Scopes::OrderMakeExternalPayment,
            Scopes::OrderRead,
            Scopes::OrgRead,
            Scopes::OrgReadFans,
            Scopes::OrgWrite,
            Scopes::OrgManageAdminUsers,
            Scopes::OrgManageUsers,
            Scopes::RedeemTicket,
            Scopes::TicketAdmin,
            Scopes::TicketRead,
            Scopes::TicketTransfer,
            Scopes::UserRead,
            Scopes::VenueWrite,
        ],
        res
    );
}

#[test]
fn scopes_to_string() {
    assert_eq!("org:admin".to_string(), Scopes::OrgAdmin.to_string());
}

#[test]
fn get_scopes_test() {
    let mut res = get_scopes(vec![Roles::OrgOwner]);
    res.sort();
    assert_eq!(
        vec![
            "artist:write",
            "code:read",
            "code:write",
            "comp:read",
            "comp:write",
            "dashboard:read",
            "event:interest",
            "event:scan",
            "event:view-guests",
            "event:write",
            "hold:read",
            "hold:write",
            "order::make-external-payment",
            "order:read",
            "org:admin-users",
            "org:fans",
            "org:read",
            "org:users",
            "org:write",
            "redeem:ticket",
            "ticket:admin",
            "ticket:read",
            "ticket:transfer",
            "user:read",
            "venue:write",
        ],
        res
    );
    let mut res = get_scopes(vec![Roles::Admin]);
    res.sort();
    assert_eq!(
        vec![
            "artist:write",
            "code:read",
            "code:write",
            "comp:read",
            "comp:write",
            "dashboard:read",
            "event:interest",
            "event:scan",
            "event:view-guests",
            "event:write",
            "hold:read",
            "hold:write",
            "order::make-external-payment",
            "order:read",
            "org:admin",
            "org:admin-users",
            "org:fans",
            "org:read",
            "org:users",
            "org:write",
            "redeem:ticket",
            "region:write",
            "ticket:admin",
            "ticket:read",
            "ticket:transfer",
            "user:read",
            "venue:write",
        ],
        res
    );

    let res = get_scopes(vec![Roles::OrgOwner, Roles::Admin]);
    assert_eq!(
        vec![
            "artist:write",
            "code:read",
            "code:write",
            "comp:read",
            "comp:write",
            "dashboard:read",
            "event:interest",
            "event:scan",
            "event:view-guests",
            "event:write",
            "hold:read",
            "hold:write",
            "order::make-external-payment",
            "order:read",
            "org:admin",
            "org:admin-users",
            "org:fans",
            "org:read",
            "org:users",
            "org:write",
            "redeem:ticket",
            "region:write",
            "ticket:admin",
            "ticket:read",
            "ticket:transfer",
            "user:read",
            "venue:write",
        ],
        res
    );
}
