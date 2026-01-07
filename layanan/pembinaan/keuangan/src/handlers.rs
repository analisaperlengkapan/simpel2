use axum::{
    extract::{State, Path},
    Json, http::StatusCode,
};
use deadpool_postgres::Pool;
use uuid::Uuid;
use crate::models::{BudgetItem, Transaction, CreateBudgetRequest, CreateTransactionRequest, FinancialMetrics};

// Dashboard Metrics
pub async fn get_metrics(State(pool): State<Pool>) -> Result<Json<FinancialMetrics>, (StatusCode, String)> {
    let client = pool.get().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Calculate totals
    let budget_row = client.query_one("SELECT COALESCE(SUM(allocated_amount), 0) as total, COALESCE(SUM(realized_amount), 0) as realized FROM keuangan.budgets", &[])
        .await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total_budget: f64 = budget_row.get("total");
    let total_realization: f64 = budget_row.get("realized");

    let pending_count: i64 = client.query_one("SELECT COUNT(*) FROM keuangan.transactions WHERE status = 'Pending'", &[])
        .await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?.get(0);

    let approved_count: i64 = client.query_one("SELECT COUNT(*) FROM keuangan.transactions WHERE status = 'Approved'", &[])
        .await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?.get(0);

    let metrics = FinancialMetrics {
        total_budget,
        total_realization,
        utilization_percentage: if total_budget > 0.0 { (total_realization / total_budget) * 100.0 } else { 0.0 },
        pending_transactions: pending_count,
        approved_transactions: approved_count,
        monthly_variance: 0.0, // Placeholder calculation
        cash_flow: total_budget - total_realization,
        budget_variance: 0.0, // Placeholder
    };

    Ok(Json(metrics))
}

// Budget Handlers
pub async fn list_budgets(State(pool): State<Pool>) -> Result<Json<Vec<BudgetItem>>, (StatusCode, String)> {
    let client = pool.get().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let rows = client.query("SELECT * FROM keuangan.budgets ORDER BY created_at DESC", &[])
        .await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let budgets: Vec<BudgetItem> = rows.into_iter().map(BudgetItem::from).collect();
    Ok(Json(budgets))
}

pub async fn create_budget(
    State(pool): State<Pool>,
    Json(payload): Json<CreateBudgetRequest>,
) -> Result<Json<BudgetItem>, (StatusCode, String)> {
    let client = pool.get().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let row = client.query_one(
        r#"
        INSERT INTO keuangan.budgets (category, subcategory, budget_code, allocated_amount, priority, responsible_unit)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
        &[
            &payload.category,
            &payload.subcategory,
            &payload.budget_code,
            &payload.allocated_amount,
            &payload.priority,
            &payload.responsible_unit
        ],
    ).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(BudgetItem::from(row)))
}

// Transaction Handlers
pub async fn list_transactions(State(pool): State<Pool>) -> Result<Json<Vec<Transaction>>, (StatusCode, String)> {
    let client = pool.get().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let rows = client.query("SELECT * FROM keuangan.transactions ORDER BY transaction_date DESC LIMIT 10", &[])
        .await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let transactions: Vec<Transaction> = rows.into_iter().map(Transaction::from).collect();
    Ok(Json(transactions))
}

pub async fn create_transaction(
    State(pool): State<Pool>,
    Json(payload): Json<CreateTransactionRequest>,
) -> Result<Json<Transaction>, (StatusCode, String)> {
    let client = pool.get().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Transaction code generation (simple)
    let code = format!("TRX-{}", Uuid::new_v4().simple().to_string()[0..8].to_uppercase());

    let row = client.query_one(
        r#"
        INSERT INTO keuangan.transactions (transaction_code, description, amount, transaction_type, budget_item_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
        &[
            &code,
            &payload.description,
            &payload.amount,
            &payload.transaction_type,
            &payload.budget_item_id
        ],
    ).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // If it's an expense and linked to a budget, update the realized amount
    if payload.transaction_type == "Expense" {
        if let Some(budget_id) = payload.budget_item_id {
            let _ = client.execute(
                "UPDATE keuangan.budgets SET realized_amount = realized_amount + $1 WHERE id = $2",
                &[&payload.amount, &budget_id],
            ).await;
        }
    }

    Ok(Json(Transaction::from(row)))
}
