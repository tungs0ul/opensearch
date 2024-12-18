use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{debug_handler, Json, Router};
use opensearch::auth::Credentials;
use opensearch::cert::CertificateValidation;
use opensearch::http::transport::{BuildError, SingleNodeConnectionPool, TransportBuilder};
use opensearch::{OpenSearch, SearchParts};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
enum AppError {
    #[error("Url parse error")]
    UrlParse(#[from] url::ParseError),
    #[error("Opensearch transport build error")]
    OpenSearchTransportBuild(#[from] BuildError),
    #[error("Opensearch request error")]
    OpenSearchRequest(#[from] opensearch::Error),
    #[error("")]
    SerdeJson(#[from] serde_json::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let url = std::env::var("OPEN_SEARCH_URL").expect("No open search url provided");
    let url = Url::parse(&url)?;
    let conn_pool = SingleNodeConnectionPool::new(url);
    let transport = TransportBuilder::new(conn_pool)
        .disable_proxy()
        .cert_validation(CertificateValidation::None)
        .auth(Credentials::Basic("admin".into(), "admin".into()))
        .build()?;
    let client = OpenSearch::new(transport);

    let app = Router::new()
        .route("/first-names/:name", get(handler))
        .with_state(Arc::new(client));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
    Ok(())
}

#[debug_handler]
async fn handler(
    State(client): State<Arc<OpenSearch>>,
    Path(name): Path<String>,
) -> Result<Json<Vec<OpenSearchResponseRow>>, AppError> {
    let response = client
        .search(SearchParts::Index(&["ecommerce"]))
        .from(0)
        .body(json!({"query":{"match":{"customer_first_name":name}}}))
        .send()
        .await?
        .json::<OpenSearchResponse>()
        .await?;

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
