use bigneon_db::models::*;
use support::project::TestProject;

pub struct VenueBuilder<'a> {
    test_project: &'a TestProject,
}

impl<'a> VenueBuilder<'a> {
    pub fn new(test_project: &TestProject) -> VenueBuilder {
        VenueBuilder {
            test_project: &test_project,
        }
    }

    pub fn finish(self) -> Venue {
        Venue::create("Name").commit(self.test_project).unwrap()
    }
}
