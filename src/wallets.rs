//! Wallet management API endpoints for CoinPayments API
//!
//! This module provides functionality for:
//! - Creating and managing wallets
//! - Managing wallet addresses (temporary and permanent)
//! - Wallet operations and information retrieval

use crate::{CoinPaymentsClient, Result};
use serde::{Deserialize, Serialize};

// === Wallet Types ===

/// Wallet information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Wallet {
    pub id: String,
    pub label: String,
    pub currency_id: String,
    pub currency_symbol: String,
    pub balance: String,
    pub balance_f: f64,
    pub available_balance: String,
    pub available_balance_f: f64,
    pub pending_balance: String,
    pub pending_balance_f: f64,
    pub address_type: AddressType,
    pub status: WalletStatus,
    pub created_at: String,
    pub updated_at: String,
}

/// Wallet status
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WalletStatus {
    Active,
    Inactive,
    Frozen,
    Closed,
}

/// Address type for wallets
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AddressType {
    /// Temporary addresses (recycled after use)
    Temporary,
    /// Permanent addresses (persistent)
    Permanent,
}

/// Wallet address information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WalletAddress {
    pub id: String,
    pub label: String,
    pub address: String,
    pub wallet_id: String,
    pub currency_id: String,
    pub address_type: AddressType,
    pub balance: String,
    pub balance_f: f64,
    pub is_activated: bool,
    pub webhook_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Request to create or retrieve a wallet
#[derive(Debug, Serialize, Clone)]
pub struct CreateWalletRequest {
    pub label: String,
    pub currency_id: String,
    pub use_permanent_addresses: Option<bool>,
    pub webhook_url: Option<String>,
    pub auto_create_address: Option<bool>,
}

/// Response for wallet operations
#[derive(Debug, Deserialize, Serialize)]
pub struct WalletResponse {
    pub wallet: Wallet,
    pub addresses: Option<Vec<WalletAddress>>,
}

/// Response for getting wallets
#[derive(Debug, Deserialize, Serialize)]
pub struct GetWalletsResponse {
    pub wallets: Vec<Wallet>,
    pub pagination: Option<WalletPaginationInfo>,
}

/// Pagination information for wallets
#[derive(Debug, Deserialize, Serialize)]
pub struct WalletPaginationInfo {
    pub page: u32,
    pub per_page: u32,
    pub total: u32,
    pub total_pages: u32,
}

/// Wallet count response
#[derive(Debug, Deserialize, Serialize)]
pub struct WalletCountResponse {
    pub count: u32,
    pub active_count: u32,
    pub inactive_count: u32,
}

/// Response for getting wallet addresses
#[derive(Debug, Deserialize, Serialize)]
pub struct GetAddressesResponse {
    pub addresses: Vec<WalletAddress>,
    pub pagination: Option<AddressPaginationInfo>,
}

/// Pagination information for addresses
#[derive(Debug, Deserialize, Serialize)]
pub struct AddressPaginationInfo {
    pub page: u32,
    pub per_page: u32,
    pub total: u32,
    pub total_pages: u32,
}

/// Address count response
#[derive(Debug, Deserialize, Serialize)]
pub struct AddressCountResponse {
    pub count: u32,
    pub activated_count: u32,
    pub unactivated_count: u32,
}

/// Webhook configuration for wallet/address
#[derive(Debug, Serialize, Clone)]
pub struct WebhookConfig {
    pub url: String,
    pub events: Vec<WebhookEvent>,
    pub secret: Option<String>,
}

/// Webhook events for wallets/addresses
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum WebhookEvent {
    InternalReceive,
    UtxoExternalReceive,
    AccountBasedExternalReceive,
    InternalSpend,
    ExternalSpend,
    SameUserReceive,
    AccountBasedExternalTokenReceive,
    AccountBasedTokenSpend,
}

impl Default for CreateWalletRequest {
    fn default() -> Self {
        Self {
            label: String::new(),
            currency_id: String::new(),
            use_permanent_addresses: Some(false),
            webhook_url: None,
            auto_create_address: Some(true),
        }
    }
}

