use crate::pages::{FinancialMetrics, BudgetItem, Transaction, BudgetPriority, BudgetStatus, TransactionType, TransactionStatus};
use gloo_net::http::Request;
use leptos::prelude::*;
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

pub async fn fetch_metrics() -> Result<FinancialMetrics, String> {
    Request::get(&format!("{}/metrics", API_BASE_URL))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<FinancialMetrics>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn fetch_budgets() -> Result<Vec<BudgetItem>, String> {
    Request::get(&format!("{}/budgets", API_BASE_URL))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<BudgetItem>>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn fetch_transactions() -> Result<Vec<Transaction>, String> {
    Request::get(&format!("{}/transactions", API_BASE_URL))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<Transaction>>()
        .await
        .map_err(|e| e.to_string())
}
