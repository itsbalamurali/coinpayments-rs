//! Invoice management API endpoints for CoinPayments API
//!
//! This module provides functionality for:
//! - Creating and managing invoices
//! - Getting invoice payment information
//! - Managing invoice status and history
//! - Invoice payouts and completion tracking

use crate::{CoinPaymentsClient, Result};
use serde::{Deserialize, Serialize};

// === Invoice Types ===

/// Invoice information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Invoice {
    pub id: String,
    pub merchant_id: String,
    pub invoice_number: Option<String>,
    pub amount: String,
    pub amount_f: f64,
    pub currency: String,
    pub description: String,
    pub item_name: Option<String>,
    pub item_number: Option<String>,
    pub buyer_email: Option<String>,
    pub buyer_name: Option<String>,
    pub status: InvoiceStatus,
    pub created_at: String,
    pub updated_at: String,
    pub expires_at: String,
    pub paid_at: Option<String>,
    pub completed_at: Option<String>,
    pub invoice_url: String,
    pub payment_urls: Option<Vec<PaymentUrl>>,
}

/// Invoice statuses
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InvoiceStatus {
    /// Invoice saved as draft
    Draft,
    /// Created and waiting to be mailed out on the set date
    Scheduled,
    /// Invoice created, waiting for payment
    Unpaid,
    /// Payment detected on chain, waiting to be received by CPS
    Pending,
    /// All confirmation received, payment received by CPS and scheduled for payout
    Paid,
    /// Paid out to merchant
    Completed,
    /// Invoice cancelled by merchant
    Cancelled,
    /// Invoice expired
    TimedOut,
    /// Invoice deleted
    Deleted,
}

/// Payment URL for different currencies
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaymentUrl {
    pub currency_id: String,
    pub currency_symbol: String,
    pub url: String,
}

/// Request to create an invoice
#[derive(Debug, Serialize, Clone)]
pub struct CreateInvoiceRequest {
    pub amount: String,
    pub currency: String,
    pub description: String,
    pub invoice_number: Option<String>,
    pub item_name: Option<String>,
    pub item_number: Option<String>,
    pub buyer_email: Option<String>,
    pub buyer_name: Option<String>,
    pub success_url: Option<String>,
    pub cancel_url: Option<String>,
    pub ipn_url: Option<String>,
    pub expires_in: Option<u32>, // seconds
    pub payment_currencies: Option<Vec<String>>,
    pub auto_accept_payments: Option<bool>,
}

/// Response for creating an invoice
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateInvoiceResponse {
    pub invoice: Invoice,
    pub payment_info: Option<Vec<PaymentInfo>>,
}

/// Payment information for an invoice
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaymentInfo {
    pub currency_id: String,
    pub currency_symbol: String,
    pub address: String,
    pub amount: String,
    pub amount_f: f64,
    pub qr_code_url: String,
    pub payment_url: String,
    pub timeout: u32,
    pub required_confirmations: u32,
}

/// Payment status information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaymentStatus {
    pub currency_id: String,
    pub amount_paid: String,
    pub amount_paid_f: f64,
    pub amount_received: String,
    pub amount_received_f: f64,
    pub confirmations: u32,
    pub required_confirmations: u32,
    pub status: PaymentStatusType,
    pub txid: Option<String>,
    pub first_seen: Option<String>,
    pub last_updated: String,
}

/// Payment status types
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatusType {
    Waiting,
    Pending,
    Confirmed,
    Completed,
    Failed,
    Expired,
}

/// Response for getting invoices
#[derive(Debug, Deserialize, Serialize)]
pub struct GetInvoicesResponse {
    pub invoices: Vec<Invoice>,
    pub pagination: Option<InvoicePaginationInfo>,
}

/// Pagination information for invoices
#[derive(Debug, Deserialize, Serialize)]
pub struct InvoicePaginationInfo {
    pub page: u32,
    pub per_page: u32,
    pub total: u32,
    pub total_pages: u32,
}

/// Invoice payout information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InvoicePayout {
    pub id: String,
    pub invoice_id: String,
    pub amount: String,
    pub amount_f: f64,
    pub currency: String,
    pub destination_address: String,
    pub txid: Option<String>,
    pub status: PayoutStatus,
    pub fee: Option<String>,
    pub fee_f: Option<f64>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// Payout status
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PayoutStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Response for getting invoice payouts
#[derive(Debug, Deserialize, Serialize)]
pub struct GetInvoicePayoutsResponse {
    pub payouts: Vec<InvoicePayout>,
}

/// Invoice history entry
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InvoiceHistoryEntry {
    pub id: String,
    pub invoice_id: String,
    pub event_type: InvoiceEventType,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
}

