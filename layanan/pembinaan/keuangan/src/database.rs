use anyhow::Result;
use deadpool_postgres::Pool;

pub async fn init_db(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    // Create schema
    client.execute(
        "CREATE SCHEMA IF NOT EXISTS keuangan",
        &[],
    ).await?;

    // Create budgets table
    client.execute(
        r#"
        CREATE TABLE IF NOT EXISTS keuangan.budgets (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            category VARCHAR NOT NULL,
            subcategory VARCHAR NOT NULL,
            budget_code VARCHAR NOT NULL,
            allocated_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
            realized_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
            priority VARCHAR NOT NULL DEFAULT 'Medium',
            status VARCHAR NOT NULL DEFAULT 'Active',
            start_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            end_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            responsible_unit VARCHAR NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#,
        &[],
    ).await?;

    // Create transactions table
    client.execute(
        r#"
        CREATE TABLE IF NOT EXISTS keuangan.transactions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            transaction_code VARCHAR NOT NULL,
            description TEXT NOT NULL,
            amount DOUBLE PRECISION NOT NULL,
            transaction_type VARCHAR NOT NULL,
            status VARCHAR NOT NULL DEFAULT 'Pending',
            budget_item_id UUID REFERENCES keuangan.budgets(id),
            transaction_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            approval_date TIMESTAMPTZ,
            approved_by VARCHAR,
            supporting_documents TEXT[],
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#,
        &[],
    ).await?;

    tracing::info!("Database initialized successfully");
    Ok(())
}
