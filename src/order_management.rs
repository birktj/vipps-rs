use crate::*;

impl epayment::Payment {
    #[tracing::instrument(skip_all, err)]
    pub async fn add_category(&self, category: OrderCategory, details_url: &str) -> Result<()> {
        let req = AddCategoryReq {
            category,
            order_details_url: details_url.to_string(),
            image_id: None,
        };

        let _res = self
            .api
            .0
            .client
            .put(format!(
                "{}/order-management/v2/ecom/categories/{}",
                self.api.0.base_url, &self.reference.0
            ))
            .bearer_auth(self.api.access_token().await?.token())
            .json(&req)
            .send()
            .await?
            .into_vipps_result()
            .await?;

        tracing::debug!("Added category to order");

        Ok(())
    }

    pub fn add_reciept(&self, currency: Currency) -> RecieptBuilder {
        RecieptBuilder {
            payment: &self,
            req: AddRecieptReq {
                order_lines: Vec::new(),
                bottom_line: RecieptBottomLine { currency },
            },
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddCategoryReq {
    category: OrderCategory,
    order_details_url: String,
    image_id: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderCategory {
    General,
    Reciept,
    OrderConfirmation,
    Delivery,
    Ticket,
    Booking,
}

pub struct RecieptBuilder<'a> {
    payment: &'a epayment::Payment,
    req: AddRecieptReq,
}

impl<'a> RecieptBuilder<'a> {
    #[tracing::instrument(skip_all, err)]
    pub async fn send(self) -> Result<()> {
        let _res = self
            .payment
            .api
            .0
            .client
            .post(format!(
                "{}/order-management/v2/ecom/receipts/{}",
                self.payment.api.0.base_url, &self.payment.reference.0
            ))
            .bearer_auth(self.payment.api.access_token().await?.token())
            .json(&self.req)
            .send()
            .await?
            .into_vipps_result()
            .await?;

        tracing::debug!("Added reciept to order");

        Ok(())
    }

    pub fn order_line(mut self, order_line: OrderLine) -> Self {
        self.req.order_lines.push(order_line);
        self
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddRecieptReq {
    order_lines: Vec<OrderLine>,
    bottom_line: RecieptBottomLine,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderLine {
    pub name: String,
    pub id: String,
    pub total_amount: i64,
    pub total_amount_excluding_tax: i64,
    pub total_tax_amount: i64,
    pub tax_percentage: i32,
    pub unit_info: Option<UnitInfo>,
    pub discount: Option<i64>,
    pub product_url: Option<String>,
    pub is_return: Option<bool>,
    pub is_shipping: Option<bool>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitInfo {
    pub unit_price: i64,
    pub quantity: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecieptBottomLine {
    currency: Currency,
}
