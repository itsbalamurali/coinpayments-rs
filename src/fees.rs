//! Blockchain fees API endpoints for CoinPayments API
//!
//! This module provides functionality for:
//! - Calculating blockchain fees for transactions
//! - Getting fee estimates for different transaction types
//! - Fee optimization and recommendations

use crate::{CoinPaymentsClient, Result};
use serde::{Deserialize, Serialize};

// === Fee Types ===

/// Blockchain fee information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BlockchainFee {
    pub currency_id: String,
    pub fee_type: FeeType,
    pub amount: String,
    pub amount_f: f64,
    pub currency_symbol: String,
    pub estimated_confirmation_time: Option<u32>, // in minutes
    pub priority_level: FeePriority,
}

/// Fee type enumeration
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FeeType {
    /// Fixed fee amount
    Fixed,
    /// Percentage-based fee
    Percentage,
    /// Dynamic fee based on network conditions
    Dynamic,
    /// Gas-based fee (for EVM chains)
    Gas,
}

/// Fee priority levels
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FeePriority {
    /// Lowest fee, longest confirmation time
    Slow,
    /// Standard fee and confirmation time
    Standard,
    /// Higher fee, faster confirmation
    Fast,
    /// Highest fee, fastest confirmation
    Priority,
}

/// Fee calculation request
#[derive(Debug, Serialize, Clone)]
pub struct FeeCalculationRequest {
    pub currency_id: String,
    pub transaction_type: TransactionType,
    pub amount: Option<String>,
    pub priority: Option<FeePriority>,
    pub recipient_count: Option<u32>,
}

/// Transaction types for fee calculation
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TransactionType {
    /// Standard send transaction
    Send,
    /// Withdrawal transaction
    Withdrawal,
    /// Currency conversion
    Conversion,
    /// Wallet consolidation
    Consolidation,
    /// Smart contract interaction
    Contract,
}

/// Fee calculation response
#[derive(Debug, Deserialize, Serialize)]
pub struct FeeCalculationResponse {
    pub currency_id: String,
    pub transaction_type: TransactionType,
    pub fees: Vec<BlockchainFee>,
    pub recommended_fee: BlockchainFee,
    pub network_status: NetworkStatus,
}

/// Network status information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NetworkStatus {
    pub currency_id: String,
    pub congestion_level: CongestionLevel,
    pub average_confirmation_time: u32, // in minutes
    pub mempool_size: Option<u64>,
    pub last_updated: String,
}

/// Network congestion levels
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CongestionLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Gas fee information for EVM chains
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GasFee {
    pub currency_id: String,
    pub gas_price: String,
    pub gas_limit: u64,
    pub base_fee: Option<String>,
    pub priority_fee: Option<String>,
    pub max_fee: Option<String>,
    pub estimated_cost: String,
}

impl Default for FeeCalculationRequest {
    fn default() -> Self {
        Self {
            currency_id: String::new(),
            transaction_type: TransactionType::Send,
            amount: None,
            priority: Some(FeePriority::Standard),
            recipient_count: Some(1),
        }
    }
}

impl FeeCalculationRequest {
    /// Create a new fee calculation request
    pub fn new(currency_id: impl Into<String>, transaction_type: TransactionType) -> Self {
        Self {
            currency_id: currency_id.into(),
            transaction_type,
            ..Default::default()
        }
    }

    /// Set the transaction amount
    pub fn with_amount(mut self, amount: impl Into<String>) -> Self {
        self.amount = Some(amount.into());
        self
    }

    /// Set the fee priority
    pub fn with_priority(mut self, priority: FeePriority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Set the number of recipients
    pub fn with_recipient_count(mut self, count: u32) -> Self {
        self.recipient_count = Some(count);
        self
    }
}

impl CoinPaymentsClient {
    /// Calculate blockchain fee for a transaction
    ///
    /// # Arguments
    /// * `currency_id` - The currency ID to calculate fees for
    /// * `request` - Optional fee calculation parameters
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    ///
    /// // Calculate basic send fee for Bitcoin
    /// let fee = client.calculate_blockchain_fee("4", None).await?;
    ///
    /// // Calculate fee with specific parameters
    /// let request = FeeCalculationRequest::new("4", TransactionType::Send)
    ///     .with_amount("0.001")
    ///     .with_priority(FeePriority::Fast);
    /// let fee = client.calculate_blockchain_fee("4", Some(request)).await?;
    /// ```
    pub async fn calculate_blockchain_fee(
        &self,
        currency_id: &str,
        request: Option<FeeCalculationRequest>,
    ) -> Result<FeeCalculationResponse> {
        let endpoint = format!("v2/fees/blockchain/{}", currency_id);

        if let Some(request) = request {
            self.post_request(&endpoint, &request).await
        } else {
            let default_request = FeeCalculationRequest::new(currency_id, TransactionType::Send);
            self.post_request(&endpoint, &default_request).await
        }
    }

