//! Transaction management API endpoints for CoinPayments API
//!
//! This module provides functionality for:
//! - Managing wallet transactions
//! - Creating spend requests and confirmations
//! - Wallet consolidation operations
//! - Transaction history and information

use crate::{CoinPaymentsClient, Result};
use serde::{Deserialize, Serialize};

// === Transaction Types ===

/// Transaction information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Transaction {
    pub id: String,
    pub wallet_id: String,
    pub currency_id: String,
    pub transaction_type: TransactionType,
    pub amount: String,
    pub amount_f: f64,
    pub fee: Option<String>,
    pub fee_f: Option<f64>,
    pub status: TransactionStatus,
    pub address: Option<String>,
    pub txid: Option<String>,
    pub confirmations: u32,
    pub required_confirmations: u32,
    pub network: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

/// Transaction types
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TransactionType {
    /// Receiving funds from within the CoinPayments system
    InternalReceive,
    /// Receiving UTXO funds from an external source
    UtxoExternalReceive,
    /// Receiving funds from external account-based addresses
    AccountBasedExternalReceive,
    /// Sending funds to an external address
    ExternalSpend,
    /// Sending funds from one CoinPayments user to another
    InternalSpend,
    /// Sending funds from one wallet to another for the same CoinPayments user
    SameUserSpend,
    /// Receiving funds from one wallet to another for the same CoinPayments user
    SameUserReceive,
    /// Receiving tokens from external account-based transfers
    AccountBasedExternalTokenReceive,
    /// Sending account-based tokens to external address
    AccountBasedTokenSpend,
    /// Converting funds between user wallets
    Conversion,
    /// Funds swept automatically to an external wallet by the auto-sweeping feature
    AutoSweeping,
    /// Receiving test funds
    ReceiveTestFundsFromPool,
    /// Returning test funds
    ReturnTestFundsToPool,
    /// Transaction state unknown
    Unknown,
}

/// Transaction statuses
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TransactionStatus {
    /// Initiated withdrawal, tx is not on the blockchain yet
    Created,
    /// Detected tx on the blockchain, tx awaiting required confirmations
    Pending,
    /// Tx validation state is being processed
    Processing,
    /// Tx has been put on chain, can be detected in the blockchain mempool
    Completed,
    /// Tx was not confirmed and has expired
    Expired,
    /// Tx has failed
    Failed,
    /// Tx received all required confirmations on the blockchain
    ConfirmedOnBlockchain,
    /// Awaiting incoming deposit, tx is on chain, awaiting required confirmations
    PendingReceive,
    /// Tx has not received required amount of confirmations
    FailedOnBlockchain,
    /// Tx has been cancelled
    Cancelled,
    /// Tx has been rejected by party
    Rejected,
    /// Tx status unknown
    Unknown,
}

/// Spend request information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpendRequest {
    pub id: String,
    pub wallet_label: String,
    pub currency_id: String,
    pub amount: String,
    pub amount_f: f64,
    pub fee: String,
    pub fee_f: f64,
    pub total_amount: String,
    pub total_amount_f: f64,
    pub destination_address: Option<String>,
    pub destination_currency_id: Option<String>,
    pub note: Option<String>,
    pub status: SpendRequestStatus,
    pub created_at: String,
    pub expires_at: String,
}

/// Spend request status
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SpendRequestStatus {
    Pending,
    Confirmed,
    Cancelled,
    Expired,
    Failed,
}

/// Request to create a spend request
#[derive(Debug, Serialize, Clone)]
pub struct CreateSpendRequest {
    pub amount: String,
    pub destination_address: Option<String>,
    pub destination_currency_id: Option<String>,
    pub note: Option<String>,
    pub auto_confirm: Option<bool>,
}

/// Response for spend request operations
#[derive(Debug, Deserialize, Serialize)]
pub struct SpendRequestResponse {
    pub request: SpendRequest,
    pub preview: SpendPreview,
}

/// Spend preview information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpendPreview {
    pub amount: String,
    pub amount_f: f64,
    pub fee: String,
    pub fee_f: f64,
    pub total: String,
    pub total_f: f64,
    pub exchange_rate: Option<String>,
    pub estimated_confirmation_time: Option<u32>,
}

/// Spend confirmation request
#[derive(Debug, Serialize, Clone)]
pub struct SpendConfirmationRequest {
    pub spend_request_id: String,
}

/// Response for getting transactions
#[derive(Debug, Deserialize, Serialize)]
pub struct GetTransactionsResponse {
    pub transactions: Vec<Transaction>,
    pub pagination: Option<TransactionPaginationInfo>,
}

