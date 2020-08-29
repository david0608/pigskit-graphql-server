use uuid::Uuid;
use crate::{
    state::State,
    error::Error
};

pub struct Context {
    state: State,
    user_session_id: Option<Uuid>,
    guest_session_id: Option<Uuid>,
}

impl Context {
    pub fn new(
        state: State,
        user_session_id: Option<Uuid>,
        guest_session_id: Option<Uuid>,
    ) -> Self {
        Context {
            state: state,
            user_session_id: user_session_id,
            guest_session_id: guest_session_id,
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

    pub fn guest_session_id(&self) -> Result<Uuid, Error> {
        if let Some(id) = self.guest_session_id {
            Ok(id)
        } else {
            Err(Error::no_valid_cookie("GSSID"))
        }
    }
}

impl juniper::Context for Context {}