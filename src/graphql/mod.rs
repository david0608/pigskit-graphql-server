use juniper::RootNode;
use crate::state::State;

mod account;

use account::{QueryAccount, MutationAccount};

pub struct QueryRoot;

#[juniper::graphql_object(Context = State)]
impl QueryRoot {
    fn account() -> QueryAccount {
        QueryAccount
    }
}

pub struct MutationRoot;

#[juniper::graphql_object(Context = State)]
impl MutationRoot {
    fn account() -> MutationAccount {
        MutationAccount
    }
}

type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot)
}