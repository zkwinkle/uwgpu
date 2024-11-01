use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub enum ServerError {
    DatabaseError(sqlx::Error),
    ParsingJson(serde_json::Error),
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::DatabaseError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database Error: {}", error),
            ),
            ServerError::ParsingJson(error) => (
                StatusCode::BAD_REQUEST,
                format!("Error parsing JSON in the request: {}", error),
            ),
        }
        .into_response()
    }
}

impl From<sqlx::Error> for ServerError {
    fn from(error: sqlx::Error) -> Self { Self::DatabaseError(error) }
}

impl From<serde_json::Error> for ServerError {
    fn from(error: serde_json::Error) -> Self { Self::ParsingJson(error) }
}
