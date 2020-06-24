use std::convert::From;
use juniper::FieldResult;
use postgres::row::Row;
use uuid::Uuid;
// use crate::state::State;
use crate::graphql::Context;
// use crate::error::Error;

pub struct Account {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub nick_name: String,
}

#[juniper::graphql_object]
impl Account {
    fn id(&self) -> Uuid {
        self.id
    }

    fn username(&self) -> &String {
        &self.username
    }

    fn password(&self) -> &String {
        &self.password
    }

    fn nick_name(&self) -> &String {
        &self.nick_name
    }
}

impl From<Row> for Account {
    fn from(row: Row) -> Self {
        (&row).into()
    }
}

impl From<&Row> for Account {
    fn from(row: &Row) -> Self {
        Account {
            id: row.get(0),
            username: row.get(1),
            password: row.get(2),
            nick_name: row.get(3),
        }
    }
}

pub struct QueryAccount;

#[juniper::graphql_object(Context = Context)]
impl QueryAccount {
    fn all(context: &Context) -> FieldResult<Vec<Account>> {
        let mut conn = context.state().db_connection()?;
        let rows = conn.query("SELECT * FROM account", &[])?;
        Ok(rows.iter().map(|row| row.into()).collect())
    }
}

pub struct MutationAccount;

#[juniper::graphql_object(Context = Context)]
impl MutationAccount {
    fn register(context: &Context, username: String, password: String, nick_name: String) -> FieldResult<Account> {
        let mut conn = context.state().db_connection()?;
        if conn.execute(
            "INSERT INTO account (username, password, nick_name) values ($1, $2, $3)",
            &[&username, &password, &nick_name],
        )? == 0 {
            Err("Failed to INSERT INTO account.")?
        }

        let row = conn.query_one(
            "SELECT * FROM account WHERE username = $1",
            &[&username],
        )?;
        if row.is_empty() {
            Err("Failed to SELECT FROM account.")?
        } else {
            Ok(row.into())
        }
    }
}
