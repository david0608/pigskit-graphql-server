use juniper::{
    RootNode,
};
use uuid::Uuid;
use crate::{
    state::State,
    error::Error
};

mod account;
mod user;
mod shop;

pub struct Context {
    state: State,
    user_session_id: Option<Uuid>,
}

impl Context {
    pub fn new(state: State, user_session_id: Option<Uuid>) -> Self {
        Context {
            state: state,
            user_session_id: user_session_id,
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn user_session_id(&self) -> Result<Uuid, Error> {
        if let Some(id) = self.user_session_id {
            Ok(id)
        } else {
            Err(Error::no_valid_cookie("USSID"))
        }
    }
}

impl juniper::Context for Context {}

pub struct QueryRoot;

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    fn account() -> account::QueryAccount {
        account::QueryAccount
    }

    fn user() -> user::QueryUser {
        user::QueryUser
    }

    fn shop() -> shop::QueryShop {
        shop::QueryShop
    }
}

pub struct MutationRoot;

#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
    fn account() -> account::MutationAccount {
        account::MutationAccount
    }
}

type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot)
}