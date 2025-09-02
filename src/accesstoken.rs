use crate::*;

#[derive(Clone, Debug, serde::Deserialize)]
struct RequestTokenRes {
    expires_in: String,
    access_token: String,
}

#[derive(Clone, Debug)]
pub struct AccessToken {
    expires_on: time::OffsetDateTime,
    token: String,
}

impl AccessToken {
    fn create(res: RequestTokenRes) -> Self {
        let expires_in = res.expires_in.parse::<i64>().unwrap_or(60);
        let expires_on = time::OffsetDateTime::now_utc() + time::Duration::seconds(expires_in);

        Self {
            expires_on,
            token: res.access_token,
        }
    }

    pub fn is_valid(&self) -> bool {
        // We count token as invalid if it expires in less than 10 minutes
        let now = time::OffsetDateTime::now_utc() - time::Duration::minutes(10);
        self.expires_on > now
    }

    pub fn token(&self) -> &str {
        &self.token
    }
}

impl VippsApi {
    fn client_secret_header(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();

        headers.insert(
            "client_id",
            (&self.0.auth_info.client_id).try_into().unwrap(),
        );
        headers.insert(
            "client_secret",
            (&self.0.auth_info.client_secret).try_into().unwrap(),
        );

        headers
    }

    #[tracing::instrument(skip_all, err)]
    async fn request_access_token(&self) -> Result<AccessToken> {
        let res = self
            .0
            .client
            .post(format!("{}/accesstoken/get", self.0.base_url))
            .headers(self.client_secret_header())
            .header("content-length", 0)
            .body("")
            .send()
            .await?
            .error_for_status()?
            .json::<RequestTokenRes>()
            .await?;

        tracing::debug!(token = res.access_token, "got new access token");

        Ok(AccessToken::create(res))
    }

    #[tracing::instrument(skip_all, level = tracing::Level::DEBUG)]
    pub async fn access_token(&self) -> Result<AccessToken> {
        if self
            .0
            .current_token
            .read()
            .unwrap()
            .as_ref()
            .map(|token| !token.is_valid())
            .unwrap_or(true)
        {
            tracing::trace!("requesting a new access token");
            *self.0.current_token.write().unwrap() = Some(self.request_access_token().await?);
        } else {
            tracing::trace!("reusing previous access token");
        }

        Ok(self.0.current_token.read().unwrap().clone().unwrap())
    }
}