/// Pagination information for transactions
#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionPaginationInfo {
    pub page: u32,
    pub per_page: u32,
    pub total: u32,
    pub total_pages: u32,
}

/// Transaction count response
#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionCountResponse {
    pub count: u32,
    pub pending_count: u32,
    pub completed_count: u32,
    pub failed_count: u32,
}

/// Consolidation information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConsolidationInfo {
    pub id: String,
    pub wallet_label: String,
    pub currency_id: String,
    pub source_addresses: Vec<String>,
    pub target_address: String,
    pub amount: String,
    pub amount_f: f64,
    pub fee: String,
    pub fee_f: f64,
    pub status: ConsolidationStatus,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// Consolidation status
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ConsolidationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Consolidation request
#[derive(Debug, Serialize, Clone)]
pub struct ConsolidationRequest {
    pub source_addresses: Vec<String>,
    pub target_wallet_label: String,
    pub amount: Option<String>,
    pub note: Option<String>,
}

/// Consolidation preview request
#[derive(Debug, Serialize, Clone)]
pub struct ConsolidationPreviewRequest {
    pub source_wallets: Vec<ConsolidationSourceWallet>,
    pub target_wallet_label: String,
    pub target_currency_id: String,
}

/// Source wallet for consolidation
#[derive(Debug, Serialize, Clone)]
pub struct ConsolidationSourceWallet {
    pub wallet_label: String,
    pub currency_id: String,
    pub addresses: Vec<String>,
}

/// Consolidation preview response
#[derive(Debug, Deserialize, Serialize)]
pub struct ConsolidationPreviewResponse {
    pub total_amount: String,
    pub total_amount_f: f64,
    pub total_fee: String,
    pub total_fee_f: f64,
    pub net_amount: String,
    pub net_amount_f: f64,
    pub address_count: u32,
    pub estimated_time: Option<u32>,
}

impl Default for CreateSpendRequest {
    fn default() -> Self {
        Self {
            amount: String::new(),
            destination_address: None,
            destination_currency_id: None,
            note: None,
            auto_confirm: Some(false),
        }
    }
}

impl CreateSpendRequest {
    /// Create a new spend request
    pub fn new(amount: impl Into<String>) -> Self {
        Self {
            amount: amount.into(),
            ..Default::default()
        }
    }

    /// Set destination address for withdrawal
    pub fn to_address(mut self, address: impl Into<String>) -> Self {
        self.destination_address = Some(address.into());
        self
    }

    /// Set destination currency for conversion
    pub fn to_currency(mut self, currency_id: impl Into<String>) -> Self {
        self.destination_currency_id = Some(currency_id.into());
        self
    }

    /// Add a note to the spend request
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Auto-confirm the spend request
    pub fn auto_confirm(mut self) -> Self {
        self.auto_confirm = Some(true);
        self
    }
}

impl CoinPaymentsClient {
    /// Get transaction count for a wallet
    ///
    /// # Arguments
    /// * `wallet_label` - Wallet label
    /// * `currency_id` - Currency ID
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let count = client.get_transaction_count("my-btc-wallet", "4").await?;
    /// ```
    pub async fn get_transaction_count(
        &self,
        wallet_label: &str,
        currency_id: &str,
    ) -> Result<TransactionCountResponse> {
        let endpoint = format!(
            "v3/merchant/wallets/{}/{}/transactions/count",
            wallet_label, currency_id
        );
        self.get_request(&endpoint, &[]).await
    }

    /// Get transactions for a wallet
    ///
    /// # Arguments
    /// * `wallet_label` - Wallet label
    /// * `currency_id` - Currency ID
    /// * `page` - Page number (optional)
    /// * `per_page` - Results per page (optional)
    /// * `status` - Filter by status (optional)
    /// * `transaction_type` - Filter by type (optional)
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let transactions = client.get_transactions("my-btc-wallet", "4", None, None, None, None).await?;
    /// ```
    pub async fn get_transactions(
        &self,
        wallet_label: &str,
        currency_id: &str,
        page: Option<u32>,
        per_page: Option<u32>,
        status: Option<TransactionStatus>,
        transaction_type: Option<TransactionType>,
    ) -> Result<GetTransactionsResponse> {
        let mut query_params = Vec::new();

        if let Some(page) = page {
            query_params.push(("page", page.to_string()));
        }
        if let Some(per_page) = per_page {
            query_params.push(("per_page", per_page.to_string()));
        }
        if let Some(status) = status {
            query_params.push(("status", format!("{:?}", status).to_lowercase()));
        }
        if let Some(tx_type) = transaction_type {
            query_params.push(("type", format!("{:?}", tx_type)));
        }

        let endpoint = format!(
            "v3/merchant/wallets/{}/{}/transactions",
            wallet_label, currency_id
        );
        self.get_request(&endpoint, &query_params).await
    }

