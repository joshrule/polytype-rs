mod context;
mod parser;
mod substitution;
mod types;

pub use context::{with_ctx, Context, TypeContext};
pub use substitution::{Snapshot, Substitution};
pub use types::{Schema, Ty, Type, TypeSchema, UnificationError, Variable};

// TODO: write tp and ptp macros for atype.