    /// Get gas fee information for EVM-based currencies
    ///
    /// # Arguments
    /// * `currency_id` - The EVM currency ID (e.g., Ethereum, BSC)
    /// * `gas_limit` - Optional gas limit override
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let gas_fee = client.get_gas_fee("61", Some(21000)).await?; // Ethereum
    /// ```
    pub async fn get_gas_fee(&self, currency_id: &str, gas_limit: Option<u64>) -> Result<GasFee> {
        let endpoint = format!("v2/fees/gas/{}", currency_id);
        let mut query_params = Vec::new();

        if let Some(limit) = gas_limit {
            query_params.push(("gas_limit", limit.to_string()));
        }

        self.get_request(&endpoint, &query_params).await
    }

    /// Get network status for fee estimation
    ///
    /// # Arguments
    /// * `currency_id` - The currency ID to get network status for
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let status = client.get_network_status("4").await?; // Bitcoin
    /// ```
    pub async fn get_network_status(&self, currency_id: &str) -> Result<NetworkStatus> {
        let endpoint = format!("v2/fees/network-status/{}", currency_id);
        self.get_request(&endpoint, &[]).await
    }

    /// Get recommended fee for optimal confirmation time
    ///
    /// # Arguments
    /// * `currency_id` - The currency ID
    /// * `target_confirmation_time` - Desired confirmation time in minutes
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// // Get fee for 30-minute confirmation
    /// let fee = client.get_recommended_fee("4", 30).await?;
    /// ```
    pub async fn get_recommended_fee(
        &self,
        currency_id: &str,
        target_confirmation_time: u32,
    ) -> Result<BlockchainFee> {
        let request = FeeCalculationRequest::new(currency_id, TransactionType::Send);
        let response = self
            .calculate_blockchain_fee(currency_id, Some(request))
            .await?;

        // Find the fee that best matches the target confirmation time
        response
            .fees
            .into_iter()
            .min_by_key(|fee| {
                let est_time = fee.estimated_confirmation_time.unwrap_or(u32::MAX);
                (est_time as i32 - target_confirmation_time as i32).abs()
            })
            .ok_or_else(|| crate::CoinPaymentsError::Api {
                message: "No suitable fee found for target confirmation time".to_string(),
            })
    }
}

// === Helper Functions ===

/// Calculate total transaction cost (amount + fee)
pub fn calculate_total_cost(amount: f64, fee: &BlockchainFee) -> f64 {
    amount + fee.amount_f
}

/// Compare fees by priority level
pub fn compare_fee_priority(a: &BlockchainFee, b: &BlockchainFee) -> std::cmp::Ordering {
    use FeePriority::*;
    let priority_order = |p: &FeePriority| match p {
        Slow => 1,
        Standard => 2,
        Fast => 3,
        Priority => 4,
    };

    priority_order(&a.priority_level).cmp(&priority_order(&b.priority_level))
}

/// Get the cheapest fee option
pub fn get_cheapest_fee(fees: &[BlockchainFee]) -> Option<&BlockchainFee> {
    fees.iter().min_by(|a, b| {
        a.amount_f
            .partial_cmp(&b.amount_f)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

/// Get the fastest fee option
pub fn get_fastest_fee(fees: &[BlockchainFee]) -> Option<&BlockchainFee> {
    fees.iter()
        .min_by_key(|fee| fee.estimated_confirmation_time.unwrap_or(u32::MAX))
}

/// Check if network is congested based on status
pub fn is_network_congested(status: &NetworkStatus) -> bool {
    matches!(
        status.congestion_level,
        CongestionLevel::High | CongestionLevel::Critical
    )
}

/// Estimate fee for multiple recipients
pub fn estimate_multi_recipient_fee(base_fee: &BlockchainFee, recipient_count: u32) -> f64 {
    match base_fee.fee_type {
        FeeType::Fixed => base_fee.amount_f * recipient_count as f64,
        FeeType::Percentage => base_fee.amount_f, // Percentage doesn't scale with recipients
        FeeType::Dynamic | FeeType::Gas => {
            // Approximate scaling for dynamic/gas fees
            base_fee.amount_f * (1.0 + (recipient_count - 1) as f64 * 0.3)
        }
    }
}

/// Convert gas price to different units (gwei, wei)
pub fn convert_gas_price(gas_price_gwei: f64, to_unit: GasUnit) -> f64 {
    match to_unit {
        GasUnit::Wei => gas_price_gwei * 1_000_000_000.0,
        GasUnit::Gwei => gas_price_gwei,
        GasUnit::Ether => gas_price_gwei / 1_000_000_000.0,
    }
}

/// Gas price units
#[derive(Debug, Clone, PartialEq)]
pub enum GasUnit {
    Wei,
    Gwei,
    Ether,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_fee(
        priority: FeePriority,
        amount: f64,
        confirmation_time: Option<u32>,
    ) -> BlockchainFee {
        BlockchainFee {
            currency_id: "4".to_string(),
            fee_type: FeeType::Dynamic,
            amount: amount.to_string(),
            amount_f: amount,
            currency_symbol: "BTC".to_string(),
            estimated_confirmation_time: confirmation_time,
            priority_level: priority,
        }
    }

    #[test]
    fn test_calculate_total_cost() {
        let fee = create_test_fee(FeePriority::Standard, 0.0001, Some(30));
        let total = calculate_total_cost(0.01, &fee);
        assert_eq!(total, 0.0101);
    }

    #[test]
    fn test_get_cheapest_fee() {
        let fees = vec![
            create_test_fee(FeePriority::Fast, 0.0002, Some(15)),
            create_test_fee(FeePriority::Standard, 0.0001, Some(30)),
            create_test_fee(FeePriority::Slow, 0.00005, Some(60)),
        ];

        let cheapest = get_cheapest_fee(&fees);
        assert!(cheapest.is_some());
        assert_eq!(cheapest.unwrap().amount_f, 0.00005);
    }

    #[test]
    fn test_get_fastest_fee() {
        let fees = vec![
            create_test_fee(FeePriority::Fast, 0.0002, Some(15)),
            create_test_fee(FeePriority::Standard, 0.0001, Some(30)),
            create_test_fee(FeePriority::Slow, 0.00005, Some(60)),
        ];

        let fastest = get_fastest_fee(&fees);
        assert!(fastest.is_some());
        assert_eq!(fastest.unwrap().estimated_confirmation_time, Some(15));
    }

    #[test]
    fn test_is_network_congested() {
        let congested_status = NetworkStatus {
            currency_id: "4".to_string(),
            congestion_level: CongestionLevel::High,
            average_confirmation_time: 60,
            mempool_size: Some(100000),
            last_updated: "2023-01-01T00:00:00Z".to_string(),
        };

        let normal_status = NetworkStatus {
            currency_id: "4".to_string(),
            congestion_level: CongestionLevel::Low,
            average_confirmation_time: 10,
            mempool_size: Some(5000),
            last_updated: "2023-01-01T00:00:00Z".to_string(),
        };

        assert!(is_network_congested(&congested_status));
        assert!(!is_network_congested(&normal_status));
    }

    #[test]
    fn test_estimate_multi_recipient_fee() {
        let fixed_fee = BlockchainFee {
            currency_id: "4".to_string(),
            fee_type: FeeType::Fixed,
            amount: "0.0001".to_string(),
            amount_f: 0.0001,
            currency_symbol: "BTC".to_string(),
            estimated_confirmation_time: Some(30),
            priority_level: FeePriority::Standard,
        };

        let multi_fee = estimate_multi_recipient_fee(&fixed_fee, 3);
        assert_eq!(multi_fee, 0.0003); // 3 recipients × 0.0001
    }

    #[test]
    fn test_convert_gas_price() {
        let gwei_price = 20.0;

        assert_eq!(convert_gas_price(gwei_price, GasUnit::Gwei), 20.0);
        assert_eq!(
            convert_gas_price(gwei_price, GasUnit::Wei),
            20_000_000_000.0
        );
        assert_eq!(convert_gas_price(gwei_price, GasUnit::Ether), 0.00000002);
    }

    #[test]
    fn test_fee_calculation_request_builder() {
        let request = FeeCalculationRequest::new("4", TransactionType::Withdrawal)
            .with_amount("0.01")
            .with_priority(FeePriority::Fast)
            .with_recipient_count(2);

        assert_eq!(request.currency_id, "4");
        assert_eq!(request.transaction_type, TransactionType::Withdrawal);
        assert_eq!(request.amount, Some("0.01".to_string()));
        assert_eq!(request.priority, Some(FeePriority::Fast));
        assert_eq!(request.recipient_count, Some(2));
    }
}
