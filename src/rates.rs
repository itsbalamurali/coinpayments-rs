//! Exchange rates API endpoints for CoinPayments API
//!
//! This module provides functionality for:
//! - Getting current conversion rates between currencies
//! - Real-time rate information
//! - Rate filtering and querying

use crate::{CoinPaymentsClient, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// === Rate Types ===

/// Exchange rate information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExchangeRate {
    pub from_currency_id: String,
    pub to_currency_id: String,
    pub rate: String,
    pub rate_f: f64,
    pub last_updated: String,
    pub market_cap: Option<String>,
    pub volume_24h: Option<String>,
    pub change_24h: Option<String>,
    pub change_percentage_24h: Option<f64>,
}

/// Response for getting exchange rates
#[derive(Debug, Deserialize, Serialize)]
pub struct GetRatesResponse {
    pub rates: Vec<ExchangeRate>,
    pub base_currency: Option<String>,
    pub last_updated: String,
    pub pagination: Option<RatePaginationInfo>,
}

/// Pagination information for rates
#[derive(Debug, Deserialize, Serialize)]
pub struct RatePaginationInfo {
    pub page: u32,
    pub per_page: u32,
    pub total: u32,
    pub total_pages: u32,
}

/// Rate query parameters
#[derive(Debug, Clone)]
pub struct RateQuery {
    pub from_currency: Option<String>,
    pub to_currency: Option<String>,
    pub currencies: Option<Vec<String>>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub include_inactive: Option<bool>,
}

impl Default for RateQuery {
    fn default() -> Self {
        Self {
            from_currency: None,
            to_currency: None,
            currencies: None,
            page: None,
            per_page: None,
            include_inactive: Some(false),
        }
    }
}

impl RateQuery {
    /// Create a new rate query
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base currency to get rates from
    pub fn from_currency(mut self, currency: impl Into<String>) -> Self {
        self.from_currency = Some(currency.into());
        self
    }

    /// Set the target currency to get rates to
    pub fn to_currency(mut self, currency: impl Into<String>) -> Self {
        self.to_currency = Some(currency.into());
        self
    }

    /// Set specific currencies to get rates for
    pub fn currencies(mut self, currencies: Vec<String>) -> Self {
        self.currencies = Some(currencies);
        self
    }

    /// Set pagination
    pub fn page(mut self, page: u32, per_page: Option<u32>) -> Self {
        self.page = Some(page);
        if let Some(per_page) = per_page {
            self.per_page = Some(per_page);
        }
        self
    }

    /// Include inactive currencies in the results
    pub fn include_inactive(mut self, include: bool) -> Self {
        self.include_inactive = Some(include);
        self
    }

    /// Convert to query parameters
    pub fn to_query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = Vec::new();

        if let Some(ref from) = self.from_currency {
            params.push(("from", from.clone()));
        }
        if let Some(ref to) = self.to_currency {
            params.push(("to", to.clone()));
        }
        if let Some(ref currencies) = self.currencies {
            params.push(("currencies", currencies.join(",")));
        }
        if let Some(page) = self.page {
            params.push(("page", page.to_string()));
        }
        if let Some(per_page) = self.per_page {
            params.push(("per_page", per_page.to_string()));
        }
        if let Some(include_inactive) = self.include_inactive {
            params.push(("include_inactive", include_inactive.to_string()));
        }

        params
    }
}

impl CoinPaymentsClient {
    /// Get current conversion rates between currencies
    ///
    /// # Arguments
    /// * `query` - Optional query parameters to filter rates
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    ///
    /// // Get all rates
    /// let all_rates = client.get_rates(None).await?;
    ///
    /// // Get rates from Bitcoin to other currencies
    /// let btc_rates = client.get_rates(Some(
    ///     RateQuery::new().from_currency("4") // Bitcoin
    /// )).await?;
    ///
    /// // Get specific currency pair rate
    /// let btc_to_eth = client.get_rates(Some(
    ///     RateQuery::new()
    ///         .from_currency("4")  // Bitcoin
    ///         .to_currency("61")   // Ethereum
    /// )).await?;
    /// ```
    pub async fn get_rates(&self, query: Option<RateQuery>) -> Result<GetRatesResponse> {
        let query_params = match &query {
            Some(q) => q.to_query_params(),
            None => Vec::new(),
        };

        self.get_request("v2/rates", &query_params).await
    }

