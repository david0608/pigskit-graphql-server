use postgres::NoTls;
use r2d2_postgres::PostgresConnectionManager;

pub type Connection = r2d2::PooledConnection<PostgresConnectionManager<NoTls>>;
pub type Pool = r2d2::Pool<PostgresConnectionManager<NoTls>>;

pub fn init_pool(config: &str, size: u32) -> Pool {
    let manager = PostgresConnectionManager::new(
        config.parse().unwrap(),
        NoTls,
    );
    Pool::builder()
        .max_size(size)
        .build(manager)
        .expect("Init sync pool.")
}
