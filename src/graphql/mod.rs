use juniper::{
    RootNode,
    EmptyMutation,
};

mod context;
mod user;
mod order;
mod shop;
mod guest;

pub use context::Context;

pub struct QueryRoot;

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    fn user() -> user::QueryUser {
        user::QueryUser
    }

    fn shop() -> shop::QueryShop {
        shop::QueryShop
    }

    fn guest() -> guest::QueryGuest {
        guest::QueryGuest
    }
}

type Schema = RootNode<'static, QueryRoot, EmptyMutation<Context>>;

pub fn schema() -> Schema {
    Schema::new(QueryRoot, EmptyMutation::new())
}