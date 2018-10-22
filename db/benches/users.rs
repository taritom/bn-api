use bigneon_db::dev::*;
use bigneon_db::models::*;
use criterion::Criterion;
use diesel::prelude::*;
use dotenv::dotenv;
use rand;
use std::env;
use uuid::Uuid;

pub fn benchmark_user_create(c: &mut Criterion) {
    dotenv().ok();
    let conn_str = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be defined.");

    c.bench_function("Create user", move |b| {
        b.iter(|| {
            let connection = PgConnection::establish(&conn_str).unwrap();
            let x: usize = rand::random();
            User::create(
                &format!("first{}", x),
                &format!("last{}", x),
                &format!("email{}@test.com", x),
                &format!("222{}", x),
                &format!("password{}", x),
            ).commit(&connection)
            .unwrap()
        })
    });
}
