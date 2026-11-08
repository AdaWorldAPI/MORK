#![cfg_attr(feature = "nightly", feature(coroutine_trait, coroutines, stmt_expr_attributes))]

pub mod space;
/// The worst-case-optimal leapfrog join. Compiled only under the `leapfrog` feature, which is
/// also what routes conjunctive bodies to it; without the feature the engine is unchanged.
#[cfg(feature = "leapfrog")]
pub mod leapfrog;
mod sources;
mod sinks;
mod pure;

pub use sinks::WriteResourceRequest;
pub use sources::ResourceRequest;
