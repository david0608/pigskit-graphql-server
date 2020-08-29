use warp::{
    Filter,
    reply::Reply,
    filters::BoxedFilter,
    cookie,
    path,
    options,
};
use juniper_warp::{
    make_graphql_filter,
    graphiql_filter,
};
use uuid::Uuid;
use crate::{
    graphql::{
        Context,
        schema,
    },
    state::State,
};

fn context_filter(state: State) -> BoxedFilter<(Context,)> {
    cookie::optional("USSID")
    .and(cookie::optional("GSSID"))
    .map(move |user_session_cookie: Option<String>, guest_session_cookie: Option<String>| {
        let user_session_id = if let Some(cookie) = user_session_cookie {
            if let Ok(id) = Uuid::parse_str(cookie.as_str()) {
                Some(id)
            } else {
                None
            }
        } else {
            None
        };

        let guest_session_id = if let Some(cookie) = guest_session_cookie {
            if let Ok(id) = Uuid::parse_str(cookie.as_str()) {
                Some(id)
            } else {
                None
            }
        } else {
            None
        };

        Context::new(
            state.clone(),
            user_session_id,
            guest_session_id,
        )
    })
    .boxed()
}

pub fn routes(state: State) -> BoxedFilter<(impl Reply,)> {
    path("graphql").and(
        make_graphql_filter(schema(), context_filter(state.clone()))
    )
    .or(
        path("graphiql").and(
            graphiql_filter("/graphql")
        )
    )
    .boxed()
}

pub fn dev_routes(state: State) -> BoxedFilter<(impl Reply,)> {
    routes(state)
    .or(
        // filter for preflight requests.
        options().map(warp::reply)
    )
    .map(|reply| warp::reply::with_header(reply, "Access-Control-Allow-Headers", "Content-Type"))
    .map(|reply| warp::reply::with_header(reply, "Access-Control-Allow-Credentials", "true"))
    .map(|reply| warp::reply::with_header(reply, "Access-Control-Allow-Origin", "http://localhost:3000"))
    .map(|reply| warp::reply::with_header(reply, "Access-Control-Allow-Methods", "GET, POST, OPTIONS, PUT, PATCH, DELETE"))
    .boxed()
}