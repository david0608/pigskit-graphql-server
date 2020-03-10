pub type _Params<'a> = &'a[&'a(dyn postgres::types::ToSql + std::marker::Sync)];

#[macro_export]
macro_rules! query_one {
    (
        $pool:ident,
        $statement:expr,
        $params:expr,
        ($($column:ident: $type:ident),+),
    ) => {{
        let res: Result<($($type,)+), Error> = (|| {
            let mut conn = $pool.get()?;
            let row = conn.query_one($statement, $params)?;
            Ok(($(row.get(stringify!($column)),)+))
        })();
        res
    }}
}

#[macro_export]
macro_rules! execute {
    (
        $pool:ident,
        $statement:expr,
        $params:expr,
    ) => {{
        let res: Result<usize, Error> = (|| {
            let mut conn = $pool.get()?;
            let n = conn.execute($statement, $params)?;
            Ok(n as usize)
        })();
        res
    }}
}

#[cfg(test)]
mod test {
    use uuid::Uuid;
    use crate::error::Error;
    use crate::state::db::init_pool;
    use crate::PG_CONFIG;

    #[test]
    fn test_query_one() {
        let pool = init_pool(PG_CONFIG, 1);
        let username = "david0608";
        let password = "123123";
        let (id, nick_name) = query_one!(
            pool,
            "SELECT id, nick_name FROM account WHERE username = $1 AND password = $2",
            &[&username, &password],
            (id: Uuid, nick_name: String),
        ).unwrap();
    }

    #[test]
    fn test_execute() {
        let pool = init_pool(PG_CONFIG, 1);
        let username = "david0608";
        let password = "123123";
        let nick_name = "DavidWu";
        let n = execute!(
            pool,
            "INSERT INTO account (username, password, nick_name) VALUES ($1, $2, $3)",
            &[&username, &password, &nick_name],
        ).unwrap();
    }
}