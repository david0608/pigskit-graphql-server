use juniper::Context;

pub mod db;
#[macro_use] mod sql;

#[derive(Clone)]
pub struct State {
    db_pool: db::Pool
}

impl State {
    pub fn init(db_pool: db::Pool) -> Self {
        State {
            db_pool: db_pool,
        }
    }

    pub fn db_pool(&self) -> &db::Pool {
        &self.db_pool
    }
}

impl Context for State {}