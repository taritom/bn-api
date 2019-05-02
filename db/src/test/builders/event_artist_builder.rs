use diesel::prelude::*;
use models::{Artist, Event, EventArtist};
use test::builders::*;
use uuid::Uuid;

#[allow(dead_code)]
pub struct EventArtistBuilder<'a> {
    event_id: Option<Uuid>,
    artist_id: Option<Uuid>,
    connection: &'a PgConnection,
}

impl<'a> EventArtistBuilder<'a> {
    pub fn new(connection: &PgConnection) -> EventArtistBuilder {
        EventArtistBuilder {
            event_id: None,
            artist_id: None,
            connection,
        }
    }

    pub fn with_event(mut self, event: &Event) -> EventArtistBuilder<'a> {
        self.event_id = Some(event.id);
        self
    }

    pub fn with_artist(mut self, artist: &Artist) -> EventArtistBuilder<'a> {
        self.artist_id = Some(artist.id);
        self
    }

    pub fn finish(&self) -> EventArtist {
        let event_id = self
            .event_id
            .or_else(|| Some(EventBuilder::new(self.connection).finish().id))
            .unwrap();

        let artist_id = self
            .artist_id
            .or_else(|| Some(ArtistBuilder::new(self.connection).finish().id))
            .unwrap();

        EventArtist::create(event_id, artist_id, 1, None, 0, None)
            .commit(None, self.connection)
            .unwrap()
    }
}
