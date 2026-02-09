-- Pidmil Service Database Migrations
-- This file contains embedded migrations for the Pidmil service

-- Create schema if not exists
CREATE SCHEMA IF NOT EXISTS pidmil;

-- Create cases table
CREATE TABLE IF NOT EXISTS pidmil.cases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_number VARCHAR(100) UNIQUE NOT NULL,
    suspect_name VARCHAR(255) NOT NULL,
    charge TEXT,
    status VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_pidmil_cases_status ON pidmil.cases(status);
CREATE INDEX IF NOT EXISTS idx_pidmil_cases_created ON pidmil.cases(created_at);
