use bigneon_db::prelude::*;
use db::Connection;
use domain_events::executor_future::ExecutorFuture;
use domain_events::routing::DomainActionExecutor;
use errors::*;
use futures::future;
use log::Level::Error;

pub struct ProcessSettlementReportExecutor {}

impl DomainActionExecutor for ProcessSettlementReportExecutor {
    fn execute(&self, action: DomainAction, conn: Connection) -> ExecutorFuture {
        match self.perform_job(&action, &conn) {
            Ok(_) => ExecutorFuture::new(action, conn, Box::new(future::ok(()))),
            Err(e) => {
                jlog!(Error, "Process transfer drip action failed", {"action_id": action.id, "main_table_id": action.main_table_id, "error": e.to_string()});
                ExecutorFuture::new(action, conn, Box::new(future::err(e)))
            }
        }
    }
}

impl ProcessSettlementReportExecutor {
    pub fn new() -> ProcessSettlementReportExecutor {
        ProcessSettlementReportExecutor {}
    }

    pub fn perform_job(
        &self,
        action: &DomainAction,
        conn: &Connection,
    ) -> Result<(), BigNeonError> {
        let conn = conn.get();
        let id = action.main_table_id.clone().ok_or(ApplicationError::new(
            "No id supplied in the action".to_string(),
        ))?;

        match action.main_table.clone().ok_or(ApplicationError::new(
            "No table supplied in the action".to_string(),
        ))? {
            Tables::Organizations => {
                let organization = Organization::find(id, conn)?;
                if organization.can_process_settlements(conn)? {
                    Settlement::process_settlement_for_organization(&organization, conn)?;
                }
                organization.create_next_settlement_processing_domain_action(conn)?;
            }
            _ => return Err(ApplicationError::new("Table not supported".to_string()).into()),
        };

        Ok(())
    }
}
