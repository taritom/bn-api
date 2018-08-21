use bigneon_db::models::*;
use chrono::NaiveDate;
use support::project::TestProject;
use uuid::Uuid;

pub struct EventBuilder<'a> {
    organization_id: Option<Uuid>,
    venue_id: Option<Uuid>,
    project: &'a mut TestProject,
}

impl<'a> EventBuilder<'a> {
    pub fn new(project: &mut TestProject) -> EventBuilder {
        EventBuilder {
            organization_id: None,
            venue_id: None,
            project,
        }
    }

    pub fn finish(&mut self) -> Event {
        Event::create(
            "event name",
            self.organization_id
                .or_else(|| Some(self.project.create_organization().finish().id))
                .unwrap(),
            self.venue_id
                .or_else(|| Some(self.project.create_venue().finish().id))
                .unwrap(),
            NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
        ).commit(self.project)
            .unwrap()
    }
}
