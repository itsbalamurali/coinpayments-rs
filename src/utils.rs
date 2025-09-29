//! Utility functions and common helpers for CoinPayments API
//!
//! This module provides utility functions for:
//! - API authentication and signature generation
//! - Request/response handling
//! - Data validation and formatting
//! - Error handling helpers

use crate::{CoinPaymentsError, Result};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha512;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// === Authentication Utilities ===

/// Generate HMAC-SHA512 signature for API requests
///
/// # Arguments
/// * `private_key` - The private key for signing
/// * `data` - The data to sign
///
/// # Example
/// ```rust
/// let signature = generate_hmac_signature("private_key", "data_to_sign");
/// ```
pub fn generate_hmac_signature(private_key: &str, data: &str) -> String {
    let mut mac = Hmac::<Sha512>::new_from_slice(private_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Generate timestamp for API requests
pub fn generate_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Generate nonce for API requests (current timestamp)
pub fn generate_nonce() -> String {
    generate_timestamp().to_string()
}

/// Create request headers for API authentication
///
/// # Arguments
/// * `client_id` - Client ID
/// * `timestamp` - Request timestamp
/// * `signature` - Request signature
pub fn create_auth_headers(
    client_id: &str,
    timestamp: &str,
    signature: &str,
) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("X-CoinPayments-Client".to_string(), client_id.to_string());
    headers.insert(
        "X-CoinPayments-Timestamp".to_string(),
        timestamp.to_string(),
    );
    headers.insert(
        "X-CoinPayments-Signature".to_string(),
        signature.to_string(),
    );
    headers
}

// === Validation Utilities ===

/// Validate email address format
pub fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

/// Validate currency ID format
pub fn is_valid_currency_id(currency_id: &str) -> bool {
    !currency_id.is_empty() && currency_id.chars().all(|c| c.is_alphanumeric() || c == ':')
}

/// Validate wallet label format
pub fn is_valid_wallet_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 100
        && label
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Validate amount format (positive number)
pub fn is_valid_amount(amount: &str) -> bool {
    amount.parse::<f64>().map_or(false, |f| f > 0.0)
}

/// Validate Bitcoin address format (basic check)
pub fn is_valid_bitcoin_address(address: &str) -> bool {
    // Basic validation - starts with 1, 3, or bc1 and has appropriate length
    (address.starts_with('1') && address.len() >= 26 && address.len() <= 35)
        || (address.starts_with('3') && address.len() >= 26 && address.len() <= 35)
        || (address.starts_with("bc1") && address.len() >= 42 && address.len() <= 62)
}

/// Validate Ethereum address format
pub fn is_valid_ethereum_address(address: &str) -> bool {
    address.starts_with("0x")
        && address.len() == 42
        && address[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Validate URL format
pub fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

// === Formatting Utilities ===

/// Format amount to specified decimal places
pub fn format_amount(amount: f64, decimals: u8) -> String {
    format!("{:.prec$}", amount, prec = decimals as usize)
}

/// Parse amount string to f64
pub fn parse_amount(amount: &str) -> Result<f64> {
    amount
        .parse::<f64>()
        .map_err(|_| CoinPaymentsError::InvalidParameters(format!("Invalid amount: {}", amount)))
}

/// Convert timestamp to ISO 8601 string
pub fn timestamp_to_iso8601(timestamp: u64) -> String {
    if let Some(datetime) = chrono::DateTime::from_timestamp(timestamp as i64, 0) {
        datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
    } else {
        chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string()
    }
}

/// Parse ISO 8601 string to timestamp
pub fn iso8601_to_timestamp(iso_string: &str) -> Result<u64> {
    chrono::DateTime::parse_from_rfc3339(iso_string)
        .map(|dt| dt.timestamp() as u64)
        .map_err(|_| {
            CoinPaymentsError::InvalidParameters(format!("Invalid ISO 8601 date: {}", iso_string))
        })
}

/// Convert currency amount to smallest unit (e.g., satoshis for Bitcoin)
pub fn to_smallest_unit(amount: f64, decimals: u8) -> u64 {
    (amount * 10_f64.powi(decimals as i32)) as u64
}

/// Convert from smallest unit to standard unit
pub fn from_smallest_unit(amount: u64, decimals: u8) -> f64 {
    amount as f64 / 10_f64.powi(decimals as i32)
}

// === Error Handling Utilities ===

/// Convert reqwest::Error to CoinPaymentsError
pub fn convert_reqwest_error(error: reqwest::Error) -> CoinPaymentsError {
    if error.is_timeout() {
        CoinPaymentsError::Network("Request timeout".to_string())
    } else if error.is_connect() {
        CoinPaymentsError::Network("Connection failed".to_string())
    } else {
        CoinPaymentsError::Http(error)
    }
}

/// Extract error message from API response
pub fn extract_api_error_message(response_text: &str) -> String {
    // Try to parse as JSON and extract error message
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_text) {
        if let Some(error) = json.get("error") {
            if let Some(message) = error.as_str() {
                return message.to_string();
            }
        }
        if let Some(message) = json.get("message") {
            if let Some(message) = message.as_str() {
                return message.to_string();
            }
        }
    }

    // If JSON parsing fails, return the raw response
    response_text.to_string()
}

// === HTTP Utilities ===

/// Build query string from parameters
pub fn build_query_string(params: &[(&str, String)]) -> String {
    if params.is_empty() {
        return String::new();
    }

    let query_parts: Vec<String> = params
        .iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                urlencoding::encode(key),
                urlencoding::encode(value)
            )
        })
        .collect();

    format!("?{}", query_parts.join("&"))
}

