//! Webhook management API endpoints for CoinPayments API
//!
//! This module provides functionality for:
//! - Managing client webhooks for invoice events
//! - Managing wallet/address webhooks for transaction events
//! - Webhook authentication and verification
//! - Event handling and payload processing

use crate::{CoinPaymentsClient, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// === Webhook Types ===

/// Client webhook configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClientWebhook {
    pub id: String,
    pub client_id: String,
    pub url: String,
    pub events: Vec<ClientWebhookEvent>,
    pub secret: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Client webhook events for invoices
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ClientWebhookEvent {
    /// Triggered when a new invoice is created
    InvoiceCreated,
    /// Triggered when payment is detected and has at least one confirmation
    InvoicePending,
    /// Triggered when invoice has received all required confirmations
    InvoicePaid,
    /// Triggered when invoice is paid and funds are reflected in merchant's balance
    InvoiceCompleted,
    /// Triggered when invoice is cancelled by merchant
    InvoiceCancelled,
    /// Triggered when invoice has expired
    InvoiceTimedOut,
    /// Triggered when temporary address for payment is created
    PaymentCreated,
    /// Triggered when temporary address for payment is no longer available
    PaymentTimedOut,
}

/// Wallet webhook configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WalletWebhook {
    pub wallet_id: String,
    pub wallet_label: String,
    pub currency_id: String,
    pub url: String,
    pub events: Vec<WalletWebhookEvent>,
    pub secret: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Wallet webhook events for transactions
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum WalletWebhookEvent {
    /// Receiving funds from within the CoinPayments system
    InternalReceive,
    /// Receiving UTXO funds from an external source
    UtxoExternalReceive,
    /// Receiving funds from external account-based transfers
    AccountBasedExternalReceive,
    /// Sending funds from one CoinPayments user to another
    InternalSpend,
    /// Sending funds to an external address
    ExternalSpend,
    /// Receiving funds from one wallet to another for the same CoinPayments user
    SameUserReceive,
    /// Receiving tokens from external account-based transfers
    AccountBasedExternalTokenReceive,
    /// Sending account-based tokens to external address
    AccountBasedTokenSpend,
}

/// Address webhook configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddressWebhook {
    pub address_id: String,
    pub address_label: String,
    pub wallet_id: String,
    pub currency_id: String,
    pub url: String,
    pub events: Vec<WalletWebhookEvent>,
    pub secret: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Request to create a client webhook
#[derive(Debug, Serialize, Clone)]
pub struct CreateClientWebhookRequest {
    pub url: String,
    pub events: Vec<ClientWebhookEvent>,
    pub secret: Option<String>,
    pub is_active: Option<bool>,
}

/// Request to update webhook configuration
#[derive(Debug, Serialize, Clone)]
pub struct UpdateWebhookRequest {
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
    pub is_active: Option<bool>,
}

/// Webhook payload for client events (invoices)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClientWebhookPayload {
    pub event: ClientWebhookEvent,
    pub invoice_id: String,
    pub merchant_id: String,
    pub amount: String,
    pub currency: String,
    pub status: String,
    pub created_at: String,
    pub payment_data: Option<PaymentData>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Payment data in webhook payload
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaymentData {
    pub currency_id: String,
    pub address: String,
    pub amount: String,
    pub txid: Option<String>,
    pub confirmations: Option<u32>,
    pub first_seen: Option<String>,
}

/// Webhook payload for wallet events
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WalletWebhookPayload {
    pub event: WalletWebhookEvent,
    pub wallet_id: String,
    pub wallet_label: String,
    pub address_id: Option<String>,
    pub address: String,
    pub currency_id: String,
    pub transaction_id: String,
    pub amount: String,
    pub fee: Option<String>,
    pub txid: Option<String>,
    pub confirmations: u32,
    pub status: String,
    pub created_at: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Webhook authentication headers
#[derive(Debug, Clone)]
pub struct WebhookHeaders {
    pub client_id: String,
    pub timestamp: String,
    pub signature: String,
}

impl Default for CreateClientWebhookRequest {
    fn default() -> Self {
        Self {
            url: String::new(),
            events: vec![ClientWebhookEvent::InvoiceCompleted],
            secret: None,
            is_active: Some(true),
        }
    }
}

impl CreateClientWebhookRequest {
    /// Create a new client webhook request
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Set the events to listen for
    pub fn with_events(mut self, events: Vec<ClientWebhookEvent>) -> Self {
        self.events = events;
        self
    }

    /// Set webhook secret for verification
    pub fn with_secret(mut self, secret: impl Into<String>) -> Self {
        self.secret = Some(secret.into());
        self
    }

    /// Set webhook active status
    pub fn active(mut self, active: bool) -> Self {
        self.is_active = Some(active);
        self
    }
}

impl CoinPaymentsClient {
    /// Create a client webhook
    ///
    /// # Arguments
    /// * `client_id` - Client ID to create webhook for
    /// * `request` - Webhook creation request
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let request = CreateClientWebhookRequest::new("https://example.com/webhook")
    ///     .with_events(vec![
    ///         ClientWebhookEvent::InvoiceCreated,
    ///         ClientWebhookEvent::InvoiceCompleted
    ///     ])
    ///     .with_secret("webhook_secret");
    /// let webhook = client.create_client_webhook("client_123", request).await?;
    /// ```
    pub async fn create_client_webhook(
        &self,
        client_id: &str,
        request: CreateClientWebhookRequest,
    ) -> Result<ClientWebhook> {
        let endpoint = format!("v1/merchant/clients/{}/webhooks", client_id);
        self.post_request(&endpoint, &request).await
    }

    /// Update wallet webhook (v2 API - by ID)
    ///
    /// # Arguments
    /// * `wallet_id` - Wallet ID
    /// * `request` - Webhook update request
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let request = UpdateWebhookRequest {
    ///     url: "https://example.com/webhook".to_string(),
    ///     events: vec!["externalReceive".to_string(), "externalSpend".to_string()],
    ///     secret: Some("new_secret".to_string()),
    ///     is_active: Some(true),
    /// };
    /// client.update_wallet_webhook_v2("wallet_123", request).await?;
    /// ```
    pub async fn update_wallet_webhook_v2(
        &self,
        wallet_id: &str,
        request: UpdateWebhookRequest,
    ) -> Result<()> {
        let endpoint = format!("v2/merchant/wallets/{}/webhook", wallet_id);
        self.put_request(&endpoint, &request).await
    }

    /// Update address webhook (v2 API - by ID)
    ///
    /// # Arguments
    /// * `wallet_id` - Wallet ID
    /// * `address_id` - Address ID
    /// * `request` - Webhook update request
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let request = UpdateWebhookRequest {
    ///     url: "https://example.com/webhook".to_string(),
    ///     events: vec!["externalReceive".to_string()],
    ///     secret: Some("address_secret".to_string()),
    ///     is_active: Some(true),
    /// };
    /// client.update_address_webhook_v2("wallet_123", "addr_456", request).await?;
    /// ```
    pub async fn update_address_webhook_v2(
        &self,
        wallet_id: &str,
        address_id: &str,
        request: UpdateWebhookRequest,
    ) -> Result<()> {
        let endpoint = format!(
            "v2/merchant/wallets/{}/addresses/{}/webhook",
            wallet_id, address_id
        );
        self.put_request(&endpoint, &request).await
    }

    /// Update wallet webhook (v3 API - by label)
    ///
    /// # Arguments
    /// * `wallet_label` - Wallet label
    /// * `currency_id` - Currency ID
    /// * `request` - Webhook update request
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let request = UpdateWebhookRequest {
    ///     url: "https://example.com/webhook".to_string(),
    ///     events: vec!["externalReceive".to_string(), "externalSpend".to_string()],
    ///     secret: Some("wallet_secret".to_string()),
    ///     is_active: Some(true),
    /// };
    /// client.update_wallet_webhook_v3("my-btc-wallet", "4", request).await?;
    /// ```
    pub async fn update_wallet_webhook_v3(
        &self,
        wallet_label: &str,
        currency_id: &str,
        request: UpdateWebhookRequest,
    ) -> Result<()> {
        let endpoint = format!(
            "v3/merchant/wallets/{}/{}/webhook",
            wallet_label, currency_id
        );
        self.put_request(&endpoint, &request).await
    }

    /// Update address webhook (v3 API - by label)
    ///
    /// # Arguments
    /// * `wallet_label` - Wallet label
    /// * `currency_id` - Currency ID
    /// * `address_label` - Address label
    /// * `request` - Webhook update request
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let request = UpdateWebhookRequest {
    ///     url: "https://example.com/webhook".to_string(),
    ///     events: vec!["externalReceive".to_string()],
    ///     secret: Some("address_secret".to_string()),
    ///     is_active: Some(true),
    /// };
    /// client.update_address_webhook_v3("my-btc-wallet", "4", "address-1", request).await?;
    /// ```
    pub async fn update_address_webhook_v3(
        &self,
        wallet_label: &str,
        currency_id: &str,
        address_label: &str,
        request: UpdateWebhookRequest,
    ) -> Result<()> {
        let endpoint = format!(
            "v3/merchant/wallets/{}/{}/addresses/{}/webhook",
            wallet_label, currency_id, address_label
        );
        self.put_request(&endpoint, &request).await
    }
}

// === Webhook Verification ===

/// Verify webhook signature
///
/// # Arguments
/// * `private_key` - Your integration private key
/// * `headers` - Webhook headers containing signature
/// * `payload` - Raw webhook payload body
///
/// # Example
/// ```rust
/// let headers = WebhookHeaders {
///     client_id: "your_client_id".to_string(),
///     timestamp: "2023-01-01T00:00:00Z".to_string(),
///     signature: "received_signature".to_string(),
/// };
/// let is_valid = verify_webhook_signature("private_key", &headers, &payload_body);
/// ```
pub fn verify_webhook_signature(
    private_key: &str,
    headers: &WebhookHeaders,
    payload: &[u8],
) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha512;

    // Create the data to sign: client_id + timestamp + payload
    let mut data_to_sign = Vec::new();
    data_to_sign.extend_from_slice(headers.client_id.as_bytes());
    data_to_sign.extend_from_slice(headers.timestamp.as_bytes());
    data_to_sign.extend_from_slice(payload);

    // Generate HMAC signature
    let mut mac = match Hmac::<Sha512>::new_from_slice(private_key.as_bytes()) {
        Ok(mac) => mac,
        Err(_) => return false,
    };

    mac.update(&data_to_sign);
    let expected_signature = hex::encode(mac.finalize().into_bytes());

    // Compare signatures (constant time comparison)
    expected_signature == headers.signature
}

/// Parse webhook headers from HTTP request
///
/// # Arguments
/// * `header_map` - HTTP headers map
///
/// # Example
/// ```rust
/// // Using with axum or other web framework
/// let headers = parse_webhook_headers(&request_headers)?;
/// ```
pub fn parse_webhook_headers(header_map: &HashMap<String, String>) -> Result<WebhookHeaders> {
    let client_id = header_map
        .get("X-CoinPayments-Client")
        .ok_or_else(|| crate::CoinPaymentsError::Api {
            message: "Missing X-CoinPayments-Client header".to_string(),
        })?
        .clone();

    let timestamp = header_map
        .get("X-CoinPayments-Timestamp")
        .ok_or_else(|| crate::CoinPaymentsError::Api {
            message: "Missing X-CoinPayments-Timestamp header".to_string(),
        })?
        .clone();

    let signature = header_map
        .get("X-CoinPayments-Signature")
        .ok_or_else(|| crate::CoinPaymentsError::Api {
            message: "Missing X-CoinPayments-Signature header".to_string(),
        })?
        .clone();

    Ok(WebhookHeaders {
        client_id,
        timestamp,
        signature,
    })
}

/// Check if webhook timestamp is within acceptable time window
///
/// # Arguments
/// * `timestamp` - Webhook timestamp in ISO 8601 format
/// * `tolerance_seconds` - Maximum age in seconds (default: 300 = 5 minutes)
pub fn is_webhook_timestamp_valid(timestamp: &str, tolerance_seconds: u64) -> bool {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Parse the timestamp
    let webhook_time = match chrono::DateTime::parse_from_rfc3339(timestamp) {
        Ok(dt) => dt.timestamp() as u64,
        Err(_) => return false,
    };

    // Get current time
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Check if timestamp is within tolerance
    now.saturating_sub(webhook_time) <= tolerance_seconds
}

// === Helper Functions ===

/// Convert client webhook event to string
pub fn client_event_to_string(event: &ClientWebhookEvent) -> &'static str {
    match event {
        ClientWebhookEvent::InvoiceCreated => "invoiceCreated",
        ClientWebhookEvent::InvoicePending => "invoicePending",
        ClientWebhookEvent::InvoicePaid => "invoicePaid",
        ClientWebhookEvent::InvoiceCompleted => "invoiceCompleted",
        ClientWebhookEvent::InvoiceCancelled => "invoiceCancelled",
        ClientWebhookEvent::InvoiceTimedOut => "invoiceTimedOut",
        ClientWebhookEvent::PaymentCreated => "paymentCreated",
        ClientWebhookEvent::PaymentTimedOut => "paymentTimedOut",
    }
}

/// Convert wallet webhook event to string
pub fn wallet_event_to_string(event: &WalletWebhookEvent) -> &'static str {
    match event {
        WalletWebhookEvent::InternalReceive => "internalReceive",
        WalletWebhookEvent::UtxoExternalReceive => "utxoExternalReceive",
        WalletWebhookEvent::AccountBasedExternalReceive => "accountBasedExternalReceive",
        WalletWebhookEvent::InternalSpend => "internalSpend",
        WalletWebhookEvent::ExternalSpend => "externalSpend",
        WalletWebhookEvent::SameUserReceive => "sameUserReceive",
        WalletWebhookEvent::AccountBasedExternalTokenReceive => "accountBasedExternalTokenReceive",
        WalletWebhookEvent::AccountBasedTokenSpend => "accountBasedTokenSpend",
    }
}

/// Filter events by type
pub fn filter_client_events_by_type(
    events: &[ClientWebhookEvent],
    event_type: ClientWebhookEvent,
) -> Vec<&ClientWebhookEvent> {
    events
        .iter()
        .filter(|&event| *event == event_type)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_webhook_signature() {
        let private_key = "test_private_key";
        let headers = WebhookHeaders {
            client_id: "client_123".to_string(),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            signature: "".to_string(), // Will be calculated
        };
        let payload = b"test payload";

        // First calculate what the signature should be
        use hmac::{Hmac, Mac};
        use sha2::Sha512;

        let mut data_to_sign = Vec::new();
        data_to_sign.extend_from_slice(headers.client_id.as_bytes());
        data_to_sign.extend_from_slice(headers.timestamp.as_bytes());
        data_to_sign.extend_from_slice(payload);

        let mut mac = Hmac::<Sha512>::new_from_slice(private_key.as_bytes()).unwrap();
        mac.update(&data_to_sign);
        let expected_signature = hex::encode(mac.finalize().into_bytes());

        let headers_with_sig = WebhookHeaders {
            signature: expected_signature,
            ..headers
        };

        assert!(verify_webhook_signature(
            private_key,
            &headers_with_sig,
            payload
        ));
    }

    #[test]
    fn test_is_webhook_timestamp_valid() {
        // Test with current time (should be valid)
        let now = chrono::Utc::now();
        let timestamp = now.to_rfc3339();
        assert!(is_webhook_timestamp_valid(&timestamp, 300));

        // Test with old timestamp (should be invalid)
        let old_time = now - chrono::Duration::seconds(600); // 10 minutes ago
        let old_timestamp = old_time.to_rfc3339();
        assert!(!is_webhook_timestamp_valid(&old_timestamp, 300));
    }

    #[test]
    fn test_client_event_to_string() {
        assert_eq!(
            client_event_to_string(&ClientWebhookEvent::InvoiceCreated),
            "invoiceCreated"
        );
        assert_eq!(
            client_event_to_string(&ClientWebhookEvent::InvoiceCompleted),
            "invoiceCompleted"
        );
    }

    #[test]
    fn test_wallet_event_to_string() {
        assert_eq!(
            wallet_event_to_string(&WalletWebhookEvent::UtxoExternalReceive),
            "utxoExternalReceive"
        );
        assert_eq!(
            wallet_event_to_string(&WalletWebhookEvent::ExternalSpend),
            "externalSpend"
        );
    }

    #[test]
    fn test_create_client_webhook_request_builder() {
        let request = CreateClientWebhookRequest::new("https://example.com/webhook")
            .with_events(vec![
                ClientWebhookEvent::InvoiceCreated,
                ClientWebhookEvent::InvoiceCompleted,
            ])
            .with_secret("webhook_secret")
            .active(true);

        assert_eq!(request.url, "https://example.com/webhook");
        assert_eq!(request.events.len(), 2);
        assert_eq!(request.secret, Some("webhook_secret".to_string()));
        assert_eq!(request.is_active, Some(true));
    }

    #[test]
    fn test_parse_webhook_headers() {
        let mut headers = HashMap::new();
        headers.insert(
            "X-CoinPayments-Client".to_string(),
            "client_123".to_string(),
        );
        headers.insert(
            "X-CoinPayments-Timestamp".to_string(),
            "2023-01-01T00:00:00Z".to_string(),
        );
        headers.insert(
            "X-CoinPayments-Signature".to_string(),
            "signature_123".to_string(),
        );

        let webhook_headers = parse_webhook_headers(&headers).unwrap();
        assert_eq!(webhook_headers.client_id, "client_123");
        assert_eq!(webhook_headers.timestamp, "2023-01-01T00:00:00Z");
        assert_eq!(webhook_headers.signature, "signature_123");
    }
}
