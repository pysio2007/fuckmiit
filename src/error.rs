pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("domain is required")]
    EmptyDomain,
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error("request failed: status={status} body={body}")]
    Status { status: u16, body: String },
    #[error("decode response failed: {source} body={body}")]
    Decode {
        source: serde_json::Error,
        body: String,
    },
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("{endpoint} rejected: code={code} msg={msg}")]
    Rejected {
        endpoint: &'static str,
        code: i64,
        msg: String,
    },
    #[error("random source failed: {0}")]
    Random(getrandom::Error),
    #[error("decode base64 image failed: unsupported base64 image data")]
    Base64,
    #[error("decode image failed: {0}")]
    Image(#[from] image::ImageError),
    #[error("square not found")]
    SquareNotFound,
    #[error("checkImage failed around detected left={left}")]
    CaptchaFailed { left: i32 },
    #[error("queryByCondition returned no records for {0}")]
    NoRecords(String),
}
