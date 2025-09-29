//! Currency-related API endpoints for CoinPayments API
//!
//! This module provides functionality for:
//! - Getting supported currencies and their capabilities
//! - Retrieving merchant currencies
//! - Getting blockchain information
//! - Managing currency conversions and limits

use crate::{CoinPaymentsClient, Result};
use serde::{Deserialize, Serialize};

// === Currency Types ===

/// Currency information from the v2 API
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CurrencyV2 {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub blockchain_id: Option<String>,
    pub smart_contract_address: Option<String>,
    pub decimals: u8,
    pub is_fiat: bool,
    pub status: CurrencyStatus,
    pub capabilities: Vec<CurrencyCapability>,
    pub created_at: String,
    pub updated_at: String,
}

/// Currency status
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CurrencyStatus {
    Active,
    Inactive,
    Maintenance,
}

/// Currency capabilities
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CurrencyCapability {
    Deposit,
    Withdrawal,
    Conversion,
    InvoicePayment,
    WalletCreation,
}

/// Response for getting currencies
#[derive(Debug, Deserialize, Serialize)]
pub struct GetCurrenciesResponse {
    pub currencies: Vec<CurrencyV2>,
    pub pagination: Option<PaginationInfo>,
}

/// Pagination information
#[derive(Debug, Deserialize, Serialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub per_page: u32,
    pub total: u32,
    pub total_pages: u32,
}

/// Merchant currency information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MerchantCurrency {
    pub currency_id: String,
    pub rank: Option<u32>,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Response for getting merchant currencies
#[derive(Debug, Deserialize, Serialize)]
pub struct GetMerchantCurrenciesResponse {
    pub currencies: Vec<MerchantCurrency>,
}

/// Blockchain node information
#[derive(Debug, Deserialize, Serialize)]
pub struct BlockchainNodeInfo {
    pub currency_id: String,
    pub latest_block_number: u64,
    pub synced: bool,
    pub network: String,
}

/// Required confirmations information
#[derive(Debug, Deserialize, Serialize)]
pub struct RequiredConfirmations {
    pub currency_id: String,
    pub confirmations: u32,
    pub network: String,
}

/// Response for required confirmations
#[derive(Debug, Deserialize, Serialize)]
pub struct GetRequiredConfirmationsResponse {
    pub confirmations: Vec<RequiredConfirmations>,
}

/// Currency conversion information
#[derive(Debug, Deserialize, Serialize)]
pub struct CurrencyConversion {
    pub from_currency_id: String,
    pub to_currency_id: String,
    pub available: bool,
    pub min_amount: Option<String>,
    pub max_amount: Option<String>,
}

/// Response for currency conversions
#[derive(Debug, Deserialize, Serialize)]
pub struct GetCurrencyConversionsResponse {
    pub conversions: Vec<CurrencyConversion>,
}

/// Currency conversion limits
#[derive(Debug, Deserialize, Serialize)]
pub struct CurrencyLimits {
    pub from_currency_id: String,
    pub to_currency_id: String,
    pub min_amount: String,
    pub max_amount: String,
    pub daily_limit: Option<String>,
    pub monthly_limit: Option<String>,
}

