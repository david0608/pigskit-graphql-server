use warp::{
    Filter,
    reply::Reply,
    filters::BoxedFilter,
    cookie,
    path,
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
    .map(move |user_session_cookie: Option<String>| {
        let user_session_id = if let Some(cookie) = user_session_cookie {
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