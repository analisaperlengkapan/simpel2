use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tokio_postgres::Row;
use garde::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetItem {
    pub id: Uuid,
    pub category: String,
    pub subcategory: String,
    pub budget_code: String,
    pub allocated_amount: f64,
    pub realized_amount: f64,
    pub remaining_amount: f64,
    pub priority: String,
    pub status: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub responsible_unit: String,
}

impl From<Row> for BudgetItem {
    fn from(row: Row) -> Self {
        let allocated: f64 = row.get("allocated_amount");
        let realized: f64 = row.get("realized_amount");
        Self {
            id: row.get("id"),
            category: row.get("category"),
            subcategory: row.get("subcategory"),
            budget_code: row.get("budget_code"),
            allocated_amount: allocated,
            realized_amount: realized,
            remaining_amount: allocated - realized,
            priority: row.get("priority"),
            status: row.get("status"),
            start_date: row.get("start_date"),
            end_date: row.get("end_date"),
            responsible_unit: row.get("responsible_unit"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub transaction_code: String,
    pub description: String,
    pub amount: f64,
    pub transaction_type: String,
    pub status: String,
    pub budget_item_id: Option<Uuid>,
    pub transaction_date: DateTime<Utc>,
    pub approval_date: Option<DateTime<Utc>>,
    pub approved_by: Option<String>,
    pub supporting_documents: Option<Vec<String>>,
}

impl From<Row> for Transaction {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            transaction_code: row.get("transaction_code"),
            description: row.get("description"),
            amount: row.get("amount"),
            transaction_type: row.get("transaction_type"),
            status: row.get("status"),
            budget_item_id: row.get("budget_item_id"),
            transaction_date: row.get("transaction_date"),
            approval_date: row.get("approval_date"),
            approved_by: row.get("approved_by"),
            supporting_documents: row.get("supporting_documents"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateBudgetRequest {
    #[garde(length(min = 3))]
    pub category: String,
    #[garde(length(min = 3))]
    pub subcategory: String,
    #[garde(length(min = 3))]
    pub budget_code: String,
    #[garde(range(min = 0.0))]
    pub allocated_amount: f64,
    #[garde(skip)] // Simple string check in handler or enum later
    pub priority: String,
    #[garde(length(min = 3))]
    pub responsible_unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTransactionRequest {
    #[garde(length(min = 5))]
    pub description: String,
    #[garde(range(min = 0.0))]
    pub amount: f64,
    #[garde(custom(validate_transaction_type))]
    pub transaction_type: String, // Income, Expense
    #[garde(skip)]
    pub budget_item_id: Option<Uuid>,
}

fn validate_transaction_type(value: &str, _ctx: &()) -> garde::Result {
    if value == "Income" || value == "Expense" {
        Ok(())
    } else {
        Err(garde::Error::new("must be 'Income' or 'Expense'"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialMetrics {
    pub total_budget: f64,
    pub total_realization: f64,
    pub utilization_percentage: f64,
    pub pending_transactions: i64,
    pub approved_transactions: i64,
    pub monthly_variance: f64,
    pub cash_flow: f64,
    pub budget_variance: f64,
}
