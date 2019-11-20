use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use r2d2_redis::r2d2::PooledConnection;

type R2D2PooledConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub enum ConnectionType {
    Pg(PgConnection),
    R2D2(R2D2PooledConnection),
    Redis(PooledConnection<r2d2_redis::RedisConnectionManager>),
}
