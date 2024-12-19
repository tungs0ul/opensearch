use opensearch::auth::Credentials;
use opensearch::cert::CertificateValidation;
use opensearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use opensearch::{OpenSearch, SearchParts};
use serde::Deserialize;
use serde_json::Value;
use url::Url;

#[derive(Clone)]
pub struct OpenSearchClient {
    pub pool: OpenSearch,
}

impl OpenSearchClient {
    pub fn new(url: Url, user: String, password: String) -> OpenSearchClient {
        let conn_pool = SingleNodeConnectionPool::new(url);
        let transport = TransportBuilder::new(conn_pool)
            .disable_proxy()
            .cert_validation(CertificateValidation::None)
            .auth(Credentials::Basic(user, password))
            .build()
            .expect("Couldn't build OpenSearch client");
        Self {
            pool: OpenSearch::new(transport),
        }
    }

    pub async fn query_raw(&self, index: &str, query: Value) -> anyhow::Result<Value> {
        let response = self
            .pool
            .search(SearchParts::Index(&[index]))
            .body(query)
            .send()
            .await?
            .json::<OpenSearchResponse>()
            .await?;
        Ok(response.hits.hits)
    }

    pub async fn query<T: for<'a> Deserialize<'a>>(
        &self,
        index: &str,
        query: Value,
    ) -> anyhow::Result<T> {
        let response = self.query_raw(index, query).await?;
        let result = serde_json::from_value(response)?;
        Ok(result)
    }
}

#[derive(Deserialize, Debug)]
struct OpenSearchResponse {
    hits: OpenSearchResponseHit,
}

#[derive(Deserialize, Debug)]
struct OpenSearchResponseHit {
    hits: Value,
}