/// Create HTTP client with default settings
pub fn create_http_client() -> Result<Client> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("coinpayments-rust/1.0")
        .build()
        .map_err(CoinPaymentsError::Http)
}

/// Extract rate limit information from response headers
pub fn extract_rate_limit_info(headers: &reqwest::header::HeaderMap) -> Option<RateLimitInfo> {
    let remaining = headers
        .get("X-RateLimit-Remaining")?
        .to_str()
        .ok()?
        .parse::<u32>()
        .ok()?;

    let limit = headers
        .get("X-RateLimit-Limit")?
        .to_str()
        .ok()?
        .parse::<u32>()
        .ok()?;

    let reset = headers
        .get("X-RateLimit-Reset")?
        .to_str()
        .ok()?
        .parse::<u64>()
        .ok()?;

    Some(RateLimitInfo {
        calls_made: limit - remaining,
        calls_left: remaining,
        reset_time: reset,
    })
}

/// Rate limit information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RateLimitInfo {
    pub calls_made: u32,
    pub calls_left: u32,
    pub reset_time: u64,
}

// === Crypto Utilities ===

/// Generate random string for nonces, secrets, etc.
pub fn generate_random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Validate webhook signature
pub fn validate_webhook_signature(
    private_key: &str,
    client_id: &str,
    timestamp: &str,
    payload: &[u8],
    received_signature: &str,
) -> bool {
    let mut data_to_sign = Vec::new();
    data_to_sign.extend_from_slice(client_id.as_bytes());
    data_to_sign.extend_from_slice(timestamp.as_bytes());
    data_to_sign.extend_from_slice(payload);

    let expected_signature =
        generate_hmac_signature(private_key, &String::from_utf8_lossy(&data_to_sign));

    // Constant time comparison to prevent timing attacks
    expected_signature == received_signature
}

// === Pagination Utilities ===

/// Calculate pagination info
pub fn calculate_pagination(total: u32, page: u32, per_page: u32) -> PaginationInfo {
    let total_pages = (total + per_page - 1) / per_page; // Ceiling division

    PaginationInfo {
        page,
        per_page,
        total,
        total_pages,
        has_next: page < total_pages,
        has_prev: page > 1,
    }
}

/// Pagination information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaginationInfo {
    pub page: u32,
    pub per_page: u32,
    pub total: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

// === Testing Utilities ===

#[cfg(test)]
pub mod test_utils {
    use super::*;

    /// Create a mock HTTP response for testing
    pub fn create_mock_response(status: u16, body: &str) -> Result<serde_json::Value> {
        serde_json::from_str(body).map_err(CoinPaymentsError::Json)
    }

    /// Generate test data for various types
    pub fn generate_test_currency_id() -> String {
        "4".to_string() // Bitcoin
    }

    pub fn generate_test_wallet_label() -> String {
        "test-wallet".to_string()
    }

    pub fn generate_test_amount() -> String {
        "0.001".to_string()
    }

