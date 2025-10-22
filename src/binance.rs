use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::Value;
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

type HmacSha256 = Hmac<Sha256>;

/// Binance REST API client
#[derive(Clone)]
pub struct BinanceClient {
    endpoint: String,
    api_key: Option<String>,
    secret_key: Option<String>,
    client: Client,
}

#[derive(Debug, Deserialize)]
pub struct ServerTime {
    #[serde(rename = "serverTime")]
    pub server_time: u64,
}

#[derive(Debug, Deserialize)]
pub struct SymbolPrice {
    pub symbol: String,
    pub price: String,
}

#[derive(Debug, Deserialize)]
pub struct AccountInfo {
    #[serde(rename = "makerCommission")]
    pub maker_commission: i32,
    #[serde(rename = "takerCommission")]
    pub taker_commission: i32,
    #[serde(rename = "buyerCommission")]
    pub buyer_commission: i32,
    #[serde(rename = "sellerCommission")]
    pub seller_commission: i32,
    #[serde(rename = "canTrade")]
    pub can_trade: bool,
    #[serde(rename = "canWithdraw")]
    pub can_withdraw: bool,
    #[serde(rename = "canDeposit")]
    pub can_deposit: bool,
    pub balances: Vec<Balance>,
}

#[derive(Debug, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

impl BinanceClient {
    /// Create a new Binance REST API client
    pub fn new(endpoint: &str, api_key: Option<&str>, secret_key: Option<&str>) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            api_key: api_key.map(|s| s.to_string()),
            secret_key: secret_key.map(|s| s.to_string()),
            client: Client::new(),
        }
    }

    /// Get the API endpoint
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Generate HMAC-SHA256 signature
    fn sign(&self, query_string: &str) -> Result<String> {
        let secret = self
            .secret_key
            .as_ref()
            .ok_or_else(|| anyhow!("Secret key not configured"))?;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| anyhow!("Failed to create HMAC: {}", e))?;

        mac.update(query_string.as_bytes());
        let result = mac.finalize();
        let signature = hex::encode(result.into_bytes());

        Ok(signature)
    }

    /// Get current server timestamp in milliseconds
    fn get_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Make a public GET request (no authentication required)
    async fn public_get(&self, endpoint: &str) -> Result<Response> {
        let url = format!("{}{}", self.endpoint, endpoint);
        debug!("Public GET: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API error {}: {}", status, error_text));
        }

        Ok(response)
    }

    /// Make a signed GET request (requires API key and secret)
    async fn signed_get(&self, endpoint: &str, params: HashMap<String, String>) -> Result<Response> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow!("API key not configured"))?;

        // Add timestamp
        let mut params = params;
        params.insert("timestamp".to_string(), Self::get_timestamp().to_string());

        // Build query string
        let query_string = serde_urlencoded::to_string(&params)?;

        // Generate signature
        let signature = self.sign(&query_string)?;

        // Build final URL with signature
        let url = format!("{}{}?{}&signature={}", self.endpoint, endpoint, query_string, signature);
        debug!("Signed GET: {}", endpoint);

        let response = self
            .client
            .get(&url)
            .header("X-MBX-APIKEY", api_key)
            .send()
            .await
            .map_err(|e| anyhow!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API error {}: {}", status, error_text));
        }

        Ok(response)
    }

    /// Get server time
    pub async fn get_server_time(&self) -> Result<u64> {
        let response = self.public_get("/api/v3/time").await?;
        let server_time: ServerTime = response.json().await?;
        info!("Server time: {}", server_time.server_time);
        Ok(server_time.server_time)
    }

    /// Get exchange info
    pub async fn get_exchange_info(&self) -> Result<Value> {
        let response = self.public_get("/api/v3/exchangeInfo").await?;
        let info: Value = response.json().await?;
        debug!("Exchange info retrieved");
        Ok(info)
    }

    /// Get current price for a symbol
    pub async fn get_symbol_price(&self, symbol: &str) -> Result<f64> {
        let endpoint = format!("/api/v3/ticker/price?symbol={}", symbol);
        let response = self.public_get(&endpoint).await?;
        let price_info: SymbolPrice = response.json().await?;
        let price = price_info.price.parse::<f64>()?;
        debug!("Price for {}: {}", symbol, price);
        Ok(price)
    }

    /// Get account information (requires authentication)
    pub async fn get_account(&self) -> Result<AccountInfo> {
        let params = HashMap::new();
        let response = self.signed_get("/api/v3/account", params).await?;
        let account: AccountInfo = response.json().await?;
        info!("Account info retrieved, can_trade: {}", account.can_trade);
        Ok(account)
    }

    /// Test connectivity
    pub async fn ping(&self) -> Result<()> {
        self.public_get("/api/v3/ping").await?;
        debug!("Ping successful");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = BinanceClient::new(
            "https://api.binance.com",
            Some("test_key"),
            Some("test_secret"),
        );
        assert_eq!(client.endpoint(), "https://api.binance.com");
    }

    #[test]
    fn test_signature_generation() {
        let client = BinanceClient::new(
            "https://api.binance.com",
            Some("test_key"),
            Some("test_secret"),
        );

        let signature = client.sign("symbol=BTCUSDT&timestamp=1234567890").unwrap();
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64); // SHA256 produces 64 hex characters
    }

    #[test]
    fn test_timestamp_generation() {
        let ts1 = BinanceClient::get_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ts2 = BinanceClient::get_timestamp();
        assert!(ts2 > ts1);
    }
}
