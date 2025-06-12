pub mod decode;
pub mod encode;
pub mod error;
pub mod parser;
pub mod wire;
mod zigzag;

#[cfg(test)]
mod error_test;

pub use error::{ProtowiresError, Result};
pub use protowirers_impl::*;
