use diesel::connection::{SimpleConnection, TransactionManager};
use diesel::r2d2::{self, ConnectionManager};
use diesel::Connection as DieselConnection;
use diesel::PgConnection;

type R2D2PooledConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub enum ConnectionType {
    Pg(PgConnection),
    R2D2(R2D2PooledConnection),
}

// When connection dropped check if there is currently hanging transactions
impl Drop for ConnectionType {
    fn drop(&mut self) {
        if let Self::R2D2(ref mut conn) = self {
            // This is guaranteed checkpoint before returning PooledConnection to the Pool
            let mut transactions =
                TransactionManager::<PgConnection>::get_transaction_depth(conn.transaction_manager());
            let tm = conn.transaction_manager();
            while transactions > 0 {
                // rollback all transactions including save points
                // before returning PooledConnection to the pool
                if let Err(e) = tm.rollback_transaction(conn) {
                    error!("Diesel Error in Connection::drop: {}", e);
                    // failed to nicely rollback - last resort
                    if let Err(e) = conn.batch_execute("DISCONNECT") {
                        warn!("Failed to disconnect in Connection::drop: {}", e);
                    }
                    transactions = 0;
                } else {
                    if transactions == 1 {
                        warn!("PgConnection hanging transaction was rolled back");
                    }
                    transactions -= 1;
                }
            }
        }
    }
}
