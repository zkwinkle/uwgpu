use std::fmt;

use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use sqlx::prelude::Type;
use sqlx::{Decode, Encode, Error};

#[derive(Debug, Clone, PartialEq)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(str: String) -> Option<NonEmptyString> {
        if str.is_empty() {
            None
        } else {
            Some(Self(str))
        }
    }
}

impl fmt::Display for NonEmptyString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Type<sqlx::Postgres> for NonEmptyString {
    fn type_info() -> PgTypeInfo {
        <String as Type<sqlx::Postgres>>::type_info()
    }
}

impl<'r> Decode<'r, sqlx::Postgres> for NonEmptyString {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let decoded_string = <String as Decode<sqlx::Postgres>>::decode(value)?;

        let non_empty_string =
            NonEmptyString::new(decoded_string).ok_or_else(|| {
                Error::Decode(
                    format!("Invalid value for NonEmptyString: empty string")
                        .into(),
                )
            })?;

        Ok(non_empty_string)
    }
}

impl<'q> Encode<'q, sqlx::Postgres> for NonEmptyString {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        <String as Encode<sqlx::Postgres>>::encode_by_ref(&self.0, buf)
    }
}
