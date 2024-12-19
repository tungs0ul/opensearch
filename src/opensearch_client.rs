use opensearch::auth::Credentials;
use opensearch::cert::CertificateValidation;
use opensearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use opensearch::OpenSearch;
use url::Url;

pub fn build_client(url: Url, user: String, password: String) -> OpenSearch {
    let conn_pool = SingleNodeConnectionPool::new(url);
    let transport = TransportBuilder::new(conn_pool)
        .disable_proxy()
        .cert_validation(CertificateValidation::None)
        .auth(Credentials::Basic(user, password))
        .build()
        .expect("Couldn't build OpenSearch client");
    OpenSearch::new(transport)
}
