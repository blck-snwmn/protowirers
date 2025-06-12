use thiserror::Error;

/// 統一エラー型
#[derive(Error, Debug)]
pub enum ProtowiresError {
    /// デコードエラー
    #[error("decode error: {0}")]
    Decode(#[from] crate::decode::DecodeError),

    /// パースエラー
    #[error("parse error: {0}")]
    Parse(#[from] crate::parser::ParseError),

    /// IOエラー
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 数値変換エラー
    #[error("numeric conversion error: {0}")]
    TryFromInt(#[from] std::num::TryFromIntError),

    /// 値が大きすぎるエラー
    #[error("value too large: {value} exceeds maximum {max}")]
    ValueTooLarge { value: u128, max: u128 },

    /// UTF-8変換エラー
    #[error("UTF-8 conversion error: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
}

/// Result型のエイリアス
pub type Result<T> = std::result::Result<T, ProtowiresError>;
