//! Target split skeleton for `verification-control`.
//!
//! Move the current monolith definitions into the modules below.
//! Keep the split pass semantic-no-op.

// pub use case::{...};
// pub use check_plan::{...};
// pub use attempt::{...};
// pub use receipt::{...};
// pub use repair::{...};
// pub use ledger::{...};
// pub use scheduler::{...};

pub mod case;
pub mod check_plan;
pub mod attempt;
pub mod receipt;
pub mod repair;
pub mod ledger;
pub mod scheduler;