    /// Get a specific transaction
    ///
    /// # Arguments
    /// * `wallet_label` - Wallet label
    /// * `currency_id` - Currency ID
    /// * `transaction_id` - Transaction ID (optional)
    /// * `spend_request_id` - Spend request ID (optional)
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let transaction = client.get_transaction("my-btc-wallet", "4", Some("tx_123"), None).await?;
    /// ```
    pub async fn get_transaction(
        &self,
        wallet_label: &str,
        currency_id: &str,
        transaction_id: Option<&str>,
        spend_request_id: Option<&str>,
    ) -> Result<Transaction> {
        let mut query_params = Vec::new();

        if let Some(tx_id) = transaction_id {
            query_params.push(("transactionId", tx_id.to_string()));
        }
        if let Some(spend_id) = spend_request_id {
            query_params.push(("spendRequestId", spend_id.to_string()));
        }

        let endpoint = format!(
            "v3/merchant/wallets/{}/{}/transaction",
            wallet_label, currency_id
        );
        self.get_request(&endpoint, &query_params).await
    }

    /// Create a spend request
    ///
    /// # Arguments
    /// * `wallet_label` - Wallet label
    /// * `currency_id` - Currency ID
    /// * `request` - Spend request details
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    ///
    /// // Withdrawal
    /// let withdrawal = CreateSpendRequest::new("0.001")
    ///     .to_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa");
    /// let spend = client.create_spend_request("my-btc-wallet", "4", withdrawal).await?;
    ///
    /// // Conversion
    /// let conversion = CreateSpendRequest::new("0.001")
    ///     .to_currency("61"); // Convert BTC to ETH
    /// let spend = client.create_spend_request("my-btc-wallet", "4", conversion).await?;
    /// ```
    pub async fn create_spend_request(
        &self,
        wallet_label: &str,
        currency_id: &str,
        request: CreateSpendRequest,
    ) -> Result<SpendRequestResponse> {
        let endpoint = format!(
            "v3/merchant/wallets/{}/{}/spend/request",
            wallet_label, currency_id
        );
        self.post_request(&endpoint, &request).await
    }

    /// Confirm a spend request
    ///
    /// # Arguments
    /// * `wallet_label` - Wallet label
    /// * `currency_id` - Currency ID
    /// * `spend_request_id` - ID of the spend request to confirm
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let confirmation = client.confirm_spend_request("my-btc-wallet", "4", "spend_123").await?;
    /// ```
    pub async fn confirm_spend_request(
        &self,
        wallet_label: &str,
        currency_id: &str,
        spend_request_id: &str,
    ) -> Result<Transaction> {
        let endpoint = format!(
            "v3/merchant/wallets/{}/{}/spend/confirmation",
            wallet_label, currency_id
        );
        let request = SpendConfirmationRequest {
            spend_request_id: spend_request_id.to_string(),
        };
        self.post_request(&endpoint, &request).await
    }

