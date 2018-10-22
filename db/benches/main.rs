#[macro_use]
extern crate criterion;
extern crate bigneon_db;
extern crate diesel;
extern crate dotenv;
extern crate rand;
extern crate uuid;

use criterion::Criterion;

mod carts;
mod users;

use carts::*;

use users::*;

criterion_group!(benches, benchmark_carts, benchmark_user_create);
criterion_main!(benches);