    /// Get rate for a specific currency pair
    ///
    /// # Arguments
    /// * `from_currency` - Source currency ID
    /// * `to_currency` - Target currency ID
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let rate = client.get_rate("4", "61").await?; // BTC to ETH
    /// ```
    pub async fn get_rate(&self, from_currency: &str, to_currency: &str) -> Result<ExchangeRate> {
        let query = RateQuery::new()
            .from_currency(from_currency)
            .to_currency(to_currency);

        let response: GetRatesResponse = self.get_rates(Some(query)).await?;

        response
            .rates
            .into_iter()
            .find(|rate| {
                rate.from_currency_id == from_currency && rate.to_currency_id == to_currency
            })
            .ok_or_else(|| crate::CoinPaymentsError::Api {
                message: format!("Rate not found for {} to {}", from_currency, to_currency),
            })
    }

    /// Get all rates for a specific currency
    ///
    /// # Arguments
    /// * `currency_id` - Currency ID to get rates for
    /// * `as_base` - If true, get rates from this currency to others; if false, get rates to this currency
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    ///
    /// // Get rates from Bitcoin to all other currencies
    /// let btc_rates = client.get_currency_rates("4", true).await?;
    ///
    /// // Get rates from all currencies to Bitcoin
    /// let to_btc_rates = client.get_currency_rates("4", false).await?;
    /// ```
    pub async fn get_currency_rates(
        &self,
        currency_id: &str,
        as_base: bool,
    ) -> Result<Vec<ExchangeRate>> {
        let query = if as_base {
            RateQuery::new().from_currency(currency_id)
        } else {
            RateQuery::new().to_currency(currency_id)
        };

        let response: GetRatesResponse = self.get_rates(Some(query)).await?;
        Ok(response.rates)
    }

    /// Get rates for multiple specific currencies
    ///
    /// # Arguments
    /// * `currency_ids` - List of currency IDs to get rates for
    ///
    /// # Example
    /// ```rust
    /// let client = CoinPaymentsClient::new("client_id", "client_secret");
    /// let currencies = vec!["4".to_string(), "61".to_string(), "3".to_string()]; // BTC, ETH, LTC
    /// let rates = client.get_multiple_currency_rates(currencies).await?;
    /// ```
    pub async fn get_multiple_currency_rates(
        &self,
        currency_ids: Vec<String>,
    ) -> Result<Vec<ExchangeRate>> {
        let query = RateQuery::new().currencies(currency_ids);
        let response: GetRatesResponse = self.get_rates(Some(query)).await?;
        Ok(response.rates)
    }
}

// === Helper Functions ===

/// Calculate conversion amount using exchange rate
pub fn calculate_conversion(amount: f64, rate: &ExchangeRate) -> f64 {
    amount * rate.rate_f
}

/// Find rate between two currencies in a list of rates
pub fn find_rate<'a>(
    rates: &'a [ExchangeRate],
    from_currency: &str,
    to_currency: &str,
) -> Option<&'a ExchangeRate> {
    rates
        .iter()
        .find(|rate| rate.from_currency_id == from_currency && rate.to_currency_id == to_currency)
}

/// Get all rates for a specific currency as base
pub fn get_base_currency_rates<'a>(
    rates: &'a [ExchangeRate],
    base_currency: &str,
) -> Vec<&'a ExchangeRate> {
    rates
        .iter()
        .filter(|rate| rate.from_currency_id == base_currency)
        .collect()
}

/// Get all rates for a specific currency as target
pub fn get_target_currency_rates<'a>(
    rates: &'a [ExchangeRate],
    target_currency: &str,
) -> Vec<&'a ExchangeRate> {
    rates
        .iter()
        .filter(|rate| rate.to_currency_id == target_currency)
        .collect()
}

