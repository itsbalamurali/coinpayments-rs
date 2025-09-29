# CoinPayments Rust SDK

A comprehensive Rust client library for the CoinPayments API that provides easy access to cryptocurrency payment processing, wallet management, transaction handling, and more.

[![Crates.io](https://img.shields.io/crates/v/coinpayments.svg)](https://crates.io/crates/coinpayments)
[![Documentation](https://docs.rs/coinpayments/badge.svg)](https://docs.rs/coinpayments)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **🪙 Currencies**: Get supported currencies, rates, and conversion information
- **💱 Rates**: Real-time exchange rates and market data
- **💸 Fees**: Blockchain fee calculations and estimates
- **👛 Wallets**: Create and manage wallets and addresses
- **🔄 Transactions**: Handle payments, withdrawals, and consolidations
- **🧾 Invoices**: Create and manage payment invoices
- **🔔 Webhooks**: Set up and manage webhook notifications
- **🛠️ Utils**: Comprehensive utility functions for validation and formatting

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
coinpayments = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

### Basic Usage

```rust
use coinpayments::{CoinPaymentsClient, CreateInvoiceRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the client with your API credentials
    let client = CoinPaymentsClient::new("your_client_id", "your_client_secret");

    // Test API connectivity
    let ping = client.ping().await?;
    println!("API Status: {}", ping.message);

    // Create an invoice
    let invoice_request = CreateInvoiceRequest::new("25.00", "USD", "Payment for premium service")
        .with_buyer("customer@example.com", Some("John Doe".to_string()))
        .with_payment_currencies(vec!["4".to_string(), "61".to_string()]) // BTC, ETH
        .expires_in_minutes(60);

    let invoice = client.create_invoice(invoice_request).await?;
    println!("Invoice created: {}", invoice.invoice.invoice_url);

    Ok(())
}
```

## API Structure

The SDK is organized into logical modules:

### 🪙 Currencies API

```rust
// Get all supported currencies
let currencies = client.get_currencies(None, None).await?;

// Get specific currency by ID
let bitcoin = client.get_currency_by_id("4").await?; // Bitcoin

// Get merchant's accepted currencies
let merchant_currencies = client.get_merchant_currencies().await?;

// Get currency limits for conversions
let limits = client.get_currency_limits("4", "61").await?; // BTC to ETH
```

### 💱 Exchange Rates API

```rust
use coinpayments::RateQuery;

// Get all exchange rates
let rates = client.get_rates(None).await?;

// Get specific rate
let btc_eth_rate = client.get_rate("4", "61").await?; // BTC to ETH

// Get rates with filters
let btc_rates = client.get_rates(Some(
    RateQuery::new().from_currency("4").page(1, Some(10))
)).await?;
```

### 💸 Blockchain Fees API

```rust
use coinpayments::{FeeCalculationRequest, TransactionType, FeePriority};

// Calculate basic fees
let fees = client.calculate_blockchain_fee("4", None).await?; // Bitcoin

// Calculate fees with specific parameters
let fee_request = FeeCalculationRequest::new("4", TransactionType::Send)
    .with_amount("0.001")
    .with_priority(FeePriority::Fast);
let fees = client.calculate_blockchain_fee("4", Some(fee_request)).await?;

// Get recommended fee for target confirmation time
let fee = client.get_recommended_fee("4", 30).await?; // 30 minutes
```

### 👛 Wallets API

```rust
use coinpayments::{CreateWalletRequest, AddressType, WalletStatus};

// Create a new wallet
let wallet_request = CreateWalletRequest::new("my-btc-wallet", "4")
    .with_permanent_addresses(true)
    .with_webhook("https://your-server.com/webhook");
let wallet = client.create_wallet(wallet_request).await?;

// List wallets
let wallets = client.get_wallets(None, None, None, Some(WalletStatus::Active)).await?;

// Get wallet addresses
let addresses = client.get_wallet_addresses("my-btc-wallet", "4", None, None).await?;
```

### 🔄 Transactions API

```rust
use coinpayments::CreateSpendRequest;

// Create spend request (withdrawal)
let spend_request = CreateSpendRequest::new("0.001")
    .to_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa")
    .with_note("Test withdrawal");
let spend = client.create_spend_request("my-btc-wallet", "4", spend_request).await?;

// Confirm spend request
let transaction = client.confirm_spend_request("my-btc-wallet", "4", &spend.request.id).await?;

// Get transaction history
let transactions = client.get_transactions("my-btc-wallet", "4", None, None, None, None).await?;
```

### 🧾 Invoices API

```rust
use coinpayments::{CreateInvoiceRequest, InvoiceStatus};

// Create an invoice
let invoice_request = CreateInvoiceRequest::new("100.00", "USD", "Premium subscription")
    .with_invoice_number("INV-2024-001")
    .with_buyer("customer@example.com", Some("John Doe".to_string()))
    .with_payment_currencies(vec!["4".to_string(), "61".to_string()])
    .expires_in_minutes(60);
let invoice = client.create_invoice(invoice_request).await?;

// Get invoices
let invoices = client.get_invoices(Some(1), Some(10), Some(InvoiceStatus::Unpaid), None).await?;

// Get invoice payment status
let payment_status = client.get_invoice_payment_status(&invoice.invoice.id, "4").await?;
```

### 🔔 Webhooks API

```rust
use coinpayments::{CreateClientWebhookRequest, ClientWebhookEvent, WebhookConfig, WalletWebhookEvent};

// Create client webhook for invoices
let webhook_request = CreateClientWebhookRequest::new("https://your-server.com/webhook")
    .with_events(vec![
        ClientWebhookEvent::InvoiceCreated,
        ClientWebhookEvent::InvoiceCompleted
    ])
    .with_secret("webhook_secret");
let webhook = client.create_client_webhook("client_id", webhook_request).await?;

// Update wallet webhook
let wallet_webhook = WebhookConfig {
    url: "https://your-server.com/wallet-webhook".to_string(),
    events: vec![WalletWebhookEvent::UtxoExternalReceive, WalletWebhookEvent::ExternalSpend],
    secret: Some("wallet_webhook_secret".to_string()),
};
client.update_wallet_webhook_v3("my-btc-wallet", "4", wallet_webhook).await?;
```

## Webhook Verification

The SDK provides utilities for webhook verification:

```rust
use coinpayments::{parse_webhook_headers, verify_webhook_signature, is_webhook_timestamp_valid};

async fn handle_webhook(
    payload: &[u8],
    headers: std::collections::HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = "your_private_key";

    // Parse headers
    let webhook_headers = parse_webhook_headers(&headers)?;

    // Verify signature
    if !verify_webhook_signature(private_key, &webhook_headers, payload) {
        return Err("Invalid signature".into());
    }

    // Check timestamp (5 minutes tolerance)
    if !is_webhook_timestamp_valid(&webhook_headers.timestamp, 300) {
        return Err("Timestamp too old".into());
    }

    // Process webhook...
    println!("Valid webhook received!");
    Ok(())
}
```

## Utility Functions

The SDK includes comprehensive utility functions:

```rust
use coinpayments::{
    format_amount, to_smallest_unit, from_smallest_unit,
    is_valid_bitcoin_address, is_valid_ethereum_address, is_valid_email
};

// Format amounts
let formatted = format_amount(1.23456789, 2); // "1.23"

// Convert to/from smallest units (satoshis, wei, etc.)
let satoshis = to_smallest_unit(1.0, 8); // 100_000_000
let btc = from_smallest_unit(100_000_000, 8); // 1.0

// Validate addresses and emails
let is_valid_btc = is_valid_bitcoin_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa");
let is_valid_eth = is_valid_ethereum_address("0x742d35Cc6635C0532925a3b8D6ac492395a3d728");
let is_valid_mail = is_valid_email("user@example.com");
```

## Legacy API Support

For backward compatibility with the v1 API:

```rust
use coinpayments::endpoints::CoinPaymentsLegacyClient;
use coinpayments::types::{CreateTransactionRequest, CreateWithdrawalRequest};

// Create legacy client
let legacy_client = CoinPaymentsLegacyClient::new("public_key", "private_key");

// Use legacy methods
let rates = legacy_client.get_rates(Some(true)).await?;
let balances = legacy_client.get_balances(None).await?;

// Create transaction (legacy style)
let transaction_request = CreateTransactionRequest::new(10.0, "USD", "BTC")
    .with_buyer_email("customer@example.com");
let transaction = legacy_client.create_transaction(&transaction_request).await?;
```

## Error Handling

The SDK provides comprehensive error handling:

```rust
use coinpayments::{CoinPaymentsError, Result};

match client.get_currencies(None, None).await {
    Ok(currencies) => {
        println!("Success: {} currencies", currencies.currencies.len());
    }
    Err(CoinPaymentsError::Authentication) => {
        println!("Authentication failed - check your credentials");
    }
    Err(CoinPaymentsError::RateLimit) => {
        println!("Rate limit exceeded - wait before retrying");
    }
    Err(CoinPaymentsError::NotFound) => {
        println!("Resource not found");
    }
    Err(CoinPaymentsError::Api { message }) => {
        println!("API error: {}", message);
    }
    Err(e) => {
        println!("Other error: {}", e);
    }
}
```

## Testing

Run the tests:

```bash
cargo test
```

Run the comprehensive example:

```bash
# Set your credentials as environment variables
export COINPAYMENTS_CLIENT_ID="your_client_id"
export COINPAYMENTS_CLIENT_SECRET="your_client_secret"

cargo run --example comprehensive_example
```

## API Reference

For detailed API documentation, visit:
- [CoinPayments API Documentation](https://a-docs.coinpayments.net/)
- [Rust SDK Documentation](https://docs.rs/coinpayments)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- 📚 [Documentation](https://docs.rs/coinpayments)
- 🐛 [Issues](https://github.com/itsbalamurali/coinpayments-rs/issues)
- 💬 [Discussions](https://github.com/itsbalamurali/coinpayments-rs/discussions)

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a list of changes and version history.