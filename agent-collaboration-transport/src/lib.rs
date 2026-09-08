#![deny(unsafe_code)]

pub mod client;
pub mod frame;
pub mod identity;
pub mod replay;
pub mod server;
pub mod transport;

pub use client::connect_and_send;
pub use frame::{read_frame, write_frame, FrameError, FrameV1, MAX_FRAME_BYTES, PROTOCOL_VERSION};
pub use identity::{IdentityError, StaticIdentity};
pub use replay::ReplayCache;
pub use server::accept_one;
pub use transport::{receive_authenticated_frame, send_authenticated_frame, TransportError};
