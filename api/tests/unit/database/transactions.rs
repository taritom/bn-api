use crate::support::database::create_connection_pool;
use actix_web::{dev, FromRequest};
use api::config::Config;
use api::database::{Connection, Database};
use db::prelude::*;
use diesel::connection::TransactionManager;
use diesel::{Connection as DieselConnection, PgConnection};

#[actix_rt::test]
async fn no_hanging_transaction_in_pool() {
    let mut config = Config::new(Environment::Test);
    config.connection_pool.min = 1;
    config.connection_pool.max = 1;
    let db = Database::from_config(&config);
    let conn = db.get_connection().expect("DB connection failed");
    conn.begin_transaction().unwrap();
    let tm = conn.get().transaction_manager();
    assert_eq!(TransactionManager::<PgConnection>::get_transaction_depth(tm), 1);
    drop(conn);

    let conn = db.get_connection().expect("DB connection failed");
    let tm = conn.get().transaction_manager();
    assert_eq!(TransactionManager::<PgConnection>::get_transaction_depth(tm), 0);
}

use crate::support::test_request::TestRequest;

#[actix_rt::test]
async fn wipes_transaction_for_request() {
    let pool_size = TestRequest::create().config.connection_pool.max;

    // check that pooled connections cleanup transactions
    for _ in 0..(pool_size * 2) {
        let request = TestRequest::create().request;
        let conn = Connection::from_request(&request, &mut dev::Payload::None)
            .await
            .expect("Failed to get connection from request");
        let tm = conn.get().transaction_manager();
        assert_eq!(TransactionManager::<PgConnection>::get_transaction_depth(tm), 1);
    }

    // check that all pooled connections do not have pending transactions
    let database = TestRequest::create().extract_state().await.database.clone();
    for _ in 0..(pool_size * 2) {
        let conn = database.get_connection().expect("failed to get connection");
        let tm = conn.get().transaction_manager();
        assert_eq!(TransactionManager::<PgConnection>::get_transaction_depth(tm), 0);
    }
}

#[actix_rt::test]
async fn diesel_pool_does_not_release_transaction() {
    let mut config = Config::new(Environment::Test);
    config.connection_pool.min = 1;
    config.connection_pool.max = 1;
    let pool = create_connection_pool(&config);
    let conn = pool.get().expect("failed to get pooled connection");
    conn.transaction_manager()
        .begin_transaction(&conn)
        .expect("failed to start transaction");
    let tm = conn.transaction_manager();
    assert_eq!(TransactionManager::<PgConnection>::get_transaction_depth(tm), 1);
    drop(conn);

    let conn = pool.get().expect("failed to get pooled connection");
    let tm = conn.transaction_manager();
    assert_eq!(TransactionManager::<PgConnection>::get_transaction_depth(tm), 1);
    drop(conn);
}
