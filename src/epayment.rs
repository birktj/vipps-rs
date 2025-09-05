#![allow(dead_code)]

use crate::*;

/// # Epayment api
impl VippsApi {
    pub fn create_payment(&self) -> CreatePaymentBuilder {
        let req = CreatePaymentReq {
            amount: Amount::nok(0),
            customer: None,
            customer_interaction: CustomerInteraction::CustomerNotPresent,
            payment_method: PaymentMethod::wallet(),
            profile: None,
            reference: PaymentReference(self.create_unique_reference()),
            return_url: None,
            user_flow: UserFlow::WebRedirect,
            payment_description: None,
        };

        CreatePaymentBuilder { api: &self, req }
    }

    #[cfg(not(feature = "mock"))]
    #[tracing::instrument(skip_all, fields(reference = reference.as_str()), err)]
    pub async fn payment(&self, reference: PaymentReference) -> Result<Payment> {
        let data = self
            .0
            .client
            .get(format!(
                "{}/epayment/v1/payments/{}",
                self.0.base_url, reference.0
            ))
            .bearer_auth(self.access_token().await?.token())
            .send()
            .await?
            .into_vipps_result()
            .await?
            .json::<GetPaymentRes>()
            .await?;

        Ok(Payment {
            api: self.clone(),
            reference,
            data,
        })
    }
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CreatePaymentReq {
    amount: Amount,
    customer: Option<Customer>,
    customer_interaction: CustomerInteraction,
    payment_method: PaymentMethod,
    profile: Option<ProfileScope>,
    reference: PaymentReference,
    return_url: Option<String>,
    user_flow: UserFlow,
    payment_description: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CustomerInteraction {
    CustomerNotPresent,
    CustomerPresent,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfileScope {
    scope: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePaymentRes {
    redirect_url: Option<String>,
    reference: PaymentReference,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PaymentReference(pub(crate) String);

impl std::fmt::Display for PaymentReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for PaymentReference {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl PaymentReference {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub struct CreatePaymentBuilder<'a> {
    api: &'a VippsApi,
    req: CreatePaymentReq,
}

impl<'a> CreatePaymentBuilder<'a> {
    #[cfg(not(feature = "mock"))]
    #[tracing::instrument(skip(self), err)]
    pub async fn send(self) -> Result<Payment> {
        let idempotency_key = self.api.create_unique_reference();

        // TODO: retry system

        let res = self
            .api
            .0
            .client
            .post(format!("{}/epayment/v1/payments", self.api.0.base_url))
            .header("Idempotency-Key", &idempotency_key)
            .bearer_auth(self.api.access_token().await?.token())
            .json(&self.req)
            .send()
            .await?
            .into_vipps_result()
            .await?
            .json::<CreatePaymentRes>()
            .await?;

        tracing::debug!(reference = res.reference.as_str(), "payment created");

        return Ok(Payment {
            api: self.api.clone(),
            reference: res.reference.clone(),
            data: GetPaymentRes {
                amount: self.req.amount,
                state: PaymentState::Created,
                payment_method: PaymentMethodResponse {
                    ty: self.req.payment_method.ty,
                    card_bin: None,
                },
                profile: ProfileSub { sub: None },
                redirect_url: res.redirect_url,
                reference: res.reference,
            },
        });
    }

    pub fn reference(&self) -> PaymentReference {
        self.req.reference.clone()
    }

    pub fn set_amount(&mut self, amount: Amount) {
        self.req.amount = amount;
    }

    pub fn amount(mut self, amount: Amount) -> Self {
        self.set_amount(amount);
        self
    }

    pub fn set_customer(&mut self, customer: Customer) {
        self.req.customer = Some(customer);
    }

    pub fn customer(mut self, customer: Customer) -> Self {
        self.set_customer(customer);
        self
    }

    pub fn set_customer_interaction(&mut self, customer_interaction: CustomerInteraction) {
        self.req.customer_interaction = customer_interaction;
    }

    pub fn customer_interaction(mut self, customer_interaction: CustomerInteraction) -> Self {
        self.set_customer_interaction(customer_interaction);
        self
    }

    pub fn set_scope(&mut self, scope: String) {
        self.req.profile = Some(ProfileScope { scope })
    }

    pub fn scope(mut self, scope: String) -> Self {
        self.set_scope(scope);
        self
    }

    pub fn set_return_url(&mut self, return_url: String) {
        self.req.return_url = Some(return_url);
    }

    pub fn return_url(mut self, return_url: String) -> Self {
        self.set_return_url(return_url);
        self
    }

    pub fn set_user_flow(&mut self, user_flow: UserFlow) {
        self.req.user_flow = user_flow;
    }

    pub fn user_flow(mut self, user_flow: UserFlow) -> Self {
        self.set_user_flow(user_flow);
        self
    }

    pub fn set_payment_description(&mut self, payment_description: String) {
        self.req.payment_description = Some(payment_description);
    }

    pub fn payment_description(mut self, payment_description: String) -> Self {
        self.set_payment_description(payment_description);
        self
    }

    pub fn set_payment_method(&mut self, payment_method: PaymentMethodType) {
        self.req.payment_method.ty = payment_method;
    }

    pub fn payment_method(mut self, payment_method: PaymentMethodType) -> Self {
        self.set_payment_method(payment_method);
        self
    }
}

#[derive(Clone)]
pub struct Payment {
    pub(crate) api: VippsApi,
    pub(crate) reference: PaymentReference,
    pub(crate) data: GetPaymentRes,
}

impl Payment {
    pub fn reference(&self) -> PaymentReference {
        self.reference.clone()
    }

    pub fn redirect_uri(&self) -> Option<&str> {
        self.data.redirect_url.as_deref()
    }

    pub fn sub(&self) -> Option<&str> {
        self.data.profile.sub.as_deref()
    }

    pub fn amount(&self) -> Amount {
        self.data.amount.clone()
    }

    pub fn state(&self) -> PaymentState {
        self.data.state.clone()
    }

    #[cfg(not(feature = "mock"))]
    #[tracing::instrument(skip_all, fields(reference = self.reference().as_str()), err)]
    pub async fn cancel(&mut self) -> Result<()> {
        let idempotency_key = self.api.create_unique_reference();
        let res = self
            .api
            .0
            .client
            .post(format!(
                "{}/epayment/v1/payments/{}/cancel",
                self.api.0.base_url, self.reference.0
            ))
            .header("Idempotency-Key", &idempotency_key)
            .bearer_auth(self.api.access_token().await?.token())
            .header("Content-Length", 0)
            .send()
            .await?
            .into_vipps_result()
            .await?
            .json::<AdjustmentRes>()
            .await?;

        tracing::debug!("canceled payment");

        self.data.update(&res);

        Ok(())
    }

    #[cfg(not(feature = "mock"))]
    #[tracing::instrument(skip_all, fields(reference = self.reference().as_str()), err)]
    pub async fn capture(&mut self, amount: Amount) -> Result<()> {
        // TODO: retry?
        let idempotency_key = self.api.create_unique_reference();
        let res = self
            .api
            .0
            .client
            .post(format!(
                "{}/epayment/v1/payments/{}/capture",
                self.api.0.base_url, self.reference.0
            ))
            .header("Idempotency-Key", &idempotency_key)
            .bearer_auth(self.api.access_token().await?.token())
            .json(&ModificationReq {
                modification_amount: amount,
            })
            .send()
            .await?
            .into_vipps_result()
            .await?
            .json::<AdjustmentRes>()
            .await?;

        tracing::debug!("captured payment");

        self.data.update(&res);

        Ok(())
    }

    #[cfg(not(feature = "mock"))]
    #[tracing::instrument(skip_all, fields(reference = self.reference().as_str()), err)]
    pub async fn refund(&mut self, amount: Amount) -> Result<()> {
        // TODO: retry?
        let idempotency_key = self.api.create_unique_reference();
        let res = self
            .api
            .0
            .client
            .post(format!(
                "{}/epayment/v1/payments/{}/refund",
                self.api.0.base_url, self.reference.0
            ))
            .header("Idempotency-Key", &idempotency_key)
            .bearer_auth(self.api.access_token().await?.token())
            .json(&ModificationReq {
                modification_amount: amount,
            })
            .send()
            .await?
            .into_vipps_result()
            .await?
            .json::<AdjustmentRes>()
            .await?;

        tracing::debug!("refunded payment");

        self.data.update(&res);

        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug" fields(reference = self.reference().as_str()), err)]
    pub async fn update(&mut self) -> Result<()> {
        let payment = self.api.payment(self.reference.clone()).await?;
        self.data = payment.data;

        tracing::debug!("updated payment data");

        Ok(())
    }
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ModificationReq {
    modification_amount: Amount,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetPaymentRes {
    amount: Amount,
    state: PaymentState,
    // aggregate: PayymentAggregate,
    payment_method: PaymentMethodResponse,
    profile: ProfileSub,
    // psp_reference: String,
    redirect_url: Option<String>,
    reference: PaymentReference,
}

impl GetPaymentRes {
    fn update(&mut self, adjustment: &AdjustmentRes) {
        self.amount = adjustment.amount.clone();
        self.state = adjustment.state.clone();
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdjustmentRes {
    amount: Amount,
    state: PaymentState,
    // aggregate: PayymentAggregate,
    psp_reference: String,
    reference: PaymentReference,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PaymentMethodResponse {
    #[serde(rename = "type")]
    ty: PaymentMethodType,
    card_bin: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfileSub {
    sub: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentState {
    Created,
    Aborted,
    Expired,
    Authorized,
    Terminated,
}

impl PaymentState {
    pub fn completed(&self) -> bool {
        !matches!(self, PaymentState::Created)
    }
}

#[cfg(feature = "mock")]
pub(crate) mod mock {
    use super::*;

    use std::collections::HashMap;
    use std::sync::{LazyLock, Mutex};

    struct MockPaymentData {
        pay_data: GetPaymentRes,
        return_url: Option<String>,
    }

    struct MockPaymentDb {
        pub db: Mutex<HashMap<PaymentReference, MockPaymentData>>,
    }

    static MOCK_DB: LazyLock<MockPaymentDb> = LazyLock::new(|| MockPaymentDb {
        db: Mutex::new(HashMap::new()),
    });

    impl<'a> CreatePaymentBuilder<'a> {
        #[cfg(feature = "mock")]
        pub async fn send(self) -> Result<Payment> {
            let redirect_url = Some(format!("/mock/vipps/payment/{}", self.req.reference.0));
            let payment = Payment {
                api: self.api.clone(),
                reference: self.req.reference.clone(),
                data: GetPaymentRes {
                    amount: self.req.amount,
                    state: PaymentState::Created,
                    payment_method: PaymentMethodResponse {
                        ty: self.req.payment_method.ty,
                        card_bin: None,
                    },
                    profile: ProfileSub { sub: None },
                    redirect_url,
                    reference: self.req.reference.clone(),
                },
            };

            mock::MOCK_DB.db.lock().unwrap().insert(
                payment.reference.clone(),
                MockPaymentData {
                    pay_data: payment.data.clone(),
                    return_url: self.req.return_url,
                },
            );

            Ok(payment)
        }
    }

    impl VippsApi {
        pub async fn payment(&self, reference: PaymentReference) -> Result<Payment> {
            let data = mock::MOCK_DB
                .db
                .lock()
                .unwrap()
                .get(&reference)
                .ok_or(Error::Mock)?
                .pay_data
                .clone();

            Ok(Payment {
                api: self.clone(),
                reference,
                data,
            })
        }
    }

    impl Payment {
        pub fn set_mock_state(&self, state: PaymentState) {
            mock::MOCK_DB
                .db
                .lock()
                .unwrap()
                .get_mut(&self.reference)
                .unwrap()
                .pay_data
                .state = state;
        }

        pub fn get_mock_return_url(&self) -> Option<String> {
            mock::MOCK_DB
                .db
                .lock()
                .unwrap()
                .get_mut(&self.reference)
                .unwrap()
                .return_url
                .clone()
        }

        pub async fn cancel(&mut self) -> Result<()> {
            Ok(())
        }

        pub async fn capture(&mut self, _amount: Amount) -> Result<()> {
            Ok(())
        }

        pub async fn refund(&mut self, _amount: Amount) -> Result<()> {
            Ok(())
        }
    }
}