/// Invoice event types
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum InvoiceEventType {
    Created,
    Updated,
    PaymentCreated,
    PaymentReceived,
    PaymentConfirmed,
    Paid,
    Completed,
    Cancelled,
    Expired,
    PayoutCreated,
    PayoutCompleted,
}

/// Response for getting invoice history
#[derive(Debug, Deserialize, Serialize)]
pub struct GetInvoiceHistoryResponse {
    pub history: Vec<InvoiceHistoryEntry>,
}

impl Default for CreateInvoiceRequest {
    fn default() -> Self {
        Self {
            amount: String::new(),
            currency: String::new(),
            description: String::new(),
            invoice_number: None,
            item_name: None,
            item_number: None,
            buyer_email: None,
            buyer_name: None,
            success_url: None,
            cancel_url: None,
            ipn_url: None,
            expires_in: Some(3600), // 1 hour default
            payment_currencies: None,
            auto_accept_payments: Some(true),
        }
    }
}

impl CreateInvoiceRequest {
    /// Create a new invoice request
    pub fn new(
        amount: impl Into<String>,
        currency: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            amount: amount.into(),
            currency: currency.into(),
            description: description.into(),
            ..Default::default()
        }
    }

    /// Set invoice number
    pub fn with_invoice_number(mut self, number: impl Into<String>) -> Self {
        self.invoice_number = Some(number.into());
        self
    }

    /// Set item details
    pub fn with_item(mut self, name: impl Into<String>, number: Option<String>) -> Self {
        self.item_name = Some(name.into());
        self.item_number = number;
        self
    }

    /// Set buyer information
    pub fn with_buyer(mut self, email: impl Into<String>, name: Option<String>) -> Self {
        self.buyer_email = Some(email.into());
        self.buyer_name = name;
        self
    }

    /// Set success URL for completed payments
    pub fn with_success_url(mut self, url: impl Into<String>) -> Self {
        self.success_url = Some(url.into());
        self
    }

    /// Set cancel URL for cancelled payments
    pub fn with_cancel_url(mut self, url: impl Into<String>) -> Self {
        self.cancel_url = Some(url.into());
        self
    }

    /// Set IPN URL for payment notifications
    pub fn with_ipn_url(mut self, url: impl Into<String>) -> Self {
        self.ipn_url = Some(url.into());
        self
    }

    /// Set expiration time in seconds
    pub fn expires_in_seconds(mut self, seconds: u32) -> Self {
        self.expires_in = Some(seconds);
        self
    }

    /// Set expiration time in minutes
    pub fn expires_in_minutes(mut self, minutes: u32) -> Self {
        self.expires_in = Some(minutes * 60);
        self
    }

    /// Set accepted payment currencies
    pub fn with_payment_currencies(mut self, currencies: Vec<String>) -> Self {
        self.payment_currencies = Some(currencies);
        self
    }

    /// Set auto-accept payments
    pub fn auto_accept_payments(mut self, auto_accept: bool) -> Self {
        self.auto_accept_payments = Some(auto_accept);
        self
    }
}

impl CoinPaymentsClient {
    /// Create a new invoice
    ///
    /// # Arguments
    /// * `request` - Invoice creation request
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let request = CreateInvoiceRequest::new("10.00", "USD", "Payment for services")
    ///     .with_buyer("customer@example.com", Some("John Doe".to_string()))
    ///     .with_payment_currencies(vec!["4".to_string(), "61".to_string()]) // BTC, ETH
    ///     .expires_in_minutes(60);
    /// let invoice = client.create_invoice(request).await?;
    /// ```
    pub async fn create_invoice(
        &self,
        request: CreateInvoiceRequest,
    ) -> Result<CreateInvoiceResponse> {
        self.post_request("v2/merchant/invoices", &request).await
    }

    /// Cancel an invoice
    ///
    /// # Arguments
    /// * `invoice_id` - ID of the invoice to cancel
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// client.cancel_invoice("inv_123").await?;
    /// ```
    pub async fn cancel_invoice(&self, invoice_id: &str) -> Result<()> {
        let endpoint = format!("v1/merchant/invoices/{}/cancel", invoice_id);
        self.post_request(&endpoint, &serde_json::Value::Null).await
    }

