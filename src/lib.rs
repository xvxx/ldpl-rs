#[macro_use]
extern crate pest_derive;

pub mod emitter;
pub mod error;
pub mod parser;

pub use error::LDPLError;
pub type LDPLResult<T> = std::result::Result<T, LDPLError>;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PLATFORM: &str = env!("PLATFORM");
pub const GIT_REF: &str = env!("GIT_REF");
pub const BUILD_DATE: &str = env!("BUILD_DATE");
