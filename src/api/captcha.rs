use std::fmt::Write;

use serde::{Deserialize, Serialize};

use super::{Client, nullable};
use crate::{Error, Result};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCheckImagePointRequest<'a> {
    pub client_uid: &'a str,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct GetCheckImagePointResponse {
    #[serde(deserialize_with = "nullable")]
    pub code: i64,
    #[serde(deserialize_with = "nullable")]
    pub msg: String,
    #[serde(deserialize_with = "nullable")]
    pub params: GetCheckImageParams,
    #[serde(deserialize_with = "nullable")]
    pub success: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GetCheckImageParams {
    #[serde(deserialize_with = "nullable")]
    pub small_image: String,
    #[serde(deserialize_with = "nullable")]
    pub big_image: String,
    #[serde(deserialize_with = "nullable")]
    pub uuid: String,
    #[serde(deserialize_with = "nullable")]
    pub height: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckImageRequest<'a> {
    pub key: &'a str,
    pub value: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CheckImageResponse {
    #[serde(deserialize_with = "nullable")]
    pub code: i64,
    #[serde(deserialize_with = "nullable")]
    pub msg: String,
    /// The sign to send as the `Sign` header on success.
    #[serde(deserialize_with = "nullable")]
    pub params: String,
    #[serde(deserialize_with = "nullable")]
    pub success: bool,
}

impl Client {
    pub fn get_check_image_point(&self, client_uid: &str) -> Result<GetCheckImagePointResponse> {
        let response: GetCheckImagePointResponse = self.post_json(
            "image/getCheckImagePoint",
            &GetCheckImagePointRequest { client_uid },
        )?;
        if response.code != 200 || !response.success {
            return Err(Error::Rejected {
                endpoint: "getCheckImagePoint",
                code: response.code,
                msg: response.msg,
            });
        }
        Ok(response)
    }

    /// Like [`Client::try_check_image`], but a rejected answer is an error.
    pub fn check_image(&self, key: &str, value: i32) -> Result<CheckImageResponse> {
        let response = self.try_check_image(key, value)?;
        if response.code != 200 || !response.success {
            return Err(Error::Rejected {
                endpoint: "checkImage",
                code: response.code,
                msg: response.msg,
            });
        }
        Ok(response)
    }

    /// Submits a slider offset; a rejected answer is returned, not an error.
    pub fn try_check_image(&self, key: &str, value: i32) -> Result<CheckImageResponse> {
        self.post_json(
            "image/checkImage",
            &CheckImageRequest {
                key,
                value: value.to_string(),
            },
        )
    }

    pub fn set_sign(&mut self, sign: &str) {
        self.set_header("Sign", sign);
    }

    pub fn set_uuid(&mut self, uuid: &str) {
        self.set_header("Uuid", uuid);
    }
}

/// `point-` followed by a random v4 UUID.
pub fn new_client_uid() -> Result<String> {
    let mut raw = [0u8; 16];
    getrandom::fill(&mut raw).map_err(Error::Random)?;
    raw[6] = (raw[6] & 0x0f) | 0x40;
    raw[8] = (raw[8] & 0x3f) | 0x80;

    let mut uid = String::with_capacity(42);
    uid.push_str("point-");
    for (i, byte) in raw.iter().enumerate() {
        if matches!(i, 4 | 6 | 8 | 10) {
            uid.push('-');
        }
        write!(uid, "{byte:02x}").expect("writing to a String cannot fail");
    }
    Ok(uid)
}
