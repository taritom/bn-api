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
                Some(format!("first{}", x).to_string()),
                Some(format!("last{}", x).to_string()),
                format!("email{}@test.com", x),
                Some(format!("222{}", x).to_string()),
                &format!("password{}", x),
            )
            .commit(&connection)
            .unwrap()
        })
    });
}
