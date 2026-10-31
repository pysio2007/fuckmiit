//! Low-level client for the MIIT ICP query HTTP API.

mod auth;
mod captcha;
mod client;
mod icp;

use serde::{Deserialize, Deserializer};

pub use auth::*;
pub use captcha::*;
pub use client::*;
pub use icp::*;

/// Decodes JSON `null` as `T::default()`, matching the server's loose typing.
fn nullable<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}
