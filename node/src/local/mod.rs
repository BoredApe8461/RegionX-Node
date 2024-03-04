/// Local development service.
mod service;

/// Development chain specs.
mod chain_spec;

pub use chain_spec::*;
pub use service::{start_node, new_partial};
