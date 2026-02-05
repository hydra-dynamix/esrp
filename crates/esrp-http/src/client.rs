//! Reqwest-based ESRP HTTP client

use crate::error::ESRPHttpError;
use esrp_core::{ESRPRequest, ESRPResponse};
use reqwest::Client;
use std::time::Duration;

/// ESRP HTTP client for making requests to ESRP-compatible services
///
/// # Example
///
/// ```ignore
/// use esrp_http::ESRPClient;
/// use esrp_core::ESRPRequest;
///
/// let client = ESRPClient::new("http://localhost:8080");
///
/// let request = ESRPRequest { /* ... */ };
/// let response = client.execute(request).await?;
/// ```
pub struct ESRPClient {
    client: Client,
    base_url: String,
}

impl ESRPClient {
    /// Create a new ESRP client with the given base URL
    ///
    /// The base URL should not include a trailing slash.
    /// The client will append `/v1/execute` for execute requests.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(600))
                .build()
                .unwrap(),
            base_url: base_url.into(),
        }
    }

    /// Create a new ESRP client with custom settings
    pub fn with_client(client: Client, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into(),
        }
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Execute an ESRP request
    ///
    /// Sends the request to `{base_url}/v1/execute` and returns the response.
    pub async fn execute(&self, request: ESRPRequest) -> Result<ESRPResponse, ESRPHttpError> {
        let url = format!("{}/v1/execute", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(ESRPHttpError::RequestError)?;

        let esrp_response: ESRPResponse =
            response.json().await.map_err(ESRPHttpError::RequestError)?;

        Ok(esrp_response)
    }

    /// Execute an ESRP request to a specific endpoint
    ///
    /// Sends the request to `{base_url}/{path}` and returns the response.
    pub async fn execute_at(
        &self,
        path: &str,
        request: ESRPRequest,
    ) -> Result<ESRPResponse, ESRPHttpError> {
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(ESRPHttpError::RequestError)?;

        let esrp_response: ESRPResponse =
            response.json().await.map_err(ESRPHttpError::RequestError)?;

        Ok(esrp_response)
    }
}

impl Default for ESRPClient {
    fn default() -> Self {
        Self::new("http://localhost:8080")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ESRPClient::new("http://localhost:8080");
        assert_eq!(client.base_url(), "http://localhost:8080");
    }

    #[test]
    fn test_default_client() {
        let client = ESRPClient::default();
        assert_eq!(client.base_url(), "http://localhost:8080");
    }

    #[test]
    fn test_custom_base_url() {
        let client = ESRPClient::new("https://api.example.com");
        assert_eq!(client.base_url(), "https://api.example.com");
    }
}
