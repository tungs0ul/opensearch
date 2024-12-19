use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{debug_handler, Json, Router};
use opensearch::{OpenSearch, SearchParts};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

pub fn build_router(client: OpenSearch) -> Router {
    Router::new()
        .route("/health_check", get(|| async { StatusCode::OK }))
        .route("/query", get(query_handler))
        .with_state(Arc::new(client))
}

#[debug_handler]
async fn query_handler(
    State(client): State<Arc<OpenSearch>>,
    Json(query): Json<Value>,
) -> Result<Json<Vec<OpenSearchResponseRow>>, AppError> {
    let response = client
        .search(SearchParts::Index(&["ecommerce"]))
        .body(query)
        .send()
        .await
        .map_err(|err| AppError::Error(err.to_string()))?
        .json::<OpenSearchResponse>()
        .await
        .map_err(|err| AppError::Error(err.to_string()))?;

    Ok(Json(response.hits.hits))
}

#[derive(Deserialize, Debug)]
struct OpenSearchResponse {
    hits: OpenSearchResponseHit,
}

#[derive(Deserialize, Debug)]
struct OpenSearchResponseHit {
    hits: Vec<OpenSearchResponseRow>,
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
