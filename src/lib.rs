//! CoinPayments API Client
//!
//! A comprehensive Rust client library for the CoinPayments API that provides easy access to
//! cryptocurrency payment processing, wallet management, transaction handling, and more.
//!
//! # Features
//!
//! - **Currencies**: Get supported currencies, rates, and conversion information
//! - **Rates**: Real-time exchange rates and market data
//! - **Fees**: Blockchain fee calculations and estimates
//! - **Wallets**: Create and manage wallets and addresses
//! - **Transactions**: Handle payments, withdrawals, and consolidations
//! - **Invoices**: Create and manage payment invoices
//! - **Webhooks**: Set up and manage webhook notifications
//!
//! # Quick Start
//!
//! ```rust
//! use coinpayments::{CoinPaymentsClient, CreateInvoiceRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = CoinPaymentsClient::new("your_client_id", "your_client_secret");
//!
//!     // Create an invoice
//!     let request = CreateInvoiceRequest::new("10.00", "USD", "Payment for services")
//!         .with_buyer("customer@example.com", Some("John Doe".to_string()));
//!
//!     let invoice = client.create_invoice(request).await?;
//!     println!("Invoice created: {}", invoice.invoice.invoice_url);
//!
//!     Ok(())
//! }
//! ```

use reqwest::Client;
use serde::{Deserialize, Serialize};

use thiserror::Error;

// Re-export all module types for easier access
pub use currencies::*;
pub use fees::*;
pub use invoices::*;
pub use rates::*;
pub use transactions::*;
pub use utils::{
    build_query_string, calculate_pagination, convert_reqwest_error, create_auth_headers,
    create_http_client, extract_api_error_message, extract_rate_limit_info, format_amount,
    from_smallest_unit, generate_hmac_signature, generate_nonce, generate_random_string,
    generate_timestamp, is_valid_amount, is_valid_bitcoin_address, is_valid_currency_id,
    is_valid_email, is_valid_ethereum_address, is_valid_url, is_valid_wallet_label,
    iso8601_to_timestamp, parse_amount, timestamp_to_iso8601, to_smallest_unit,
    validate_webhook_signature, PaginationInfo as UtilsPaginationInfo, RateLimitInfo,
};
pub use wallets::*;
pub use webhooks::*;

// Module declarations
pub mod currencies;
pub mod fees;
pub mod invoices;
pub mod rates;
pub mod transactions;
pub mod utils;
pub mod wallets;
pub mod webhooks;

/// Base URL for CoinPayments API
const API_BASE_URL: &str = "https://a-api.coinpayments.net/api";

/// CoinPayments API Client
#[derive(Debug, Clone)]
pub struct CoinPaymentsClient {
    client: Client,
    client_id: String,
    client_secret: String,
    base_url: String,
}

/// API Error types
#[derive(Error, Debug)]
pub enum CoinPaymentsError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API error: {message}")]
    Api { message: String },

    #[error("Authentication failed")]
    Authentication,

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Resource not found")]
    NotFound,

    #[error("Insufficient funds")]
    InsufficientFunds,
}

/// Result type alias for CoinPayments operations
pub type Result<T> = std::result::Result<T, CoinPaymentsError>;

/// Generic API response wrapper
#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMetadata>,
}

/// API error information
#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Pagination metadata
#[derive(Debug, Deserialize, Serialize)]
pub struct PaginationMetadata {
    pub page: u32,
    pub per_page: u32,
    pub total: u32,
    pub total_pages: u32,
}

/// Authentication type
#[derive(Debug, Clone)]
pub enum AuthType {
    /// OAuth 2.0 authentication
    OAuth(String),
    /// Client ID and Secret authentication
    ClientCredentials {
        client_id: String,
        client_secret: String,
    },
}

