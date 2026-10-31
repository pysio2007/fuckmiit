use serde::Deserialize;

use super::{Client, nullable};
use crate::{Error, Result};

const AUTH_SECRET: &str = "testtest";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AuthResponse {
    #[serde(deserialize_with = "nullable")]
    pub code: i64,
    #[serde(deserialize_with = "nullable")]
    pub msg: String,
    #[serde(deserialize_with = "nullable")]
    pub params: AuthParams,
    #[serde(deserialize_with = "nullable")]
    pub success: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AuthParams {
    #[serde(deserialize_with = "nullable")]
    pub expire: i64,
    #[serde(deserialize_with = "nullable")]
    pub refresh: String,
    /// Upstream misspells this field as `bussiness`.
    #[serde(rename = "bussiness", deserialize_with = "nullable")]
    pub business: String,
}

impl Client {
    /// Obtains a session token and installs it as the `Token` header.
    pub fn auth(&mut self, timestamp: u64) -> Result<AuthResponse> {
        let auth_key = build_auth_key(timestamp);
        let timestamp = timestamp.to_string();
        let response: AuthResponse = self.post_form_json(
            "auth",
            &[
                ("authKey", auth_key.as_str()),
                ("timeStamp", timestamp.as_str()),
            ],
        )?;
        if response.code != 200 || !response.success {
            return Err(Error::Rejected {
                endpoint: "auth request",
                code: response.code,
                msg: response.msg,
            });
        }

        self.set_token(&response.params.business);
        Ok(response)
    }
}

/// `md5_hex(secret ++ timestamp)`.
pub fn build_auth_key(timestamp: u64) -> String {
    format!("{:x}", md5::compute(format!("{AUTH_SECRET}{timestamp}")))
}
