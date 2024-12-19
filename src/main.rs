use open_search_api::api;
use open_search_api::opensearch_client::OpenSearchClient;
use std::net::SocketAddr;
use tracing_subscriber::FmtSubscriber;
use url::Url;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_line_number(true)
        .with_file(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let url = std::env::var("OPEN_SEARCH_URL").expect("No open search url provided");
    let open_search_username =
        std::env::var("OPEN_SEARCH_USERNAME").expect("No open search username provided");
    let open_search_password =
        std::env::var("OPEN_SEARCH_PASSWORD").expect("No open search password");
    let url = Url::parse(&url).expect("Incorrect opensearch url");
    let client = OpenSearchClient::new(url, open_search_username, open_search_password);
    let app = api::build_router(client);

    let port = std::env::var("API_PORT")
        .expect("No api port")
        .parse()
        .expect("Could not parse port");
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Could not bind to {port}");

    tracing::info!("Server is listening on port 3000");
    axum::serve(listener, app).await.unwrap();
}