impl CoinPaymentsClient {
    /// Create a new CoinPayments client with client credentials
    ///
    /// # Arguments
    /// * `client_id` - Your CoinPayments client ID
    /// * `client_secret` - Your CoinPayments client secret
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("your_client_id", "your_client_secret");
    /// ```
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            base_url: API_BASE_URL.to_string(),
        }
    }

    /// Create a new client with custom HTTP client
    ///
    /// # Arguments
    /// * `client` - Custom reqwest client
    /// * `client_id` - Your CoinPayments client ID
    /// * `client_secret` - Your CoinPayments client secret
    pub fn with_client(
        client: Client,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
    ) -> Self {
        Self {
            client,
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            base_url: API_BASE_URL.to_string(),
        }
    }

    /// Set a custom base URL (useful for testing)
    ///
    /// # Arguments
    /// * `base_url` - Custom API base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Generate timestamp for API requests
    fn generate_timestamp(&self) -> String {
        chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string()
    }

    /// Generate HMAC signature for API request
    fn generate_signature(
        &self,
        timestamp: &str,
        method: &str,
        endpoint: &str,
        body: &str,
    ) -> String {
        let data_to_sign = format!(
            "{}{}{}{}",
            self.client_id,
            timestamp,
            method.to_uppercase(),
            endpoint
        );
        let data_with_body = if !body.is_empty() {
            format!("{}{}", data_to_sign, body)
        } else {
            data_to_sign
        };

        utils::generate_hmac_signature(&self.client_secret, &data_with_body)
    }

    /// Create authentication headers
    fn create_auth_headers(
        &self,
        method: &str,
        endpoint: &str,
        body: &str,
    ) -> std::collections::HashMap<String, String> {
        let timestamp = self.generate_timestamp();
        let signature = self.generate_signature(&timestamp, method, endpoint, body);

        utils::create_auth_headers(&self.client_id, &timestamp, &signature)
    }

    /// Make a GET request to the API
    pub(crate) async fn get_request<T>(
        &self,
        endpoint: &str,
        query_params: &[(&str, String)],
    ) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.base_url, endpoint);
        let query_string = utils::build_query_string(query_params);
        let full_url = format!("{}{}", url, query_string);

        let auth_headers = self.create_auth_headers("GET", endpoint, "");

        let mut request = self
            .client
            .get(&full_url)
            .header("Content-Type", "application/json");

        for (key, value) in auth_headers {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// Make a POST request to the API
    pub(crate) async fn post_request<T, B>(&self, endpoint: &str, body: &B) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        let url = format!("{}/{}", self.base_url, endpoint);
        let body_json = serde_json::to_string(body)?;

        let auth_headers = self.create_auth_headers("POST", endpoint, &body_json);

        let mut request = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body_json);

        for (key, value) in auth_headers {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// Make a PUT request to the API
    pub(crate) async fn put_request<T, B>(&self, endpoint: &str, body: &B) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        let url = format!("{}/{}", self.base_url, endpoint);
        let body_json = serde_json::to_string(body)?;

        let auth_headers = self.create_auth_headers("PUT", endpoint, &body_json);

        let mut request = self
            .client
            .put(&url)
            .header("Content-Type", "application/json")
            .body(body_json);

        for (key, value) in auth_headers {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// Make a DELETE request to the API
    pub(crate) async fn delete_request<T>(&self, endpoint: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.base_url, endpoint);

        let auth_headers = self.create_auth_headers("DELETE", endpoint, "");

        let mut request = self
            .client
            .delete(&url)
            .header("Content-Type", "application/json");

        for (key, value) in auth_headers {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// Handle API response and convert to Result
    async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let status = response.status();
        let response_text = response.text().await?;

        // Handle HTTP error status codes
        if !status.is_success() {
            let error_message = utils::extract_api_error_message(&response_text);

            return Err(match status.as_u16() {
                401 => CoinPaymentsError::Authentication,
                404 => CoinPaymentsError::NotFound,
                429 => CoinPaymentsError::RateLimit,
                _ => CoinPaymentsError::Api {
                    message: format!("HTTP {}: {}", status, error_message),
                },
            });
        }

        // Try to parse the response
        serde_json::from_str::<T>(&response_text).or_else(|parse_error| {
            // If direct parsing fails, try parsing as ApiResponse wrapper
            match serde_json::from_str::<ApiResponse<T>>(&response_text) {
                Ok(api_response) => {
                    if let Some(error) = api_response.error {
                        Err(CoinPaymentsError::Api {
                            message: format!("API Error: {}", error.message),
                        })
                    } else if let Some(data) = api_response.data {
                        Ok(data)
                    } else {
                        Err(CoinPaymentsError::Api {
                            message: "No data in response".to_string(),
                        })
                    }
                }
                Err(_) => Err(CoinPaymentsError::Api {
                    message: format!(
                        "Failed to parse response: {} - Response: {}",
                        parse_error, response_text
                    ),
                }),
            }
        })
    }

    /// Get client information
    pub async fn get_client_info(&self) -> Result<ClientInfo> {
        self.get_request("v1/client/info", &[]).await
    }

    /// Test API connectivity and authentication
    pub async fn ping(&self) -> Result<PingResponse> {
        self.get_request("v1/ping", &[]).await
    }
}

/// Client information
#[derive(Debug, Deserialize, Serialize)]
pub struct ClientInfo {
    pub client_id: String,
    pub name: String,
    pub permissions: Vec<String>,
    pub rate_limits: RateLimits,
    pub created_at: String,
    pub updated_at: String,
}

/// Rate limit information
#[derive(Debug, Deserialize, Serialize)]
pub struct RateLimits {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
}

/// Ping response
#[derive(Debug, Deserialize, Serialize)]
pub struct PingResponse {
    pub message: String,
    pub timestamp: String,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = CoinPaymentsClient::new("test_client_id", "test_client_secret");
        assert_eq!(client.client_id, "test_client_id");
        assert_eq!(client.client_secret, "test_client_secret");
        assert_eq!(client.base_url, API_BASE_URL);
    }

    #[test]
    fn test_signature_generation() {
        let client = CoinPaymentsClient::new("test_client", "test_secret");
        let signature = client.generate_signature("2023-01-01T00:00:00.000Z", "GET", "/test", "");
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 128); // SHA512 hex string length
    }

    #[test]
    fn test_custom_base_url() {
        let client = CoinPaymentsClient::new("test_client", "test_secret")
            .with_base_url("https://custom-api.example.com");
        assert_eq!(client.base_url, "https://custom-api.example.com");
    }

    #[tokio::test]
    async fn test_api_error_handling() {
        // This test would require a mock HTTP client
        // For now, just test that the error types can be created
        let error = CoinPaymentsError::Api {
            message: "Test error".to_string(),
        };
        assert!(matches!(error, CoinPaymentsError::Api { .. }));
    }
}
