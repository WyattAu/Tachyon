use super::*;

/// Billing API methods.
///
/// Reserved for future use: subscription and billing management.
impl ApiClient {
    /// List all available billing plans.
    pub async fn get_billing_plans(&self) -> Result<crate::types::BillingPlansResponse, ApiError> {
        let url = format!("{}/billing/plans", self.base_url);
        self.get(&url).await
    }

    /// Fetch the current subscription for an organization.
    pub async fn get_subscription(
        &self,
        org_id: &str,
    ) -> Result<crate::types::SubscriptionResponse, ApiError> {
        let url = format!("{}/billing/subscriptions/{}", self.base_url, org_id);
        self.get(&url).await
    }

    /// Subscribe an organization to the specified plan.
    pub async fn create_subscription(
        &self,
        org_id: &str,
        plan: &str,
    ) -> Result<crate::types::SubscriptionResponse, ApiError> {
        let url = format!("{}/billing/subscriptions", self.base_url);
        let body = serde_json::json!({ "organization_id": org_id, "plan": plan });
        self.post(&url, &body).await
    }

    /// Cancel the subscription for an organization.
    pub async fn cancel_subscription(&self, org_id: &str) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/billing/subscriptions/{}/cancel", self.base_url, org_id);
        self.post_empty_json(&url).await
    }

    /// List invoices for an organization.
    pub async fn get_invoices(
        &self,
        org_id: &str,
    ) -> Result<crate::types::InvoicesResponse, ApiError> {
        let url = format!("{}/billing/invoices/{}", self.base_url, org_id);
        self.get(&url).await
    }

    /// Fetch current resource usage for an organization.
    pub async fn get_usage(&self, org_id: &str) -> Result<crate::types::UsageResponse, ApiError> {
        let url = format!("{}/billing/usage/{}", self.base_url, org_id);
        self.get(&url).await
    }

    pub async fn create_mandate(
        &self,
        org_id: &str,
        return_url: &str,
    ) -> Result<crate::types::MandateResponse, ApiError> {
        let url = format!("{}/billing/mandates", self.base_url);
        let body = serde_json::json!({ "organization_id": org_id, "return_url": return_url });
        self.post(&url, &body).await
    }

    pub async fn get_mandate_status(
        &self,
        mandate_id: &str,
    ) -> Result<crate::types::MandateStatusResponse, ApiError> {
        let url = format!("{}/billing/mandates/{}", self.base_url, mandate_id);
        self.get(&url).await
    }

    pub async fn get_payment_status(
        &self,
        payment_id: &str,
    ) -> Result<crate::types::PaymentStatusResponse, ApiError> {
        let url = format!("{}/billing/payments/{}", self.base_url, payment_id);
        self.get(&url).await
    }
}
