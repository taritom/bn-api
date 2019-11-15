use std::fmt::{Display, Error, Formatter};

#[derive(Serialize, Debug)]
pub struct FBID(pub String);

impl Display for FBID {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", &self.0)
    }
}
