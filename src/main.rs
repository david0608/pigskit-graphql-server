#[macro_use] extern crate serde_derive;

#[macro_use] mod sql;
mod route;
mod graphql;
mod state;
mod argument;
mod error;

use state::{State, db::init_pool};

const DEFAULT_PORT: u16 = 80;
const DEFAULT_PORT_DEV: u16 = 8000;
const PG_CONFIG: &'static str = "host=postgres-server user=postgres dbname=postgres";
const PG_CONFIG_DEV: &'static str = "host=localhost user=postgres dbname=postgres";

fn main() {
    ::std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let mut port = DEFAULT_PORT;
    let mut pg_config = PG_CONFIG;

    let args = argument::parse_arguments();
    let is_dev = args.is_present("dev");
    if is_dev {
        port = DEFAULT_PORT_DEV;
        pg_config = PG_CONFIG_DEV;
    }
    if let Some(p) = argument::args_port(&args) {
        port = p;
    }

    let db_pool = init_pool(pg_config, 1);

    if is_dev {
        warp::serve(route::dev_routes(State::init(db_pool))).run(([0, 0, 0, 0], port));
    } else {
        warp::serve(route::routes(State::init(db_pool))).run(([0, 0, 0, 0], port));
    }
}