impl CreateWalletRequest {
    /// Create a new wallet request
    pub fn new(label: impl Into<String>, currency_id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            currency_id: currency_id.into(),
            ..Default::default()
        }
    }

    /// Set whether to use permanent addresses
    pub fn with_permanent_addresses(mut self, permanent: bool) -> Self {
        self.use_permanent_addresses = Some(permanent);
        self
    }

    /// Set webhook URL for wallet events
    pub fn with_webhook(mut self, url: impl Into<String>) -> Self {
        self.webhook_url = Some(url.into());
        self
    }

    /// Set whether to auto-create an address
    pub fn with_auto_create_address(mut self, auto_create: bool) -> Self {
        self.auto_create_address = Some(auto_create);
        self
    }
}

impl CoinPaymentsClient {
    /// Get list of merchant wallets
    ///
    /// # Arguments
    /// * `page` - Page number for pagination (optional)
    /// * `per_page` - Number of results per page (optional)
    /// * `currency_id` - Filter by currency ID (optional)
    /// * `status` - Filter by wallet status (optional)
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let wallets = client.get_wallets(None, None, None, None).await?;
    /// ```
    pub async fn get_wallets(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
        currency_id: Option<&str>,
        status: Option<WalletStatus>,
    ) -> Result<GetWalletsResponse> {
        let mut query_params = Vec::new();

        if let Some(page) = page {
            query_params.push(("page", page.to_string()));
        }
        if let Some(per_page) = per_page {
            query_params.push(("per_page", per_page.to_string()));
        }
        if let Some(currency_id) = currency_id {
            query_params.push(("currency_id", currency_id.to_string()));
        }
        if let Some(status) = status {
            let status_str = match status {
                WalletStatus::Active => "active",
                WalletStatus::Inactive => "inactive",
                WalletStatus::Frozen => "frozen",
                WalletStatus::Closed => "closed",
            };
            query_params.push(("status", status_str.to_string()));
        }

        self.get_request("v3/merchant/wallets", &query_params).await
    }

    /// Create or retrieve a wallet by external IDs
    ///
    /// # Arguments
    /// * `request` - Wallet creation request
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let request = CreateWalletRequest::new("my-btc-wallet", "4")
    ///     .with_permanent_addresses(true);
    /// let wallet = client.create_wallet(request).await?;
    /// ```
    pub async fn create_wallet(&self, request: CreateWalletRequest) -> Result<WalletResponse> {
        self.put_request("v3/merchant/wallets", &request).await
    }

    /// Get wallet count
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let count = client.get_wallet_count().await?;
    /// ```
    pub async fn get_wallet_count(&self) -> Result<WalletCountResponse> {
        self.get_request("v3/merchant/wallets/count", &[]).await
    }