/// Convert rates to a HashMap for quick lookup
pub fn rates_to_hashmap(rates: &[ExchangeRate]) -> HashMap<(String, String), &ExchangeRate> {
    rates
        .iter()
        .map(|rate| {
            (
                (rate.from_currency_id.clone(), rate.to_currency_id.clone()),
                rate,
            )
        })
        .collect()
}

/// Check if a rate has changed significantly (more than threshold percentage)
pub fn rate_changed_significantly(rate: &ExchangeRate, threshold_percent: f64) -> bool {
    rate.change_percentage_24h
        .map(|change| change.abs() > threshold_percent)
        .unwrap_or(false)
}

/// Sort rates by 24h change percentage (highest first)
pub fn sort_rates_by_change(rates: &mut [ExchangeRate], descending: bool) {
    rates.sort_by(|a, b| {
        let a_change = a.change_percentage_24h.unwrap_or(0.0);
        let b_change = b.change_percentage_24h.unwrap_or(0.0);

        if descending {
            b_change
                .partial_cmp(&a_change)
                .unwrap_or(std::cmp::Ordering::Equal)
        } else {
            a_change
                .partial_cmp(&b_change)
                .unwrap_or(std::cmp::Ordering::Equal)
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rate(from: &str, to: &str, rate: f64, change: Option<f64>) -> ExchangeRate {
        ExchangeRate {
            from_currency_id: from.to_string(),
            to_currency_id: to.to_string(),
            rate: rate.to_string(),
            rate_f: rate,
            last_updated: "2023-01-01T00:00:00Z".to_string(),
            market_cap: None,
            volume_24h: None,
            change_24h: None,
            change_percentage_24h: change,
        }
    }

    #[test]
    fn test_calculate_conversion() {
        let rate = create_test_rate("4", "61", 15.5, None);
        let result = calculate_conversion(1.0, &rate);
        assert_eq!(result, 15.5);
    }

    #[test]
    fn test_find_rate() {
        let rates = vec![
            create_test_rate("4", "61", 15.5, None),
            create_test_rate("4", "3", 25.0, None),
        ];

        let found = find_rate(&rates, "4", "61");
        assert!(found.is_some());
        assert_eq!(found.unwrap().rate_f, 15.5);

        let not_found = find_rate(&rates, "61", "4");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_rate_query_builder() {
        let query = RateQuery::new()
            .from_currency("4")
            .to_currency("61")
            .page(1, Some(10))
            .include_inactive(true);

        let params = query.to_query_params();
        assert!(params.contains(&("from", "4".to_string())));
        assert!(params.contains(&("to", "61".to_string())));
        assert!(params.contains(&("page", "1".to_string())));
        assert!(params.contains(&("per_page", "10".to_string())));
        assert!(params.contains(&("include_inactive", "true".to_string())));
    }

    #[test]
    fn test_rate_changed_significantly() {
        let rate_with_big_change = create_test_rate("4", "61", 15.5, Some(10.5));
        let rate_with_small_change = create_test_rate("4", "3", 25.0, Some(2.0));
        let rate_without_change = create_test_rate("61", "3", 1.6, None);

        assert!(rate_changed_significantly(&rate_with_big_change, 5.0));
        assert!(!rate_changed_significantly(&rate_with_small_change, 5.0));
        assert!(!rate_changed_significantly(&rate_without_change, 5.0));
    }

    #[test]
    fn test_sort_rates_by_change() {
        let mut rates = vec![
            create_test_rate("4", "61", 15.5, Some(2.0)),
            create_test_rate("4", "3", 25.0, Some(10.0)),
            create_test_rate("61", "3", 1.6, Some(-5.0)),
        ];

        sort_rates_by_change(&mut rates, true); // Descending
        assert_eq!(rates[0].change_percentage_24h.unwrap(), 10.0);
        assert_eq!(rates[1].change_percentage_24h.unwrap(), 2.0);
        assert_eq!(rates[2].change_percentage_24h.unwrap(), -5.0);
    }
}
