use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    // Decode/encode specific
    #[error("unexpected format. end came before MSB cleared")]
    UnexpectedFormat,
    #[error("unexpected repeat size. got={0}, want={1}")]
    UnexpectedRepeatSize(u128, u128),
    #[error("no expected type value. got={0}")]
    UnexpectedWireDataValue(u128),

    // Parse/type mapping
    #[error("unexpected type. got={got}, want={want}")]
    UnexpectedType { want: String, got: String },

    // Value constraints
    #[error("value too large for {ty}. max={max}")]
    ValueTooLarge { ty: &'static str, max: u128 },

    // Transparent sources
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    IntConversion(#[from] std::num::TryFromIntError),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
}
