//! See [NonEmptyString]

/// Represents the storing of a non-empty string in the Database.
///
/// This is used to disambiguate the meaning of [Option<NonEmptyString>].
///
/// This way if it's [Some] you don't have to check if the string is empty or
/// not.
#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(transparent)]
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