    /// Get wallet consolidation information
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
    /// let consolidations = client.get_wallet_consolidation("my-btc-wallet", "4", None, None).await?;
    /// ```
    pub async fn get_wallet_consolidation(
        &self,
        wallet_label: &str,
        currency_id: &str,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<Vec<ConsolidationInfo>> {
        let mut query_params = Vec::new();

        if let Some(page) = page {
            query_params.push(("page", page.to_string()));
        }
        if let Some(per_page) = per_page {
            query_params.push(("per_page", per_page.to_string()));
        }

        let endpoint = format!(
            "v3/merchant/wallets/{}/{}/consolidation",
            wallet_label, currency_id
        );
        self.get_request(&endpoint, &query_params).await
    }

    /// Execute wallet consolidation
    ///
    /// # Arguments
    /// * `wallet_label` - Source wallet label
    /// * `currency_id` - Currency ID
    /// * `target_wallet_label` - Target wallet label
    /// * `request` - Consolidation request details
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let request = ConsolidationRequest {
    ///     source_addresses: vec!["addr1".to_string(), "addr2".to_string()],
    ///     target_wallet_label: "main-wallet".to_string(),
    ///     amount: None,
    ///     note: Some("Consolidating funds".to_string()),
    /// };
    /// let consolidation = client.execute_wallet_consolidation("temp-wallet", "4", "main-wallet", request).await?;
    /// ```
    pub async fn execute_wallet_consolidation(
        &self,
        wallet_label: &str,
        currency_id: &str,
        target_wallet_label: &str,
        request: ConsolidationRequest,
    ) -> Result<ConsolidationInfo> {
        let endpoint = format!(
            "v3/merchant/wallets/{}/{}/consolidation/{}",
            wallet_label, currency_id, target_wallet_label
        );
        self.post_request(&endpoint, &request).await
    }

    /// Execute multi-wallet consolidation
    ///
    /// # Arguments
    /// * `target_wallet_label` - Target wallet label
    /// * `request` - Multi-wallet consolidation request
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let request = ConsolidationRequest {
    ///     source_addresses: vec!["addr1".to_string(), "addr2".to_string()],
    ///     target_wallet_label: "main-wallet".to_string(),
    ///     amount: None,
    ///     note: Some("Multi-wallet consolidation".to_string()),
    /// };
    /// let consolidation = client.execute_multi_wallet_consolidation("main-wallet", request).await?;
    /// ```
    pub async fn execute_multi_wallet_consolidation(
        &self,
        target_wallet_label: &str,
        request: ConsolidationRequest,
    ) -> Result<ConsolidationInfo> {
        let endpoint = format!("v3/merchant/wallets/consolidation/{}", target_wallet_label);
        self.post_request(&endpoint, &request).await
    }

    /// Preview consolidation operation
    ///
    /// # Arguments
    /// * `request` - Consolidation preview request
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let request = ConsolidationPreviewRequest {
    ///     source_wallets: vec![ConsolidationSourceWallet {
    ///         wallet_label: "temp-wallet".to_string(),
    ///         currency_id: "4".to_string(),
    ///         addresses: vec!["addr1".to_string(), "addr2".to_string()],
    ///     }],
    ///     target_wallet_label: "main-wallet".to_string(),
    ///     target_currency_id: "4".to_string(),
    /// };
    /// let preview = client.preview_consolidation(request).await?;
    /// ```
    pub async fn preview_consolidation(
        &self,
        request: ConsolidationPreviewRequest,
    ) -> Result<ConsolidationPreviewResponse> {
        self.post_request("v3/merchant/wallets/consolidation-preview", &request)
            .await
    }

    /// Get consolidation transactions
    ///
    /// # Arguments
    /// * `wallet_label` - Wallet label
    /// * `currency_id` - Currency ID
    /// * `consolidation_id` - Consolidation ID
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let transactions = client.get_consolidation_transactions("my-btc-wallet", "4", "cons_123").await?;
    /// ```
    pub async fn get_consolidation_transactions(
        &self,
        wallet_label: &str,
        currency_id: &str,
        consolidation_id: &str,
    ) -> Result<Vec<Transaction>> {
        let endpoint = format!(
            "v3/merchant/wallets/{}/{}/consolidation-transactions/{}",
            wallet_label, currency_id, consolidation_id
        );
        self.get_request(&endpoint, &[]).await
    }
}

// === Helper Functions ===

/// Check if transaction is completed
pub fn is_transaction_completed(transaction: &Transaction) -> bool {
    matches!(
        transaction.status,
        TransactionStatus::Completed | TransactionStatus::ConfirmedOnBlockchain
    )
}

/// Check if transaction is pending
pub fn is_transaction_pending(transaction: &Transaction) -> bool {
    matches!(
        transaction.status,
        TransactionStatus::Pending
            | TransactionStatus::PendingReceive
            | TransactionStatus::Processing
    )
}

/// Check if transaction failed
pub fn is_transaction_failed(transaction: &Transaction) -> bool {
    matches!(
        transaction.status,
        TransactionStatus::Failed
            | TransactionStatus::FailedOnBlockchain
            | TransactionStatus::Expired
    )
}

/// Filter transactions by type
pub fn filter_transactions_by_type(
    transactions: &[Transaction],
    transaction_type: TransactionType,
) -> Vec<&Transaction> {
    transactions
        .iter()
        .filter(|tx| tx.transaction_type == transaction_type)
        .collect()
}

/// Filter transactions by status
pub fn filter_transactions_by_status(
    transactions: &[Transaction],
    status: TransactionStatus,
) -> Vec<&Transaction> {
    transactions
        .iter()
        .filter(|tx| tx.status == status)
        .collect()
}

/// Calculate total transaction amount including fees
pub fn calculate_total_amount(transaction: &Transaction) -> f64 {
    transaction.amount_f + transaction.fee_f.unwrap_or(0.0)
}

/// Get transactions within a date range
pub fn filter_transactions_by_date_range<'a>(
    transactions: &'a [Transaction],
    start_date: &str,
    end_date: &str,
) -> Vec<&'a Transaction> {
    transactions
        .iter()
        .filter(|tx| tx.created_at.as_str() >= start_date && tx.created_at.as_str() <= end_date)
        .collect()
}

/// Group transactions by currency
pub fn group_transactions_by_currency(
    transactions: &[Transaction],
) -> std::collections::HashMap<String, Vec<&Transaction>> {
    let mut grouped = std::collections::HashMap::new();

    for transaction in transactions {
        grouped
            .entry(transaction.currency_id.clone())
            .or_insert_with(Vec::new)
            .push(transaction);
    }

    grouped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_transaction(
        id: &str,
        transaction_type: TransactionType,
        status: TransactionStatus,
        amount: f64,
        fee: Option<f64>,
    ) -> Transaction {
        Transaction {
            id: id.to_string(),
            wallet_id: "wallet_123".to_string(),
            currency_id: "4".to_string(),
            transaction_type,
            amount: amount.to_string(),
            amount_f: amount,
            fee: fee.map(|f| f.to_string()),
            fee_f: fee,
            status,
            address: Some("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string()),
            txid: Some("tx_hash_123".to_string()),
            confirmations: 6,
            required_confirmations: 6,
            network: "mainnet".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
            completed_at: Some("2023-01-01T01:00:00Z".to_string()),
        }
    }

    #[test]
    fn test_is_transaction_completed() {
        let completed = create_test_transaction(
            "tx1",
            TransactionType::ExternalSpend,
            TransactionStatus::Completed,
            0.001,
            Some(0.0001),
        );
        let pending = create_test_transaction(
            "tx2",
            TransactionType::ExternalSpend,
            TransactionStatus::Pending,
            0.001,
            Some(0.0001),
        );

        assert!(is_transaction_completed(&completed));
        assert!(!is_transaction_completed(&pending));
    }

    #[test]
    fn test_is_transaction_pending() {
        let pending = create_test_transaction(
            "tx1",
            TransactionType::ExternalSpend,
            TransactionStatus::Pending,
            0.001,
            Some(0.0001),
        );
        let completed = create_test_transaction(
            "tx2",
            TransactionType::ExternalSpend,
            TransactionStatus::Completed,
            0.001,
            Some(0.0001),
        );

        assert!(is_transaction_pending(&pending));
        assert!(!is_transaction_pending(&completed));
    }

    #[test]
    fn test_calculate_total_amount() {
        let transaction = create_test_transaction(
            "tx1",
            TransactionType::ExternalSpend,
            TransactionStatus::Completed,
            0.001,
            Some(0.0001),
        );

        let total = calculate_total_amount(&transaction);
        assert_eq!(total, 0.0011);
    }

    #[test]
    fn test_filter_transactions_by_type() {
        let transactions = vec![
            create_test_transaction(
                "tx1",
                TransactionType::ExternalSpend,
                TransactionStatus::Completed,
                0.001,
                Some(0.0001),
            ),
            create_test_transaction(
                "tx2",
                TransactionType::InternalReceive,
                TransactionStatus::Completed,
                0.002,
                None,
            ),
            create_test_transaction(
                "tx3",
                TransactionType::ExternalSpend,
                TransactionStatus::Pending,
                0.001,
                Some(0.0001),
            ),
        ];

        let spend_transactions =
            filter_transactions_by_type(&transactions, TransactionType::ExternalSpend);
        assert_eq!(spend_transactions.len(), 2);
    }

    #[test]
    fn test_create_spend_request_builder() {
        let request = CreateSpendRequest::new("0.001")
            .to_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa")
            .with_note("Test withdrawal")
            .auto_confirm();

        assert_eq!(request.amount, "0.001");
        assert_eq!(
            request.destination_address,
            Some("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string())
        );
        assert_eq!(request.note, Some("Test withdrawal".to_string()));
        assert_eq!(request.auto_confirm, Some(true));
    }

    #[test]
    fn test_group_transactions_by_currency() {
        let mut transactions = vec![
            create_test_transaction(
                "tx1",
                TransactionType::ExternalSpend,
                TransactionStatus::Completed,
                0.001,
                Some(0.0001),
            ),
            create_test_transaction(
                "tx2",
                TransactionType::InternalReceive,
                TransactionStatus::Completed,
                0.002,
                None,
            ),
        ];

        // Change one transaction to different currency
        transactions[1].currency_id = "61".to_string(); // ETH

        let grouped = group_transactions_by_currency(&transactions);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get("4").unwrap().len(), 1);
        assert_eq!(grouped.get("61").unwrap().len(), 1);
    }
}
