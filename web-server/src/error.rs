use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub enum ServerError {
    DatabaseError(sqlx::Error),
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::DatabaseError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database Error: {}", error),
            ),
        }
        .into_response()
    }
}

impl From<sqlx::Error> for ServerError {
    fn from(error: sqlx::Error) -> Self { Self::DatabaseError(error) }
}