    /// Get wallet addresses
    ///
    /// # Arguments
    /// * `wallet_label` - Wallet label
    /// * `currency_id` - Currency ID
    /// * `page` - Page number (optional)
    /// * `per_page` - Results per page (optional)
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let addresses = client.get_wallet_addresses("my-btc-wallet", "4", None, None).await?;
    /// ```
    pub async fn get_wallet_addresses(
        &self,
        wallet_label: &str,
        currency_id: &str,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<GetAddressesResponse> {
        let mut query_params = Vec::new();

        if let Some(page) = page {
            query_params.push(("page", page.to_string()));
        }
        if let Some(per_page) = per_page {
            query_params.push(("per_page", per_page.to_string()));
        }

        let endpoint = format!(
            "v3/merchant/wallets/{}/{}/addresses",
            wallet_label, currency_id
        );
        self.get_request(&endpoint, &query_params).await
    }

    /// Get wallet address count
    ///
    /// # Arguments
    /// * `wallet_label` - Wallet label
    /// * `currency_id` - Currency ID
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let count = client.get_wallet_address_count("my-btc-wallet", "4").await?;
    /// ```
    pub async fn get_wallet_address_count(
        &self,
        wallet_label: &str,
        currency_id: &str,
    ) -> Result<AddressCountResponse> {
        let endpoint = format!(
            "v3/merchant/wallets/{}/{}/addresses/count",
            wallet_label, currency_id
        );
        self.get_request(&endpoint, &[]).await
    }

    /// Get specific address by label
    ///
    /// # Arguments
    /// * `wallet_label` - Wallet label
    /// * `currency_id` - Currency ID
    /// * `address_label` - Address label
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let address = client.get_address_by_label("my-btc-wallet", "4", "address-1").await?;
    /// ```
    pub async fn get_address_by_label(
        &self,
        wallet_label: &str,
        currency_id: &str,
        address_label: &str,
    ) -> Result<WalletAddress> {
        let endpoint = format!(
            "v3/merchant/wallets/{}/{}/addresses/{}",
            wallet_label, currency_id, address_label
        );
        self.get_request(&endpoint, &[]).await
    }

    /// Update wallet webhook
    ///
    /// # Arguments
    /// * `wallet_label` - Wallet label
    /// * `currency_id` - Currency ID
    /// * `webhook_config` - Webhook configuration
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let webhook = WebhookConfig {
    ///     url: "https://example.com/webhook".to_string(),
    ///     events: vec![WebhookEvent::ExternalReceive],
    ///     secret: Some("webhook_secret".to_string()),
    /// };
    /// client.update_wallet_webhook("my-btc-wallet", "4", webhook).await?;
    /// ```
    pub async fn update_wallet_webhook(
        &self,
        wallet_label: &str,
        currency_id: &str,
        webhook_config: WebhookConfig,
    ) -> Result<()> {
        let endpoint = format!(
            "v3/merchant/wallets/{}/{}/webhook",
            wallet_label, currency_id
        );
        self.put_request(&endpoint, &webhook_config).await
    }

    /// Update address webhook
    ///
    /// # Arguments
    /// * `wallet_label` - Wallet label
    /// * `currency_id` - Currency ID
    /// * `address_label` - Address label
    /// * `webhook_config` - Webhook configuration
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let webhook = WebhookConfig {
    ///     url: "https://example.com/webhook".to_string(),
    ///     events: vec![WebhookEvent::ExternalReceive],
    ///     secret: Some("webhook_secret".to_string()),
    /// };
    /// client.update_address_webhook("my-btc-wallet", "4", "address-1", webhook).await?;
    /// ```
    pub async fn update_address_webhook(
        &self,
        wallet_label: &str,
        currency_id: &str,
        address_label: &str,
        webhook_config: WebhookConfig,
    ) -> Result<()> {
        let endpoint = format!(
            "v3/merchant/wallets/{}/{}/addresses/{}/webhook",
            wallet_label, currency_id, address_label
        );
        self.put_request(&endpoint, &webhook_config).await
    }
}

// === Helper Functions ===

/// Check if wallet has sufficient balance for amount
pub fn has_sufficient_balance(wallet: &Wallet, amount: f64) -> bool {
    wallet.available_balance_f >= amount
}

/// Calculate total wallet value across multiple wallets
pub fn calculate_total_wallet_value(wallets: &[Wallet]) -> f64 {
    wallets.iter().map(|w| w.balance_f).sum()
}

/// Filter wallets by status
pub fn filter_wallets_by_status(wallets: &[Wallet], status: WalletStatus) -> Vec<&Wallet> {
    wallets
        .iter()
        .filter(|wallet| wallet.status == status)
        .collect()
}

/// Filter wallets by currency
pub fn filter_wallets_by_currency<'a>(wallets: &'a [Wallet], currency_id: &str) -> Vec<&'a Wallet> {
    wallets
        .iter()
        .filter(|wallet| wallet.currency_id == currency_id)
        .collect()
}

/// Get wallet by label
pub fn find_wallet_by_label<'a>(wallets: &'a [Wallet], label: &str) -> Option<&'a Wallet> {
    wallets.iter().find(|wallet| wallet.label == label)
}

/// Check if address is activated
pub fn is_address_activated(address: &WalletAddress) -> bool {
    address.is_activated
}

/// Get addresses with balance
pub fn get_addresses_with_balance(addresses: &[WalletAddress]) -> Vec<&WalletAddress> {
    addresses
        .iter()
        .filter(|addr| addr.balance_f > 0.0)
        .collect()
}

