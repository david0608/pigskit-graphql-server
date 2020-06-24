use juniper::{graphql_value, IntoFieldError, FieldError};
use serde_json::json;

#[derive(Debug)]
enum InnerError {
    Sql(postgres::error::Error),
    R2d2(r2d2::Error),
    SerdeJson(serde_json::error::Error),
}

#[derive(Debug)]
pub struct Error {
    r#type: String,
    message: String,
    inner: Option<InnerError>,
}

impl Error {
    pub fn new(r#type: String, message: String) -> Self {
        Error {
            r#type: r#type,
            message: message,
            inner: None
        }
    }

    pub fn permission_denied() -> Self {
        Self::new(
            "Unauthorized".to_string(),
            "Permission denied".to_string(),
        )
    }

    pub fn no_valid_cookie(name: &str) -> Self {
        Self::new(
            "NoValidCookie".to_string(),
            format!(r#"Missing or invalid cookie "{}" in request."#, name),
        )
    }
}

impl IntoFieldError for Error {
    fn into_field_error(self) -> FieldError {
        let error_type = self.r#type;
        FieldError::new(
            self.message,
            graphql_value!({
                "type": error_type
            }),
        )
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = json!({
            "type": &self.r#type,
            "message": &self.message,
        });
        write!(f, "{}", error.to_string())
    }
}

macro_rules! impl_from_for_error {
    ($from:ty, $inner:ident) => {
        impl From<$from> for InnerError {
            fn from(err: $from) -> Self {
                InnerError::$inner(err)
            }
        }

        impl From<$from> for Error {
            fn from(err: $from) -> Self {
                Error {
                    r#type: "InternalServerError".to_string(),
                    message: "Internal server error.".to_string(),
                    inner: Some(err.into()),
                }
            }
        }
    }
}

impl_from_for_error!(r2d2::Error, R2d2);
impl_from_for_error!(serde_json::error::Error, SerdeJson);

impl From<postgres::error::Error> for InnerError {
    fn from(err: postgres::error::Error) -> Self {
        InnerError::Sql(err)
    }
}

impl From<postgres::error::Error> for Error {
    fn from(err: postgres::error::Error) -> Self {
        let code = if let Some(state) = err.code() {
            state.code()
        } else {
            ""
        };

        match code {
            "C2002" => {
                Error {
                    r#type: "SessionExpired".to_string(),
                    message: "Session expired".to_string(),
                    inner: Some(err.into()),
                }
            }
            _ => {
                Error {
                    r#type: "InternalServerError".to_string(),
                    message: "Internal server error.".to_string(),
                    inner: Some(err.into()),
                }
            }
        }
    }
}