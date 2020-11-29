pub(crate) mod adapters;
pub mod configuration;
pub(crate) mod handlers;
mod middleware;
pub(crate) mod repository;
mod startup;
pub(crate) mod state;
pub mod telemetry;

pub use startup::run;
