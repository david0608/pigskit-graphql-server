use warp::Filter;

mod graphql;
mod state;
mod argument;
mod error;

use graphql::schema;
use argument::parse_arguments;
use state::{State, db::init_pool};

const DEFAULT_PORT: &'static str = "80";
const PG_CONFIG: &'static str = "host=postgres-server user=postgres dbname=postgres";

fn main() {
    ::std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args = parse_arguments();
    let port = args.value_of("port").unwrap().parse::<u16>().unwrap();

    let db_pool = init_pool(PG_CONFIG, 1);
    let state = State::init(db_pool);

    let state_filter = warp::any().map(move || state.clone()).boxed();

    let graphql_filter = warp::path("graphql").and(
        juniper_warp::make_graphql_filter(
            schema(),
            state_filter,
        )
    ).boxed();

    let graphiql_filter = warp::path("graphiql").and(
        juniper_warp::graphiql_filter("/graphql")
    ).boxed();

    warp::serve(
        graphql_filter
        .or(graphiql_filter)
    )
    .run(([0, 0, 0, 0], port));
}