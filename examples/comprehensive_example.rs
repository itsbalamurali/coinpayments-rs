//! Comprehensive example demonstrating the CoinPayments API v2/v3 features
//!
//! This example showcases the main functionality of the restructured CoinPayments API client:
//! - Currency management
//! - Exchange rates
//! - Blockchain fees
//! - Wallet operations
//! - Transaction handling
//! - Invoice creation
//! - Webhook management

use coinpayments::{
    AddressType, CoinPaymentsClient, CreateInvoiceRequest, CreateSpendRequest, CreateWalletRequest,
    CurrencyCapability, CurrencyStatus, InvoiceStatus, RateQuery, Result, WalletStatus,
    WalletWebhookEvent, WebhookConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the client with your credentials
    let client = CoinPaymentsClient::new("your_client_id", "your_client_secret");

    println!("🚀 CoinPayments API Demo Starting...\n");

    // Test API connectivity
    match client.ping().await {
        Ok(ping) => println!("✅ API Connection: {}", ping.message),
        Err(e) => {
            println!("❌ API Connection failed: {}", e);
            return Err(e);
        }
    }

    // === CURRENCIES API DEMO ===
    println!("\n📊 === CURRENCIES API ===");

    // Get supported currencies
    match client.get_currencies(Some(1), Some(10)).await {
        Ok(currencies_response) => {
            println!(
                "✅ Found {} currencies",
                currencies_response.currencies.len()
            );

            // Filter active currencies that support deposits
            let active_deposit_currencies: Vec<_> = currencies_response
                .currencies
                .iter()
                .filter(|c| {
                    c.status == CurrencyStatus::Active
                        && c.capabilities.contains(&CurrencyCapability::Deposit)
                })
                .collect();

            println!(
                "💰 Active deposit currencies: {}",
                active_deposit_currencies.len()
            );

            if let Some(first_currency) = currencies_response.currencies.first() {
                println!(
                    "📈 Example currency: {} ({}) - {} decimals",
                    first_currency.name, first_currency.symbol, first_currency.decimals
                );
            }
        }
        Err(e) => println!("❌ Failed to get currencies: {}", e),
    }

    // Get specific currency (Bitcoin)
    match client.get_currency_by_id("4").await {
        Ok(bitcoin) => {
            println!("₿ Bitcoin: {} ({})", bitcoin.name, bitcoin.symbol);
        }
        Err(e) => println!("❌ Failed to get Bitcoin info: {}", e),
    }

    // === RATES API DEMO ===
    println!("\n💱 === EXCHANGE RATES API ===");

    // Get all rates
    match client.get_rates(None).await {
        Ok(rates_response) => {
            println!("✅ Retrieved {} exchange rates", rates_response.rates.len());

            if let Some(first_rate) = rates_response.rates.first() {
                println!(
                    "📊 Example rate: {} -> {} = {}",
                    first_rate.from_currency_id, first_rate.to_currency_id, first_rate.rate_f
                );
            }
        }
        Err(e) => println!("❌ Failed to get rates: {}", e),
    }

    // Get specific rate (BTC to ETH)
    match client.get_rate("4", "61").await {
        Ok(rate) => {
            println!("₿➡️📈 BTC to ETH rate: {}", rate.rate_f);
        }
        Err(e) => println!("❌ Failed to get BTC/ETH rate: {}", e),
    }

    // Get rates with query filters
    let rate_query = RateQuery::new()
        .from_currency("4") // Bitcoin
        .page(1, Some(5));

    match client.get_rates(Some(rate_query)).await {
        Ok(btc_rates) => {
            println!("₿ Bitcoin rates to {} currencies", btc_rates.rates.len());
        }
        Err(e) => println!("❌ Failed to get Bitcoin rates: {}", e),
    }

    // === FEES API DEMO ===
    println!("\n💸 === BLOCKCHAIN FEES API ===");

    // Calculate Bitcoin transaction fee
    match client.calculate_blockchain_fee("4", None).await {
        Ok(fee_response) => {
            println!("✅ Bitcoin fees calculated");
            println!(
                "🏃 Recommended fee: {} (priority: {:?})",
                fee_response.recommended_fee.amount_f, fee_response.recommended_fee.priority_level
            );

            // Show all fee options
            for fee in &fee_response.fees {
                println!(
                    "  💰 {:?}: {} {} (est. {} min)",
                    fee.priority_level,
                    fee.amount_f,
                    fee.currency_symbol,
                    fee.estimated_confirmation_time.unwrap_or(0)
                );
            }
        }
        Err(e) => println!("❌ Failed to calculate fees: {}", e),
    }

    // Get recommended fee for 30-minute confirmation
    match client.get_recommended_fee("4", 30).await {
        Ok(fee) => {
            println!(
                "⏰ 30-min confirmation fee: {} {} ({:?})",
                fee.amount_f, fee.currency_symbol, fee.priority_level
            );
        }
        Err(e) => println!("❌ Failed to get recommended fee: {}", e),
    }

    // === WALLETS API DEMO ===
    println!("\n👛 === WALLETS API ===");

    // Create a new wallet
    let wallet_request = CreateWalletRequest::new("demo-btc-wallet", "4")
        .with_permanent_addresses(true)
        .with_webhook("https://your-server.com/webhook");

    match client.create_wallet(wallet_request).await {
        Ok(wallet_response) => {
            println!("✅ Created wallet: {}", wallet_response.wallet.label);
            println!(
                "💰 Balance: {} {}",
                wallet_response.wallet.balance_f, wallet_response.wallet.currency_symbol
            );
        }
        Err(e) => println!("❌ Failed to create wallet: {}", e),
    }

    // Get wallet count
    match client.get_wallet_count().await {
        Ok(count) => {
            println!(
                "📊 Total wallets: {} (active: {})",
                count.count, count.active_count
            );
        }
        Err(e) => println!("❌ Failed to get wallet count: {}", e),
    }

    // List all wallets
    match client
        .get_wallets(None, None, None, Some(WalletStatus::Active))
        .await
    {
        Ok(wallets_response) => {
            println!("📝 Active wallets: {}", wallets_response.wallets.len());

            for wallet in &wallets_response.wallets {
                println!(
                    "  👛 {}: {} {} ({})",
                    wallet.label,
                    wallet.balance_f,
                    wallet.currency_symbol,
                    match wallet.address_type {
                        AddressType::Temporary => "Temp",
                        AddressType::Permanent => "Perm",
                    }
                );
            }
        }
        Err(e) => println!("❌ Failed to get wallets: {}", e),
    }

    // === TRANSACTIONS API DEMO ===
    println!("\n🔄 === TRANSACTIONS API ===");

    // Get transaction count for a wallet
    match client.get_transaction_count("demo-btc-wallet", "4").await {
        Ok(count) => {
            println!(
                "📊 Transaction count: {} (pending: {}, completed: {})",
                count.count, count.pending_count, count.completed_count
            );
        }
        Err(e) => println!("❌ Failed to get transaction count: {}", e),
    }

    // Create a spend request (withdrawal)
    let spend_request = CreateSpendRequest::new("0.001")
        .to_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa")
        .with_note("Demo withdrawal");

    match client
        .create_spend_request("demo-btc-wallet", "4", spend_request)
        .await
    {
        Ok(spend_response) => {
            println!("✅ Created spend request: {}", spend_response.request.id);
            println!(
                "💰 Amount: {} + fee: {} = total: {}",
                spend_response.preview.amount_f,
                spend_response.preview.fee_f,
                spend_response.preview.total_f
            );
        }
        Err(e) => println!("❌ Failed to create spend request: {}", e),
    }

    // === INVOICES API DEMO ===
    println!("\n🧾 === INVOICES API ===");

    // Create an invoice
    let invoice_request = CreateInvoiceRequest::new("25.00", "USD", "Payment for premium service")
        .with_invoice_number("INV-2024-001")
        .with_buyer("customer@example.com", Some("John Doe".to_string()))
        .with_item("Premium Plan", Some("PLAN-PREMIUM".to_string()))
        .with_payment_currencies(vec!["4".to_string(), "61".to_string()]) // BTC, ETH
        .expires_in_minutes(60)
        .with_success_url("https://your-site.com/success")
        .with_cancel_url("https://your-site.com/cancel");

    match client.create_invoice(invoice_request).await {
        Ok(invoice_response) => {
            println!("✅ Created invoice: {}", invoice_response.invoice.id);
            println!("🔗 Payment URL: {}", invoice_response.invoice.invoice_url);
            println!(
                "💰 Amount: {} {}",
                invoice_response.invoice.amount_f, invoice_response.invoice.currency
            );

            if let Some(payment_info) = &invoice_response.payment_info {
                println!("💳 Payment options: {}", payment_info.len());
                for info in payment_info {
                    println!(
                        "  {} {}: {} ({})",
                        info.currency_symbol, info.amount_f, info.address, info.payment_url
                    );
                }
            }
        }
        Err(e) => println!("❌ Failed to create invoice: {}", e),
    }

    // Get invoices
    match client
        .get_invoices(Some(1), Some(5), Some(InvoiceStatus::Unpaid), None)
        .await
    {
        Ok(invoices_response) => {
            println!("📋 Unpaid invoices: {}", invoices_response.invoices.len());

            for invoice in &invoices_response.invoices {
                println!(
                    "  🧾 {}: {} {} ({:?})",
                    invoice.invoice_number.as_deref().unwrap_or("N/A"),
                    invoice.amount_f,
                    invoice.currency,
                    invoice.status
                );
            }
        }
        Err(e) => println!("❌ Failed to get invoices: {}", e),
    }

    // === WEBHOOKS API DEMO ===
    println!("\n🔔 === WEBHOOKS API ===");

    // Update wallet webhook
    let webhook_config = WebhookConfig {
        url: "https://your-server.com/wallet-webhook".to_string(),
        events: vec![
            WalletWebhookEvent::UtxoExternalReceive,
            WalletWebhookEvent::ExternalSpend,
        ],
        secret: Some("webhook_secret_123".to_string()),
    };

    let webhook_request = coinpayments::UpdateWebhookRequest {
        url: webhook_config.url,
        events: webhook_config
            .events
            .iter()
            .map(|e| format!("{:?}", e))
            .collect(),
        secret: webhook_config.secret,
        is_active: Some(true),
    };

    match client
        .update_wallet_webhook_v3("demo-btc-wallet", "4", webhook_request)
        .await
    {
        Ok(_) => {
            println!("✅ Updated wallet webhook configuration");
        }
        Err(e) => println!("❌ Failed to update webhook: {}", e),
    }

    // === UTILITY FUNCTIONS DEMO ===
    println!("\n🛠️  === UTILITY FUNCTIONS ===");

    // Demonstrate helper functions
    use coinpayments::{
        format_amount, from_smallest_unit, is_valid_bitcoin_address, is_valid_email,
        to_smallest_unit,
    };

    println!("📧 Email validation:");
    println!(
        "  'user@example.com': {}",
        is_valid_email("user@example.com")
    );
    println!("  'invalid-email': {}", is_valid_email("invalid-email"));

    println!("₿ Bitcoin address validation:");
    let btc_address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
    println!(
        "  '{}': {}",
        btc_address,
        is_valid_bitcoin_address(btc_address)
    );

    println!("🔢 Amount formatting:");
    println!(
        "  1.23456789 -> 2 decimals: {}",
        format_amount(1.23456789, 2)
    );
    println!("  1.0 -> 8 decimals: {}", format_amount(1.0, 8));

    println!("⚡ Unit conversion (Bitcoin):");
    let btc_amount = 1.0;
    let satoshis = to_smallest_unit(btc_amount, 8);
    let back_to_btc = from_smallest_unit(satoshis, 8);
    println!(
        "  {} BTC = {} satoshis = {} BTC",
        btc_amount, satoshis, back_to_btc
    );

    println!("\n🎉 Demo completed successfully!");

    Ok(())
}