    /// Get list of invoices
    ///
    /// # Arguments
    /// * `page` - Page number (optional)
    /// * `per_page` - Results per page (optional)
    /// * `status` - Filter by status (optional)
    /// * `currency` - Filter by currency (optional)
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let invoices = client.get_invoices(None, None, Some(InvoiceStatus::Unpaid), None).await?;
    /// ```
    pub async fn get_invoices(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
        status: Option<InvoiceStatus>,
        currency: Option<&str>,
    ) -> Result<GetInvoicesResponse> {
        let mut query_params = Vec::new();

        if let Some(page) = page {
            query_params.push(("page", page.to_string()));
        }
        if let Some(per_page) = per_page {
            query_params.push(("per_page", per_page.to_string()));
        }
        if let Some(status) = status {
            let status_str = match status {
                InvoiceStatus::Draft => "draft",
                InvoiceStatus::Scheduled => "scheduled",
                InvoiceStatus::Unpaid => "unpaid",
                InvoiceStatus::Pending => "pending",
                InvoiceStatus::Paid => "paid",
                InvoiceStatus::Completed => "completed",
                InvoiceStatus::Cancelled => "cancelled",
                InvoiceStatus::TimedOut => "timedOut",
                InvoiceStatus::Deleted => "deleted",
            };
            query_params.push(("status", status_str.to_string()));
        }
        if let Some(currency) = currency {
            query_params.push(("currency", currency.to_string()));
        }

        self.get_request("v2/merchant/invoices", &query_params)
            .await
    }

    /// Get invoice payment information for a specific currency
    ///
    /// # Arguments
    /// * `invoice_id` - Invoice ID
    /// * `currency_id` - Currency ID
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let payment_info = client.get_invoice_payment_info("inv_123", "4").await?; // Bitcoin
    /// ```
    pub async fn get_invoice_payment_info(
        &self,
        invoice_id: &str,
        currency_id: &str,
    ) -> Result<PaymentInfo> {
        let endpoint = format!(
            "v1/invoices/{}/payment-currencies/{}",
            invoice_id, currency_id
        );
        self.get_request(&endpoint, &[]).await
    }

    /// Get invoice payment status for a specific currency
    ///
    /// # Arguments
    /// * `invoice_id` - Invoice ID
    /// * `currency_id` - Currency ID
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let status = client.get_invoice_payment_status("inv_123", "4").await?; // Bitcoin
    /// ```
    pub async fn get_invoice_payment_status(
        &self,
        invoice_id: &str,
        currency_id: &str,
    ) -> Result<PaymentStatus> {
        let endpoint = format!(
            "v1/invoices/{}/payment-currencies/{}/status",
            invoice_id, currency_id
        );
        self.get_request(&endpoint, &[]).await
    }

    /// Get invoice by ID
    ///
    /// # Arguments
    /// * `invoice_id` - Invoice ID
    /// * `include_payments` - Include payment information (optional)
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let invoice = client.get_invoice("inv_123", Some(true)).await?;
    /// ```
    pub async fn get_invoice(
        &self,
        invoice_id: &str,
        include_payments: Option<bool>,
    ) -> Result<Invoice> {
        let mut query_params = Vec::new();

        if let Some(include_payments) = include_payments {
            query_params.push(("include_payments", include_payments.to_string()));
        }

        let endpoint = format!("v2/merchant/invoices/{}", invoice_id);
        self.get_request(&endpoint, &query_params).await
    }

    /// Get invoice payouts
    ///
    /// # Arguments
    /// * `invoice_id` - Invoice ID
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let payouts = client.get_invoice_payouts("inv_123").await?;
    /// ```
    pub async fn get_invoice_payouts(&self, invoice_id: &str) -> Result<GetInvoicePayoutsResponse> {
        let endpoint = format!("v2/merchant/invoices/{}/payouts", invoice_id);
        self.get_request(&endpoint, &[]).await
    }

    /// Get invoice history
    ///
    /// # Arguments
    /// * `invoice_id` - Invoice ID
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let history = client.get_invoice_history("inv_123").await?;
    /// ```
    pub async fn get_invoice_history(&self, invoice_id: &str) -> Result<GetInvoiceHistoryResponse> {
        let endpoint = format!("v2/merchant/invoices/{}/history", invoice_id);
        self.get_request(&endpoint, &[]).await
    }
}

// === Helper Functions ===

/// Check if invoice is paid
pub fn is_invoice_paid(invoice: &Invoice) -> bool {
    matches!(
        invoice.status,
        InvoiceStatus::Paid | InvoiceStatus::Completed
    )
}

/// Check if invoice is still active (can receive payments)
pub fn is_invoice_active(invoice: &Invoice) -> bool {
    matches!(
        invoice.status,
        InvoiceStatus::Unpaid | InvoiceStatus::Pending
    )
}

/// Check if invoice has expired
pub fn is_invoice_expired(invoice: &Invoice) -> bool {
    matches!(invoice.status, InvoiceStatus::TimedOut)
}

/// Check if invoice is cancelled
pub fn is_invoice_cancelled(invoice: &Invoice) -> bool {
    matches!(invoice.status, InvoiceStatus::Cancelled)
}

/// Filter invoices by status
pub fn filter_invoices_by_status(invoices: &[Invoice], status: InvoiceStatus) -> Vec<&Invoice> {
    invoices
        .iter()
        .filter(|invoice| invoice.status == status)
        .collect()
}

/// Get total amount of invoices
pub fn calculate_total_invoice_amount(invoices: &[Invoice]) -> f64 {
    invoices.iter().map(|invoice| invoice.amount_f).sum()
}

