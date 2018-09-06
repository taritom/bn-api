use actix_web::{FromRequest, HttpRequest, Result};
use db::*;
use diesel::connection::TransactionManager;
use diesel::Connection as DieselConnection;
use diesel::PgConnection;
use errors::BigNeonError;
use server::AppState;
use std::sync::Arc;

#[derive(Clone)]
pub struct Connection {
    inner: Arc<ConnectionType>,
}

impl From<ConnectionType> for Connection {
    fn from(connection_type: ConnectionType) -> Self {
        Connection {
            inner: Arc::new(connection_type),
        }
    }
}

impl From<Arc<PgConnection>> for Connection {
    fn from(connection: Arc<PgConnection>) -> Self {
        ConnectionType::Pg(connection.clone()).into()
    }
}

impl Connection {
    pub fn get(&self) -> &PgConnection {
        match *self.inner {
            ConnectionType::Pg(ref connection) => &*connection,
            ConnectionType::R2D2(ref connection) => &**connection,
        }
    }
}

impl FromRequest<AppState> for Connection {
    type Config = ();
    type Result = Result<Connection, BigNeonError>;

    fn from_request(request: &HttpRequest<AppState>, _config: &Self::Config) -> Self::Result {
        if let Some(connection) = request.extensions().get::<Connection>() {
            return Ok(connection.clone());
        }

        let connection = request.state().database.get_connection();
        {
            let connection_object = connection.get();
            connection_object
                .transaction_manager()
                .begin_transaction(connection_object)?;
        }
        request.extensions_mut().insert(connection.clone());
        Ok(connection)
    }
}
