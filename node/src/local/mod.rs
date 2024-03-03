/// Local development service.
mod service;

/// Development chain specs.
mod chain_spec;

pub use chain_spec::*;
pub use service::{new_full, new_partial};
