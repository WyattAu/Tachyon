//! Billing API routes
//! TrueLayer open banking payment integration (NOT Stripe)

pub mod handlers;
pub mod types;

#[cfg(test)]
mod tests;

pub use handlers::{
    calculate_proration, cancel_subscription, change_plan, create_billing_router, create_mandate,
    create_payment, create_subscription, get_mandate_status, get_payment_status, get_subscription,
    get_usage, list_invoices, list_plans, plan_features, plan_max_docs, plan_max_members,
    plan_price, plan_price_f64, truelayer_error_response, validate_plan_name,
    validate_plan_transition, verify_webhook_signature, webhook_handler,
};
pub use types::{
    BillingErrorResponse, BillingState, ChangePlanRequest, ChangePlanResponse,
    CreateMandateRequest, CreatePaymentRequest, CreateSubscriptionRequest, HmacSha256,
    InvoicesResponse, MandateResponse, MandateStatusResponse, PaymentResponse,
    PaymentStatusResponse, Plan, PlanDetails, PlanInfo, PlansResponse, ProrationResult,
    SubscriptionResponse, TransitionType, UsageMetrics, UsageResponse, WebhookPayload,
};
