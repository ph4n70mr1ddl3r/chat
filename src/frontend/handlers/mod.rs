//! Frontend event handlers
//! 
//! Domain-organized handlers for managing client-side state updates.
//! Each handler corresponds to a specific domain (delivery, connection, presence, etc.)

pub mod delivery_handlers;
pub mod connection_handlers;

pub use delivery_handlers::*;
pub use connection_handlers::*;
