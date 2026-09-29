//! Official Aevo Rust SDK.
//!
//! This crate is pre-release and unpublished. Its signing module is tested
//! against vectors generated from the Aevo exchange backend.

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod signing;
pub mod ws;

pub use client::{AevoClient, AevoClientBuilder};
pub use config::{AuthMode, Env, Secret};
pub use error::{AevoError, BuilderErrorCode, Result};
