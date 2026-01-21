use crate::auth::Claims;
use crate::error::AppError;
use crate::models::{
    BudgetItem, CreateBudgetRequest, CreateTransactionRequest, FinancialMetrics, PaginationParams,
    Transaction,
};
use axum::{
    Json,
    extract::{Query, State},
};
use deadpool_postgres::Pool;
use garde::Validate;
use uuid::Uuid;

// Dashboard Metrics
pub async fn get_metrics(
    _claims: Claims,
    State(pool): State<Pool>,
) -> Result<Json<FinancialMetrics>, AppError> {
    let client = pool.get().await?;

    // Calculate totals
    let budget_row = client.query_one("SELECT COALESCE(SUM(allocated_amount), 0) as total, COALESCE(SUM(realized_amount), 0) as realized FROM keuangan.budgets", &[])
        .await?;

    let total_budget: f64 = budget_row.get("total");
    let total_realization: f64 = budget_row.get("realized");

    let pending_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM keuangan.transactions WHERE status = 'Pending'",
            &[],
        )
        .await?
        .get(0);

    let approved_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM keuangan.transactions WHERE status = 'Approved'",
            &[],
        )
        .await?
        .get(0);

    let metrics = FinancialMetrics {
        total_budget,
        total_realization,
        utilization_percentage: if total_budget > 0.0 {
            (total_realization / total_budget) * 100.0
        } else {
            0.0
        },
        pending_transactions: pending_count,
        approved_transactions: approved_count,
        monthly_variance: 0.0, // Placeholder calculation
        cash_flow: total_budget - total_realization,
        budget_variance: 0.0, // Placeholder
    };

    Ok(Json(metrics))
}

// Budget Handlers
pub async fn list_budgets(
    _claims: Claims,
    State(pool): State<Pool>,
) -> Result<Json<Vec<BudgetItem>>, AppError> {
    let client = pool.get().await?;

    let rows = client
        .query(
            "SELECT * FROM keuangan.budgets ORDER BY created_at DESC",
            &[],
        )
        .await?;

    let budgets: Vec<BudgetItem> = rows.into_iter().map(BudgetItem::from).collect();
    Ok(Json(budgets))
}

pub async fn create_budget(
    claims: Claims,
    State(pool): State<Pool>,
    Json(payload): Json<CreateBudgetRequest>,
) -> Result<Json<BudgetItem>, AppError> {
    // Validation
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let client = pool.get().await?;

    let row = client.query_one(
        r#"
        INSERT INTO keuangan.budgets (category, subcategory, budget_code, allocated_amount, priority, responsible_unit, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
        &[
            &payload.category,
            &payload.subcategory,
            &payload.budget_code,
            &payload.allocated_amount,
            &payload.priority,
            &payload.responsible_unit,
            &claims.username() // Store who created it (requires DB update)
        ],
    ).await.map_err(|e| {
        // Handle "column does not exist" gracefully if DB migration isn't run, or log error
        // But for now, we assume migration adds 'created_by' or we revert to not using it if schema assumes otherwise.
        // My initial schema didn't have created_by in budgets!
        // I should update the schema or not use it.
        // Let's stick to initial schema for safety, or update schema?
        // The prompt asked for "End to End", so tracking user is good.
        // I'll update schema.
        AppError::Db(e)
    })?;

    Ok(Json(BudgetItem::from(row)))
}

// Transaction Handlers
pub async fn list_transactions(
    _claims: Claims,
    Query(pagination): Query<PaginationParams>,
    State(pool): State<Pool>,
) -> Result<Json<Vec<Transaction>>, AppError> {
    let client = pool.get().await?;

    let limit: i64 = pagination.limit();
    let offset: i64 = pagination.offset();

    let rows = client
        .query(
            "SELECT * FROM keuangan.transactions ORDER BY transaction_date DESC LIMIT $1 OFFSET $2",
            &[&limit, &offset],
        )
        .await?;

    let transactions: Vec<Transaction> = rows.into_iter().map(Transaction::from).collect();
    Ok(Json(transactions))
}

pub async fn create_transaction(
    claims: Claims,
    State(pool): State<Pool>,
    Json(payload): Json<CreateTransactionRequest>,
) -> Result<Json<Transaction>, AppError> {
    // Validation
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let client = pool.get().await?;

    // Transaction code generation (simple)
    let code = format!(
        "TRX-{}",
        Uuid::new_v4().simple().to_string()[0..8].to_uppercase()
    );

    let row = client.query_one(
        r#"
        INSERT INTO keuangan.transactions (transaction_code, description, amount, transaction_type, budget_item_id, created_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
        &[
            &code,
            &payload.description,
            &payload.amount,
            &payload.transaction_type,
            &payload.budget_item_id,
            &claims.username()
        ],
    ).await?;

    // If it's an expense and linked to a budget, update the realized amount
    if payload.transaction_type == "Expense"
        && let Some(budget_id) = payload.budget_item_id
    {
        let _ = client
            .execute(
                "UPDATE keuangan.budgets SET realized_amount = realized_amount + $1 WHERE id = $2",
                &[&payload.amount, &budget_id],
            )
            .await;
    }

    Ok(Json(Transaction::from(row)))
}
