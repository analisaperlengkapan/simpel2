use crate::pages::{FinancialMetrics, BudgetItem, Transaction};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

// Use relative path for production (via gateway) or dev proxy
const API_BASE_URL: &str = "/api/v1/keuangan";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateBudgetRequest {
    pub category: String,
    pub subcategory: String,
    pub budget_code: String,
    pub allocated_amount: f64,
    pub priority: String,
    pub responsible_unit: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateTransactionRequest {
    pub description: String,
    pub amount: f64,
    pub transaction_type: String, // Income, Expense
    pub budget_item_id: Option<uuid::Uuid>,
}

fn get_token() -> Option<String> {
    use web_sys::window;
    window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|storage| storage.get_item("auth_token").ok())
        .flatten()
}

fn authenticated_request(url: &str) -> Request {
    let req = Request::get(url);
    if let Some(token) = get_token() {
        req.header("Authorization", &format!("Bearer {}", token))
    } else {
        req
    }
}

fn authenticated_post(url: &str, body: &impl Serialize) -> Result<Request, String> {
    let mut req = Request::post(url)
        .json(body)
        .map_err(|e| e.to_string())?;

    if let Some(token) = get_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    Ok(req)
}

pub async fn fetch_metrics() -> Result<FinancialMetrics, String> {
    authenticated_request(&format!("{}/metrics", API_BASE_URL))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<FinancialMetrics>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn fetch_budgets() -> Result<Vec<BudgetItem>, String> {
    authenticated_request(&format!("{}/budgets", API_BASE_URL))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<BudgetItem>>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn create_budget(request: CreateBudgetRequest) -> Result<BudgetItem, String> {
    authenticated_post(&format!("{}/budgets", API_BASE_URL), &request)?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<BudgetItem>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn fetch_transactions() -> Result<Vec<Transaction>, String> {
    authenticated_request(&format!("{}/transactions", API_BASE_URL))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<Transaction>>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn create_transaction(request: CreateTransactionRequest) -> Result<Transaction, String> {
    authenticated_post(&format!("{}/transactions", API_BASE_URL), &request)?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Transaction>()
        .await
        .map_err(|e| e.to_string())
}
