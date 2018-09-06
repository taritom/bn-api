use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use std::sync::Arc;

type R2D2PooledConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub enum ConnectionType {
    Pg(Arc<PgConnection>),
    R2D2(Arc<R2D2PooledConnection>),
}
