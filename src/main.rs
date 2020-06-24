#[macro_use] extern crate serde_derive;

#[macro_use] mod sql;
mod route;
mod graphql;
mod state;
mod argument;
mod error;

use route::routes;
use state::{State, db::init_pool};
use argument::parse_arguments;

const DEFAULT_PORT: &'static str = "80";
const PG_CONFIG: &'static str = "host=postgres-server user=postgres dbname=postgres";
const PG_CONFIG_DEV: &'static str = "host=localhost user=postgres dbname=postgres";

fn main() {
    ::std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args = parse_arguments();
    let port = args.value_of("port").unwrap().parse::<u16>().expect("parse argument PORT.");
    let pg_config = if args.is_present("dev") {
        PG_CONFIG_DEV
    } else {
        PG_CONFIG
    };

    let db_pool = init_pool(pg_config, 1);
    let routes = routes(State::init(db_pool));

    warp::serve(routes).run(([0, 0, 0, 0], port));
}