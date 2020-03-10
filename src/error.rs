#[derive(Debug)]
pub enum Error {
    R2D2(r2d2::Error),
    Postgres(postgres::error::Error),
    Uuid(uuid::Error),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::R2D2(err) => write!(f, "R2D2_error : {}", err),
            Error::Postgres(err) => write!(f, "Postgres_error : {}", err),
            Error::Uuid(err) => write!(f, "Uuid_error : {}", err),
            Error::Other(err) => write!(f, "Other_error : {}", err),
        }
    }
}

macro_rules! impl_from_for_error {
    ($E:ident, $F:path) => {
        impl From<$F> for Error {
            fn from(err: $F) -> Self {
                Error::$E(err)
            }
        }
    }
}

impl_from_for_error!(R2D2, r2d2::Error);
impl_from_for_error!(Postgres, postgres::error::Error);
impl_from_for_error!(Uuid, uuid::Error);
impl_from_for_error!(Other, String);
