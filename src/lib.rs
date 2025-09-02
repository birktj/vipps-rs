mod accesstoken;
mod basic;
pub mod epayment;
mod error;
pub mod order_management;
mod qr;

#[cfg(feature = "mock")]
mod mock;

use std::sync::Arc;

pub use basic::*;
pub use error::*;

#[derive(Clone, Debug)]
pub struct SystemInfo {
    pub system_name: String,
    pub system_version: String,
    pub system_plugin_name: Option<String>,
    pub system_plugin_version: Option<String>,
}

#[derive(Clone, Debug)]
pub struct MerchantInfo {
    pub subscription_key: String,
    pub msn: String,
}

#[derive(Clone, Debug)]
pub struct AuthInfo {
    pub client_id: String,
    pub client_secret: String,
}

pub(crate) struct VippsApiData {
    system_info: SystemInfo,
    merchant_info: MerchantInfo,
    auth_info: AuthInfo,
    client: reqwest::Client,
    base_url: String,
    current_token: std::sync::RwLock<Option<accesstoken::AccessToken>>,
}

#[derive(Clone)]
pub struct VippsApi(Arc<VippsApiData>);

impl VippsApi {
    fn new_inner(
        system_info: SystemInfo,
        merchant_info: MerchantInfo,
        auth_info: AuthInfo,
        base_url: String,
    ) -> Self {
        let mut default_headers = reqwest::header::HeaderMap::new();

        default_headers.insert(
            "Ocp-Apim-Subscription-Key",
            (&merchant_info.subscription_key).try_into().unwrap(),
        );
        default_headers.insert(
            "Merchant-Serial-Number",
            (&merchant_info.msn).try_into().unwrap(),
        );

        default_headers.insert(
            "Vipps-System-Name",
            (&system_info.system_name).try_into().unwrap(),
        );
        default_headers.insert(
            "Vipps-System-Version",
            (&system_info.system_version).try_into().unwrap(),
        );

        if let Some(plugin_name) = system_info.system_plugin_name.as_ref() {
            default_headers.insert("Vipps-System-Plugin-Name", plugin_name.try_into().unwrap());
        }
        if let Some(plugin_version) = system_info.system_plugin_name.as_ref() {
            default_headers.insert(
                "Vipps-System-Plugin-Version",
                plugin_version.try_into().unwrap(),
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(default_headers)
            .build()
            .unwrap();

        Self(Arc::new(VippsApiData {
            system_info,
            merchant_info,
            auth_info,
            client,
            base_url,
            current_token: std::sync::RwLock::new(None),
        }))
    }

    pub fn new(system_info: SystemInfo, merchant_info: MerchantInfo, auth_info: AuthInfo) -> Self {
        Self::new_inner(
            system_info,
            merchant_info,
            auth_info,
            "https://apitest.vipps.no".to_string(),
        )
    }

    pub fn new_production(
        system_info: SystemInfo,
        merchant_info: MerchantInfo,
        auth_info: AuthInfo,
    ) -> Self {
        Self::new_inner(
            system_info,
            merchant_info,
            auth_info,
            "https://api.vipps.no".to_string(),
        )
    }

    fn create_unique_reference(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
