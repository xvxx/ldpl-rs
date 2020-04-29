#[macro_use]
extern crate pest_derive;

#[macro_use]
pub mod error;
pub mod builder;
pub mod compiler;
pub mod parser;
mod types;

pub use error::LDPLError;
pub use types::LDPLType;
pub type LDPLResult<T> = std::result::Result<T, LDPLError>;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PLATFORM: &str = env!("PLATFORM");
pub const GIT_REF: &str = env!("GIT_REF");
pub const BUILD_DATE: &str = env!("BUILD_DATE");
