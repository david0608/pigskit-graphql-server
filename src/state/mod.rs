use crate::error::Error;

pub mod db;

#[derive(Clone)]
pub struct State {
    db_pool: db::Pool,
}

impl State {
    pub fn init(db_pool: db::Pool) -> Self {
        State {
            db_pool: db_pool,
        }
    }

    pub fn db_connection(&self) -> Result<db::Connection, Error> {
        self.db_pool.get().map_err(|err| -> Error {
            err.into()
        })
    }
}