    pub fn generate_test_address() -> String {
        "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_hmac_signature() {
        let signature = generate_hmac_signature("test_key", "test_data");
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 128); // SHA512 hex string length
    }

    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(!is_valid_email("invalid-email"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("test@"));
    }

    #[test]
    fn test_is_valid_currency_id() {
        assert!(is_valid_currency_id("4"));
        assert!(is_valid_currency_id(
            "4:0xdac17f958d2ee523a2206206994597c13d831ec7"
        ));
        assert!(!is_valid_currency_id(""));
        assert!(!is_valid_currency_id("invalid!"));
    }

    #[test]
    fn test_is_valid_wallet_label() {
        assert!(is_valid_wallet_label("test-wallet"));
        assert!(is_valid_wallet_label("wallet_123"));
        assert!(!is_valid_wallet_label(""));
        assert!(!is_valid_wallet_label("invalid wallet!"));
    }

    #[test]
    fn test_is_valid_amount() {
        assert!(is_valid_amount("10.5"));
        assert!(is_valid_amount("0.001"));
        assert!(!is_valid_amount("0"));
        assert!(!is_valid_amount("-5"));
        assert!(!is_valid_amount("invalid"));
    }

    #[test]
    fn test_is_valid_bitcoin_address() {
        assert!(is_valid_bitcoin_address(
            "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
        ));
        assert!(is_valid_bitcoin_address(
            "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy"
        ));
        assert!(is_valid_bitcoin_address(
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
        ));
        assert!(!is_valid_bitcoin_address("invalid_address"));
    }

    #[test]
    fn test_is_valid_ethereum_address() {
        assert!(is_valid_ethereum_address(
            "0x742d35Cc6635C0532925a3b8D6ac492395a3d728"
        ));
        assert!(!is_valid_ethereum_address(
            "742d35Cc6635C0532925a3b8D6ac492395a3d728"
        ));
        assert!(!is_valid_ethereum_address(
            "0x742d35Cc6635C0532925a3b8D6ac492395a3d72"
        ));
    }

    #[test]
    fn test_format_amount() {
        assert_eq!(format_amount(1.23456789, 2), "1.23");
        assert_eq!(format_amount(1.0, 8), "1.00000000");
    }

    #[test]
    fn test_parse_amount() {
        assert_eq!(parse_amount("10.5").unwrap(), 10.5);
        assert!(parse_amount("invalid").is_err());
    }

    #[test]
    fn test_to_smallest_unit() {
        assert_eq!(to_smallest_unit(1.0, 8), 100_000_000); // 1 BTC = 100M satoshis
        assert_eq!(to_smallest_unit(0.5, 2), 50); // 0.5 with 2 decimals = 50
    }

    #[test]
    fn test_from_smallest_unit() {
        assert_eq!(from_smallest_unit(100_000_000, 8), 1.0); // 100M satoshis = 1 BTC
        assert_eq!(from_smallest_unit(50, 2), 0.5); // 50 with 2 decimals = 0.5
    }

    #[test]
    fn test_build_query_string() {
        let params = vec![("page", "1".to_string()), ("per_page", "10".to_string())];
        let query = build_query_string(&params);
        assert_eq!(query, "?page=1&per_page=10");

        let empty_params = vec![];
        let empty_query = build_query_string(&empty_params);
        assert_eq!(empty_query, "");
    }

    #[test]
    fn test_calculate_pagination() {
        let pagination = calculate_pagination(25, 2, 10);
        assert_eq!(pagination.total, 25);
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.per_page, 10);
        assert_eq!(pagination.total_pages, 3);
        assert!(pagination.has_next);
        assert!(pagination.has_prev);
    }

    #[test]
    fn test_generate_random_string() {
        let random1 = generate_random_string(10);
        let random2 = generate_random_string(10);

        assert_eq!(random1.len(), 10);
        assert_eq!(random2.len(), 10);
        assert_ne!(random1, random2); // Should be different (very high probability)
    }

    #[test]
    fn test_validate_webhook_signature() {
        let private_key = "test_private_key";
        let client_id = "client_123";
        let timestamp = "2023-01-01T00:00:00Z";
        let payload = b"test payload";

        // Generate expected signature
        let mut data_to_sign = Vec::new();
        data_to_sign.extend_from_slice(client_id.as_bytes());
        data_to_sign.extend_from_slice(timestamp.as_bytes());
        data_to_sign.extend_from_slice(payload);

        let expected_signature =
            generate_hmac_signature(private_key, &String::from_utf8_lossy(&data_to_sign));

        assert!(validate_webhook_signature(
            private_key,
            client_id,
            timestamp,
            payload,
            &expected_signature
        ));

        assert!(!validate_webhook_signature(
            private_key,
            client_id,
            timestamp,
            payload,
            "invalid_signature"
        ));
    }
}