/// Example webhook handler function
#[allow(dead_code)]
async fn handle_webhook(
    payload: &[u8],
    headers: std::collections::HashMap<String, String>,
) -> Result<()> {
    use coinpayments::{parse_webhook_headers, verify_webhook_signature};

    let private_key = "your_private_key";

    // Parse webhook headers
    let webhook_headers = parse_webhook_headers(&headers)?;

    // Verify signature
    if !verify_webhook_signature(private_key, &webhook_headers, payload) {
        println!("❌ Invalid webhook signature");
        return Ok(());
    }

    // Validate timestamp (5 minutes tolerance)
    use coinpayments::webhooks::is_webhook_timestamp_valid;
    if !is_webhook_timestamp_valid(&webhook_headers.timestamp, 300) {
        println!("❌ Webhook timestamp too old");
        return Ok(());
    }

    // Process the webhook payload
    let payload_str = String::from_utf8_lossy(payload);
    println!("✅ Valid webhook received: {}", payload_str);

    Ok(())
}

/// Example of using the legacy API for backward compatibility
#[allow(dead_code)]
async fn legacy_api_example() -> Result<()> {
    use coinpayments::endpoints::{
        CoinPaymentsLegacyClient, CreateTransactionRequest, CreateWithdrawalRequest,
    };

    // Create legacy client
    let legacy_client = CoinPaymentsLegacyClient::new("public_key", "private_key");

    // Get rates (legacy API)
    match legacy_client.get_rates(Some(true)).await {
        Ok(rates) => {
            println!("📊 Legacy API rates: {} currencies", rates.rates.len());
        }
        Err(e) => println!("❌ Legacy API error: {}", e),
    }

    // Create transaction (legacy API)
    let transaction_request = CreateTransactionRequest::new(10.0, "USD", "BTC")
        .with_buyer_email("customer@example.com")
        .with_item("Test Item", Some("SKU-123".to_string()));

    match legacy_client.create_transaction(&transaction_request).await {
        Ok(transaction) => {
            println!("✅ Legacy transaction created: {}", transaction.txn_id);
        }
        Err(e) => println!("❌ Legacy transaction failed: {}", e),
    }

    Ok(())
}
