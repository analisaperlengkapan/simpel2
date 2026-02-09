-- Pengawasan Service - Initial Schema Migration
CREATE SCHEMA IF NOT EXISTS pengawasan;

-- Create schedules table
CREATE TABLE IF NOT EXISTS pengawasan.schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    scheduled_date DATE NOT NULL,
    location VARCHAR(255),
    inspectors TEXT[], -- Array of inspector names
    status VARCHAR(50) DEFAULT 'planned',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create inspection_reports table
CREATE TABLE IF NOT EXISTS pengawasan.inspection_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schedule_id UUID REFERENCES pengawasan.schedules(id) ON DELETE CASCADE,
    findings TEXT NOT NULL,
    recommendations TEXT,
    severity VARCHAR(50),
    created_by VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_pengawasan_schedules_date ON pengawasan.schedules(scheduled_date);
CREATE INDEX IF NOT EXISTS idx_pengawasan_schedules_status ON pengawasan.schedules(status);
CREATE INDEX IF NOT EXISTS idx_pengawasan_reports_schedule ON pengawasan.inspection_reports(schedule_id);
CREATE INDEX IF NOT EXISTS idx_pengawasan_reports_severity ON pengawasan.inspection_reports(severity);
