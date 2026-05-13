use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct OnboardingState {
    pub pool: tachyon_database::DatabasePool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OnboardingStatusResponse {
    pub completed: bool,
    pub steps: Vec<tachyon_database::OnboardingStep>,
    pub current_step: usize,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CompleteStepRequest {
    pub step_id: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CompleteStepResponse {
    pub success: bool,
    pub step_id: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SampleContentResponse {
    pub created: usize,
    pub skipped: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SuggestionsResponse {
    pub suggested_tags: Vec<String>,
    pub suggested_templates: Vec<TemplateSuggestion>,
    pub tips: Vec<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TemplateSuggestion {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}
