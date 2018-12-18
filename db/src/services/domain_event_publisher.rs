use diesel::prelude::*;
use models::*;
use std::collections::HashMap;
use utils::errors::DatabaseError;

pub struct DomainEventPublisher<T> {
    subscriptions: HashMap<DomainEventTypes, Vec<Box<T>>>,
}

impl<T> DomainEventPublisher<T>
where
    T: Fn(&DomainEvent) -> Option<NewDomainAction>,
{
    pub fn new() -> DomainEventPublisher<T> {
        DomainEventPublisher {
            subscriptions: HashMap::new(),
        }
    }

    pub fn add_subscription(&mut self, domain_event_type: DomainEventTypes, factory: T) {
        let item = self.subscriptions.entry(domain_event_type);

        item.or_insert(vec![]).push(Box::new(factory));
    }

    pub fn publish(&self, event: DomainEvent, conn: &PgConnection) -> Result<(), DatabaseError> {
        let actions = self.subscriptions.get(&event.event_type);
        if let Some(action_list) = actions {
            for action_factory in action_list.iter() {
                if let Some(action) = action_factory(&event) {
                    action.commit(conn)?;
                }
            }
        }
        event.mark_as_published(conn)?;
        Ok(())
    }
}
