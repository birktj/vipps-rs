use crate::*;

pub struct Qr {
    vipps: VippsApi,
    data: QrRes,
}

impl VippsApi {
    #[tracing::instrument(skip(self), err)]
    pub async fn create_redirect_qr(&self, id: &str, uri: &str) -> Result<Qr> {
        let res = self
            .0
            .client
            .post(format!("{}/qr/v1/merchant-redirect", self.0.base_url))
            .bearer_auth(self.access_token().await?.token())
            .header("accept", "image/svg+xml")
            .json(&CreateMerchantRedirectReq {
                id: id.to_string(),
                redirect_url: uri.to_string(),
            })
            .send()
            .await?
            .into_vipps_result()
            .await?
            .json::<QrRes>()
            .await?;

        tracing::debug!("created a redeirect qr");

        Ok(Qr {
            vipps: self.clone(),
            data: res,
        })
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn get_redirect_qr(&self, id: &str) -> Result<Option<Qr>> {
        let res = self
            .0
            .client
            .get(format!(
                "{}/qr/v1/merchant-redirect/{}",
                self.0.base_url, id
            ))
            .bearer_auth(self.access_token().await?.token())
            .header("accept", "image/svg+xml")
            .send()
            .await?;

        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let data = res.into_vipps_result().await?.json::<QrRes>().await?;

        tracing::debug!("found redirect qr");

        Ok(Some(Qr {
            vipps: self.clone(),
            data,
        }))
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn list_redirect_qrs(&self) -> Result<Vec<Qr>> {
        let res = self
            .0
            .client
            .get(format!("{}/qr/v1/merchant-redirect", self.0.base_url))
            .bearer_auth(self.access_token().await?.token())
            .header("accept", "image/svg+xml")
            .send()
            .await?
            .into_vipps_result()
            .await?
            .json::<Vec<QrRes>>()
            .await?;

        tracing::debug!("listed redirect qrs");

        Ok(res
            .into_iter()
            .map(|data| Qr {
                vipps: self.clone(),
                data,
            })
            .collect())
    }
}

impl Qr {
    pub fn id(&self) -> &str {
        &self.data.id
    }

    pub fn url(&self) -> &str {
        &self.data.url
    }

    pub fn redirect_url(&self) -> &str {
        &self.data.redirect_url
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn update_redirect_url(&mut self, url: &str) -> Result<()> {
        let res = self
            .vipps
            .0
            .client
            .put(format!(
                "{}/qr/v1/merchant-redirect/{}",
                self.vipps.0.base_url, &self.data.id
            ))
            .bearer_auth(self.vipps.access_token().await?.token())
            .header("accept", "image/svg+xml")
            .json(&UpdateUrlReq {
                redirect_url: url.to_string(),
            })
            .send()
            .await?
            .into_vipps_result()
            .await?
            .json::<QrRes>()
            .await?;

        self.data = res;

        tracing::debug!("updated vipps qr");

        Ok(())
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn delete(self) -> Result<()> {
        self.vipps
            .0
            .client
            .delete(format!(
                "{}/qr/v1/merchant-redirect/{}",
                self.vipps.0.base_url, &self.data.id
            ))
            .bearer_auth(self.vipps.access_token().await?.token())
            .send()
            .await?
            .into_vipps_result()
            .await?;

        tracing::debug!("deleted vipps qr");

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateMerchantRedirectReq {
    id: String,
    redirect_url: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateUrlReq {
    redirect_url: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct QrRes {
    id: String,
    url: String,
    redirect_url: String,
}
