#![allow(dead_code)]

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("http error")]
    HttpError(#[from] reqwest::Error),
    #[error("api error {code} ({title:?})")]
    ApiError {
        code: u16,
        title: String,
        detail: String,
    },
    #[cfg(feature = "mock")]
    #[error("mock error")]
    Mock,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProblemDetails {
    // status: u16,
    #[serde(rename = "type")]
    ty: Option<String>,
    title: String,
    detail: String,
    instance: String,
    extra_details: Option<Vec<InvalidParam>>,
    invalid_params: Option<Vec<InvalidParam>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct InvalidParam {
    name: String,
    reason: String,
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) trait VippsResponse: Sized {
    async fn into_vipps_result(self) -> Result<Self>;
}

impl VippsResponse for reqwest::Response {
    async fn into_vipps_result(self) -> Result<Self> {
        match self.status() {
            code if code.is_success() => Ok(self),
            code => {
                if let Ok(err) = self.json::<ProblemDetails>().await {
                    tracing::debug!(details = ?err, "Error details");
                    Err(Error::ApiError {
                        code: code.as_u16(),
                        title: err.title,
                        detail: err.detail,
                    })
                } else {
                    Err(Error::ApiError {
                        code: code.as_u16(),
                        title: "Unknown error".to_string(),
                        detail: String::new(),
                    })
                }
            }
        }
    }
}
