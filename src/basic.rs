#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Amount {
    currency: Currency,
    value: i64,
}

impl Amount {
    pub fn nok(value: i64) -> Self {
        Self {
            currency: Currency::Nok,
            value,
        }
    }

    pub fn value(&self) -> i64 {
        self.value
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Currency {
    Nok,
    Dkk,
    Eur,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(untagged)]
pub enum Customer {
    #[serde(rename_all = "camelCase")]
    PhoneNumber { phone_number: String },
    #[serde(rename_all = "camelCase")]
    QrCode { personal_qr: String },
    #[serde(rename_all = "camelCase")]
    CustomerToken { customer_token: String },
}

impl Customer {
    pub fn phone_number(phone_number: String) -> Self {
        Self::PhoneNumber { phone_number }
    }

    pub fn qr_code(personal_qr: String) -> Self {
        Self::QrCode { personal_qr }
    }

    pub fn customer_token(customer_token: String) -> Self {
        Self::CustomerToken { customer_token }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PaymentMethod {
    #[serde(rename = "type")]
    pub(crate) ty: PaymentMethodType,
}

impl PaymentMethod {
    pub fn card() -> Self {
        Self {
            ty: PaymentMethodType::Card,
        }
    }

    pub fn wallet() -> Self {
        Self {
            ty: PaymentMethodType::Wallet,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentMethodType {
    Wallet,
    Card,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserFlow {
    PushMessage,
    NativeRedirect,
    WebRedirect,
    Qr,
}
