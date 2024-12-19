use crate::opensearch_client::OpenSearchClient;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{debug_handler, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

pub fn build_router(opensearch_client: OpenSearchClient) -> Router {
    let state = AppState::new(opensearch_client);
    Router::new()
        .route("/health_check", get(|| async { StatusCode::OK }))
        .route("/query", get(query_handler))
        .with_state(Arc::new(state))
}

#[debug_handler]
async fn query_handler(
    State(state): State<Arc<AppState>>,
    Json(query): Json<Value>,
) -> Result<Json<Vec<OpenSearchResponseRow>>, AppError> {
    let result = state
        .opensearch_client
        .query("ecommerce", query)
        .await
        .map_err(|err| AppError::Error(err.to_string()))?;
    Ok(Json(result))
}

#[derive(Deserialize, Debug, Serialize)]
struct OpenSearchResponseRow {
    _source: OpenSearchSource,
}

#[derive(Deserialize, Debug, Serialize)]
struct OpenSearchSource {
    currency: String,
    customer_first_name: String,
    customer_full_name: String,
    customer_gender: String,
    customer_id: i32,
    customer_last_name: String,
    customer_phone: String,
    day_of_week: String,
    day_of_week_i: u8,
    email: String,
    manufacturer: Vec<String>,
    order_date: chrono::DateTime<chrono::Utc>,
    category: Vec<String>,
    order_id: i32,
    products: Vec<Product>,
}

#[derive(Deserialize, Debug, Serialize)]
struct Product {
    base_price: f32,
    discount_percentage: f32,
    quantity: u8,
    manufacturer: String,
    tax_amount: f32,
    product_id: i32,
    category: String,
    sku: String,
    taxless_price: f32,
    unit_discount_amount: u8,
    min_price: f32,
    _id: String,
    discount_amount: f32,
    created_on: chrono::DateTime<chrono::Utc>,
    product_name: String,
    price: f32,
    taxful_price: f32,
    base_unit_price: f32,
}

#[derive(Debug)]
pub enum AppError {
    #[allow(dead_code)]
    AuthError(StatusCode),
    Error(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::AuthError(status) => status.into_response(),
            AppError::Error(s) => (StatusCode::INTERNAL_SERVER_ERROR, s).into_response(),
        }
    }
}

pub struct AppState {
    opensearch_client: OpenSearchClient,
}

impl AppState {
    pub fn new(opensearch_client: OpenSearchClient) -> Self {
        Self { opensearch_client }
    }
}