/// Calculate total balance across addresses
pub fn calculate_total_address_balance(addresses: &[WalletAddress]) -> f64 {
    addresses.iter().map(|addr| addr.balance_f).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_wallet(label: &str, currency_id: &str, balance: f64) -> Wallet {
        Wallet {
            id: "wallet_123".to_string(),
            label: label.to_string(),
            currency_id: currency_id.to_string(),
            currency_symbol: "BTC".to_string(),
            balance: balance.to_string(),
            balance_f: balance,
            available_balance: balance.to_string(),
            available_balance_f: balance,
            pending_balance: "0".to_string(),
            pending_balance_f: 0.0,
            address_type: AddressType::Temporary,
            status: WalletStatus::Active,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        }
    }

    fn create_test_address(label: &str, balance: f64, activated: bool) -> WalletAddress {
        WalletAddress {
            id: "addr_123".to_string(),
            label: label.to_string(),
            address: "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string(),
            wallet_id: "wallet_123".to_string(),
            currency_id: "4".to_string(),
            address_type: AddressType::Permanent,
            balance: balance.to_string(),
            balance_f: balance,
            is_activated: activated,
            webhook_url: None,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_has_sufficient_balance() {
        let wallet = create_test_wallet("test", "4", 0.01);
        assert!(has_sufficient_balance(&wallet, 0.005));
        assert!(!has_sufficient_balance(&wallet, 0.02));
    }

    #[test]
    fn test_calculate_total_wallet_value() {
        let wallets = vec![
            create_test_wallet("wallet1", "4", 0.01),
            create_test_wallet("wallet2", "61", 0.5),
            create_test_wallet("wallet3", "3", 0.1),
        ];

        let total = calculate_total_wallet_value(&wallets);
        assert_eq!(total, 0.61);
    }

    #[test]
    fn test_filter_wallets_by_status() {
        let mut wallets = vec![
            create_test_wallet("active1", "4", 0.01),
            create_test_wallet("active2", "61", 0.5),
        ];
        wallets[1].status = WalletStatus::Inactive;

        let active_wallets = filter_wallets_by_status(&wallets, WalletStatus::Active);
        assert_eq!(active_wallets.len(), 1);
        assert_eq!(active_wallets[0].label, "active1");
    }

    #[test]
    fn test_find_wallet_by_label() {
        let wallets = vec![
            create_test_wallet("wallet1", "4", 0.01),
            create_test_wallet("wallet2", "61", 0.5),
        ];

        let found = find_wallet_by_label(&wallets, "wallet2");
        assert!(found.is_some());
        assert_eq!(found.unwrap().currency_id, "61");

        let not_found = find_wallet_by_label(&wallets, "nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_is_address_activated() {
        let activated = create_test_address("addr1", 0.001, true);
        let not_activated = create_test_address("addr2", 0.001, false);

        assert!(is_address_activated(&activated));
        assert!(!is_address_activated(&not_activated));
    }

    #[test]
    fn test_get_addresses_with_balance() {
        let addresses = vec![
            create_test_address("addr1", 0.001, true),
            create_test_address("addr2", 0.0, true),
            create_test_address("addr3", 0.005, false),
        ];

        let with_balance = get_addresses_with_balance(&addresses);
        assert_eq!(with_balance.len(), 2);
    }

    #[test]
    fn test_calculate_total_address_balance() {
        let addresses = vec![
            create_test_address("addr1", 0.001, true),
            create_test_address("addr2", 0.005, true),
            create_test_address("addr3", 0.002, false),
        ];

        let total = calculate_total_address_balance(&addresses);
        assert_eq!(total, 0.008);
    }

    #[test]
    fn test_create_wallet_request_builder() {
        let request = CreateWalletRequest::new("my-wallet", "4")
            .with_permanent_addresses(true)
            .with_webhook("https://example.com/webhook")
            .with_auto_create_address(false);

        assert_eq!(request.label, "my-wallet");
        assert_eq!(request.currency_id, "4");
        assert_eq!(request.use_permanent_addresses, Some(true));
        assert_eq!(
            request.webhook_url,
            Some("https://example.com/webhook".to_string())
        );
        assert_eq!(request.auto_create_address, Some(false));
    }
}
