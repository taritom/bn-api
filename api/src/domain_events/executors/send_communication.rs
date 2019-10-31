use bigneon_db::prelude::*;
use config::Config;
use db::Connection;
use domain_events::executor_future::ExecutorFuture;
use domain_events::routing::DomainActionExecutor;
use utils::communication;

pub struct SendCommunicationExecutor {
    config: Config,
}

impl SendCommunicationExecutor {
    pub fn new(config: Config) -> SendCommunicationExecutor {
        SendCommunicationExecutor { config }
    }
}

impl DomainActionExecutor for SendCommunicationExecutor {
    fn execute(&self, action: DomainAction, conn: Connection) -> ExecutorFuture {
        let future = communication::send_async(&action, &self.config, conn.get());
        ExecutorFuture::new(action, conn, Box::new(future))
    }
}
