pub mod document;
pub mod provider;

pub use document::Document;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