/// Get invoices within a date range
pub fn filter_invoices_by_date_range<'a>(
    invoices: &'a [Invoice],
    start_date: &str,
    end_date: &str,
) -> Vec<&'a Invoice> {
    invoices
        .iter()
        .filter(|invoice| {
            invoice.created_at.as_str() >= start_date && invoice.created_at.as_str() <= end_date
        })
        .collect()
}

/// Find invoice by invoice number
pub fn find_invoice_by_number<'a>(
    invoices: &'a [Invoice],
    invoice_number: &str,
) -> Option<&'a Invoice> {
    invoices.iter().find(|invoice| {
        invoice
            .invoice_number
            .as_ref()
            .map_or(false, |num| num == invoice_number)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_invoice(id: &str, status: InvoiceStatus, amount: f64) -> Invoice {
        Invoice {
            id: id.to_string(),
            merchant_id: "merchant_123".to_string(),
            invoice_number: Some("INV-001".to_string()),
            amount: amount.to_string(),
            amount_f: amount,
            currency: "USD".to_string(),
            description: "Test invoice".to_string(),
            item_name: Some("Test item".to_string()),
            item_number: Some("ITEM-001".to_string()),
            buyer_email: Some("buyer@example.com".to_string()),
            buyer_name: Some("John Doe".to_string()),
            status,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
            expires_at: "2023-01-01T01:00:00Z".to_string(),
            paid_at: None,
            completed_at: None,
            invoice_url: "https://checkout.coinpayments.net/inv_123".to_string(),
            payment_urls: None,
        }
    }

    #[test]
    fn test_is_invoice_paid() {
        let paid_invoice = create_test_invoice("inv1", InvoiceStatus::Paid, 10.0);
        let unpaid_invoice = create_test_invoice("inv2", InvoiceStatus::Unpaid, 10.0);

        assert!(is_invoice_paid(&paid_invoice));
        assert!(!is_invoice_paid(&unpaid_invoice));
    }

    #[test]
    fn test_is_invoice_active() {
        let active_invoice = create_test_invoice("inv1", InvoiceStatus::Unpaid, 10.0);
        let expired_invoice = create_test_invoice("inv2", InvoiceStatus::TimedOut, 10.0);

        assert!(is_invoice_active(&active_invoice));
        assert!(!is_invoice_active(&expired_invoice));
    }

    #[test]
    fn test_filter_invoices_by_status() {
        let invoices = vec![
            create_test_invoice("inv1", InvoiceStatus::Paid, 10.0),
            create_test_invoice("inv2", InvoiceStatus::Unpaid, 20.0),
            create_test_invoice("inv3", InvoiceStatus::Paid, 15.0),
        ];

        let paid_invoices = filter_invoices_by_status(&invoices, InvoiceStatus::Paid);
        assert_eq!(paid_invoices.len(), 2);
    }

    #[test]
    fn test_calculate_total_invoice_amount() {
        let invoices = vec![
            create_test_invoice("inv1", InvoiceStatus::Paid, 10.0),
            create_test_invoice("inv2", InvoiceStatus::Unpaid, 20.0),
            create_test_invoice("inv3", InvoiceStatus::Paid, 15.0),
        ];

        let total = calculate_total_invoice_amount(&invoices);
        assert_eq!(total, 45.0);
    }

    #[test]
    fn test_find_invoice_by_number() {
        let invoices = vec![
            create_test_invoice("inv1", InvoiceStatus::Paid, 10.0),
            create_test_invoice("inv2", InvoiceStatus::Unpaid, 20.0),
        ];

        let found = find_invoice_by_number(&invoices, "INV-001");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "inv1");

        let not_found = find_invoice_by_number(&invoices, "INV-999");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_create_invoice_request_builder() {
        let request = CreateInvoiceRequest::new("100.00", "USD", "Payment for services")
            .with_invoice_number("INV-123")
            .with_buyer("customer@example.com", Some("Jane Doe".to_string()))
            .with_item("Product A", Some("SKU-001".to_string()))
            .expires_in_minutes(30)
            .auto_accept_payments(false);

        assert_eq!(request.amount, "100.00");
        assert_eq!(request.currency, "USD");
        assert_eq!(request.description, "Payment for services");
        assert_eq!(request.invoice_number, Some("INV-123".to_string()));
        assert_eq!(
            request.buyer_email,
            Some("customer@example.com".to_string())
        );
        assert_eq!(request.buyer_name, Some("Jane Doe".to_string()));
        assert_eq!(request.item_name, Some("Product A".to_string()));
        assert_eq!(request.item_number, Some("SKU-001".to_string()));
        assert_eq!(request.expires_in, Some(1800)); // 30 minutes
        assert_eq!(request.auto_accept_payments, Some(false));
    }
}
