use std::{
    fmt::Display,
    str::FromStr,
    num::ParseIntError,
};
use postgres_types::{ToSql, FromSql};
use serde::de::{self, Deserialize, Deserializer};
use uuid::Uuid;

pub mod clause;

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql, juniper::GraphQLEnum)]
#[postgres(name = "permission")]
pub enum Permission {
    #[postgres(name = "none")]
    None,
    #[postgres(name = "read-only")]
    ReadOnly,
    #[postgres(name = "all")]
    All,
}

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
#[postgres(name = "permission_nn")]
pub struct PermissionNN(pub Permission);

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
#[postgres(name = "authority")]
pub enum Authority {
    #[postgres(name = "team_authority")]
    TeamAuthority,
    #[postgres(name = "store_authority")]
    StoreAuthority,
    #[postgres(name = "product_authority")]
    ProductAuthority,
}

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
#[postgres(name = "authority_nn")]
pub struct AuthorityNN(pub Authority);

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
#[postgres(name = "text_nn")]
pub struct TextNN(pub String);

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
#[postgres(name = "text_nz")]
pub struct TextNZ(pub String);

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
#[postgres(name = "int_nn")]
pub struct IntNN(pub i32);

impl FromStr for IntNN {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(IntNN(i32::from_str(s)?))
    }
}

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
#[postgres(name = "uuid_nn")]
pub struct UuidNN(pub Uuid);

impl FromStr for UuidNN {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UuidNN(Uuid::parse_str(s)?))
    }
}

pub fn _from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

#[macro_export]
macro_rules! query {
    (
        $conn:ident,
        $statement:expr,
        $params:expr,
    ) => {
        $conn.query($statement, $params)
        .map_err(|err| -> Error {
            err.into()
        })
    }
}

#[macro_export]
macro_rules! query_one {
    (
        $conn:ident,
        $statement:expr,
        $params:expr,
        ($($column:ident: $type:ty),+),
    ) => {
        $conn.query_one($statement, $params)
        .map(|row| -> ($($type,)+) {
            ($(row.get(stringify!($column)),)+)
        })
        .map_err(|err| -> Error {
            err.into()
        })
    }
}

#[macro_export]
macro_rules! query_opt {
    (
        $conn:ident,
        $statement:expr,
        $params:expr,
    ) => {
        $conn.query_opt($statement, $params)
        .map_err(|err| -> Error {
            err.into()
        })
    }
}

#[cfg(test)]
mod test {
    use uuid::Uuid;
    use crate::{
        state::db::init_pool,
        error::Error,
    };
    use crate::PG_CONFIG;

    #[test]
    fn test_query_one() {
        let pool = init_pool(PG_CONFIG, 1);
        let mut conn = pool.get().unwrap();
        let username = "david0608";
        let password = "123123";
        let (id, nick_name) = query_one!(
            conn,
            "SELECT id, nick_name FROM account WEHRE username = $1 AND password = $2",
            &[&username, &password],
            (id: Uuid, nick_name: String),
        ).unwrap();
        println!("id:{}, nickname:{}", id, nick_name)
    }
}