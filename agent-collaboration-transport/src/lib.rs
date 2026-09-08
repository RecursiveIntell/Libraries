#![deny(unsafe_code)]

pub mod frame;
pub mod identity;
pub mod replay;
pub mod transport;

pub use frame::{read_frame, write_frame, FrameError, FrameV1, MAX_FRAME_BYTES, PROTOCOL_VERSION};
pub use identity::{IdentityError, StaticIdentity};
pub use replay::ReplayCache;
pub use transport::{receive_authenticated_frame, send_authenticated_frame};
