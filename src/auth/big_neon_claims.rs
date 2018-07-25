use bigneon_db::models::Roles;
use bigneon_db::models::User;
use std::time;
use std::time::Duration;
use std::time::UNIX_EPOCH;
use uuid::Uuid;

#[derive(RustcDecodable, RustcEncodable)]
pub struct BigNeonClaims {
    pub sub: String,
    pub iss: String,
    pub exp: u64,
    pub roles: String,
}

impl BigNeonClaims {
    pub fn new(user: &User, issuer: String) -> BigNeonClaims {
        let mut timer = time::SystemTime::now();
        timer += Duration::from_secs(86400);
        let exp = timer.duration_since(UNIX_EPOCH).unwrap().as_secs();

        BigNeonClaims {
            iss: issuer,
            sub: user.id.hyphenated().to_string(),
            exp,
            roles: user.role.join(","),
        }
    }

    pub fn get_roles(&self) -> Vec<Roles> {
        let roles: Vec<Roles> = self.roles
            .split(",")
            .map(|x| Roles::parse(x).unwrap())
            .collect();
        roles
    }

    pub fn get_id(&self) -> Uuid {
        Uuid::parse_str(&self.sub).unwrap()
    }
}

#[test]
fn get_roles() {
    let b = BigNeonClaims {
        sub: "Sub".into(),
        iss: "iss".into(),
        exp: 0,
        roles: "Guest,Admin".into(),
    };
    let roles = b.get_roles();
    assert_eq!(roles, vec![Roles::Guest, Roles::Admin]);
}