impl CoinPaymentsClient {
    /// Get list of supported currencies
    ///
    /// # Arguments
    /// * `page` - Page number for pagination (optional)
    /// * `per_page` - Number of results per page (optional)
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let currencies = client.get_currencies(None, None).await?;
    /// ```
    pub async fn get_currencies(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<GetCurrenciesResponse> {
        let mut query_params = Vec::new();

        if let Some(page) = page {
            query_params.push(("page", page.to_string()));
        }
        if let Some(per_page) = per_page {
            query_params.push(("per_page", per_page.to_string()));
        }

        self.get_request("v2/currencies", &query_params).await
    }

    /// Get currency by ID
    ///
    /// # Arguments
    /// * `currency_id` - The currency ID to retrieve
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let currency = client.get_currency_by_id("4").await?; // Bitcoin
    /// ```
    pub async fn get_currency_by_id(&self, currency_id: &str) -> Result<CurrencyV2> {
        let endpoint = format!("v2/currencies/{}", currency_id);
        self.get_request(&endpoint, &[]).await
    }

    /// Get merchant's currently accepted currencies
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let merchant_currencies = client.get_merchant_currencies().await?;
    /// ```
    pub async fn get_merchant_currencies(&self) -> Result<GetMerchantCurrenciesResponse> {
        self.get_request("v1/merchant/currencies", &[]).await
    }

    /// Get latest blockchain block number by currency
    ///
    /// # Arguments
    /// * `currency_id` - The currency ID to get block number for
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let block_info = client.get_latest_block_number("4").await?; // Bitcoin
    /// ```
    pub async fn get_latest_block_number(&self, currency_id: &str) -> Result<BlockchainNodeInfo> {
        let endpoint = format!(
            "v2/currencies/blockchain-nodes/{}/latest-block-number",
            currency_id
        );
        self.get_request(&endpoint, &[]).await
    }

    /// Get required confirmations for each currency
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let confirmations = client.get_required_confirmations().await?;
    /// ```
    pub async fn get_required_confirmations(&self) -> Result<GetRequiredConfirmationsResponse> {
        self.get_request("v2/currencies/required-confirmations", &[])
            .await
    }

    /// Get list of all possible currency conversions
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let conversions = client.get_currency_conversions().await?;
    /// ```
    pub async fn get_currency_conversions(&self) -> Result<GetCurrencyConversionsResponse> {
        self.get_request("v2/currencies/conversions", &[]).await
    }

    /// Get conversion limits by currency pair
    ///
    /// # Arguments
    /// * `from_currency` - Source currency ID
    /// * `to_currency` - Target currency ID
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let limits = client.get_currency_limits("4", "3").await?; // BTC to LTC
    /// ```
    pub async fn get_currency_limits(
        &self,
        from_currency: &str,
        to_currency: &str,
    ) -> Result<CurrencyLimits> {
        let endpoint = format!("v2/currencies/limits/{}/{}", from_currency, to_currency);
        self.get_request(&endpoint, &[]).await
    }
}

// === Helper Functions ===

/// Check if a currency supports a specific capability
pub fn currency_supports_capability(
    currency: &CurrencyV2,
    capability: &CurrencyCapability,
) -> bool {
    currency.capabilities.contains(capability)
}

/// Filter currencies by status
pub fn filter_currencies_by_status(
    currencies: &[CurrencyV2],
    status: CurrencyStatus,
) -> Vec<&CurrencyV2> {
    currencies
        .iter()
        .filter(|currency| currency.status == status)
        .collect()
}

/// Get currencies by capability
pub fn get_currencies_with_capability(
    currencies: &[CurrencyV2],
    capability: CurrencyCapability,
) -> Vec<&CurrencyV2> {
    currencies
        .iter()
        .filter(|currency| currency_supports_capability(currency, &capability))
        .collect()
}

/// Parse token currency ID to get base currency and contract address
pub fn parse_token_currency_id(currency_id: &str) -> Option<(String, String)> {
    if let Some((base_id, contract_address)) = currency_id.split_once(':') {
        Some((base_id.to_string(), contract_address.to_string()))
    } else {
        None
    }
}

/// Check if currency is a token (has smart contract address)
pub fn is_token_currency(currency: &CurrencyV2) -> bool {
    currency.smart_contract_address.is_some()
}

/// Get base currency ID for tokens
pub fn get_base_currency_id(currency_id: &str) -> String {
    if let Some((base_id, _)) = parse_token_currency_id(currency_id) {
        base_id
    } else {
        currency_id.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_token_currency_id() {
        // Test valid token ID
        let token_id = "4:0xdac17f958d2ee523a2206206994597c13d831ec7";
        let result = parse_token_currency_id(token_id);
        assert!(result.is_some());
        let (base_id, contract) = result.unwrap();
        assert_eq!(base_id, "4");
        assert_eq!(contract, "0xdac17f958d2ee523a2206206994597c13d831ec7");

        // Test non-token ID
        let coin_id = "4";
        let result = parse_token_currency_id(coin_id);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_base_currency_id() {
        assert_eq!(
            get_base_currency_id("4:0xdac17f958d2ee523a2206206994597c13d831ec7"),
            "4"
        );
        assert_eq!(get_base_currency_id("4"), "4");
    }

    #[test]
    fn test_currency_supports_capability() {
        let currency = CurrencyV2 {
            id: "4".to_string(),
            name: "Bitcoin".to_string(),
            symbol: "BTC".to_string(),
            blockchain_id: None,
            smart_contract_address: None,
            decimals: 8,
            is_fiat: false,
            status: CurrencyStatus::Active,
            capabilities: vec![CurrencyCapability::Deposit, CurrencyCapability::Withdrawal],
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };

        assert!(currency_supports_capability(
            &currency,
            &CurrencyCapability::Deposit
        ));
        assert!(currency_supports_capability(
            &currency,
            &CurrencyCapability::Withdrawal
        ));
        assert!(!currency_supports_capability(
            &currency,
            &CurrencyCapability::Conversion
        ));
    }

    #[test]
    fn test_filter_currencies_by_status() {
        let currencies = vec![
            CurrencyV2 {
                id: "1".to_string(),
                name: "Active Coin".to_string(),
                symbol: "AC".to_string(),
                blockchain_id: None,
                smart_contract_address: None,
                decimals: 8,
                is_fiat: false,
                status: CurrencyStatus::Active,
                capabilities: vec![],
                created_at: "2023-01-01T00:00:00Z".to_string(),
                updated_at: "2023-01-01T00:00:00Z".to_string(),
            },
            CurrencyV2 {
                id: "2".to_string(),
                name: "Inactive Coin".to_string(),
                symbol: "IC".to_string(),
                blockchain_id: None,
                smart_contract_address: None,
                decimals: 8,
                is_fiat: false,
                status: CurrencyStatus::Inactive,
                capabilities: vec![],
                created_at: "2023-01-01T00:00:00Z".to_string(),
                updated_at: "2023-01-01T00:00:00Z".to_string(),
            },
        ];

        let active_currencies = filter_currencies_by_status(&currencies, CurrencyStatus::Active);
        assert_eq!(active_currencies.len(), 1);
        assert_eq!(active_currencies[0].id, "1");
    }
}
