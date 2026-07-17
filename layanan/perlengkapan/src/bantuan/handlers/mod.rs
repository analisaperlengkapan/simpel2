//! HTTP surface for helpdesk tickets. Routes are registered in
//! [`crate::routes::create_routes`] under `/bantuan/*`.

pub mod ticket;

pub use ticket::*;
