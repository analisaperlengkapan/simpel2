-- Migration for in-app notifications
-- This adds support for in-app notification storage and tracking

CREATE SCHEMA IF NOT EXISTS notifikasi;

-- In-app notifications table
CREATE TABLE IF NOT EXISTS notifikasi.in_app_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    notification_type VARCHAR(50) NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    priority VARCHAR(20) NOT NULL DEFAULT 'normal',
    category VARCHAR(50) DEFAULT 'info',
    action_url TEXT,
    metadata JSONB,
    read BOOLEAN NOT NULL DEFAULT FALSE,
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

-- Indexes for performance
CREATE INDEX idx_in_app_notifications_user_id ON notifikasi.in_app_notifications(user_id);
CREATE INDEX idx_in_app_notifications_read ON notifikasi.in_app_notifications(user_id, read);
CREATE INDEX idx_in_app_notifications_created_at ON notifikasi.in_app_notifications(created_at DESC);
CREATE INDEX idx_in_app_notifications_priority ON notifikasi.in_app_notifications(priority);

-- Comments
COMMENT ON TABLE notifikasi.in_app_notifications IS 'In-app notifications for users';
COMMENT ON COLUMN notifikasi.in_app_notifications.notification_type IS 'Type of notification (workflow_state_change, document_ready, sla_breach, izin_expiry)';
COMMENT ON COLUMN notifikasi.in_app_notifications.priority IS 'Priority level (low, normal, high, urgent)';
COMMENT ON COLUMN notifikasi.in_app_notifications.category IS 'Visual category (info, warning, error, success, system)';
COMMENT ON COLUMN notifikasi.in_app_notifications.metadata IS 'Additional notification data in JSON format';

-- User notification preferences table
CREATE TABLE IF NOT EXISTS notifikasi.user_notification_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE,
    in_app_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    email_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    sms_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    push_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    batch_non_urgent BOOLEAN NOT NULL DEFAULT FALSE,
    daily_digest_time TIME DEFAULT '08:00:00',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for user preferences
CREATE INDEX idx_user_notification_preferences_user_id ON notifikasi.user_notification_preferences(user_id);

COMMENT ON TABLE notifikasi.user_notification_preferences IS 'User notification channel preferences';
COMMENT ON COLUMN notifikasi.user_notification_preferences.batch_non_urgent IS 'Whether to batch low-priority notifications into daily digest';
COMMENT ON COLUMN notifikasi.user_notification_preferences.daily_digest_time IS 'Time to send daily digest (WIB timezone)';
