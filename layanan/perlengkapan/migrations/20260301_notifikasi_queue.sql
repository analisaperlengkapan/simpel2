-- Migration: Notification Queue and Delivery Status
-- Description: Add notification queue with retry logic and delivery status tracking
-- Date: 2026-02-11

-- Notification queue table
CREATE TABLE IF NOT EXISTS notifikasi.notification_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id UUID NOT NULL REFERENCES notifikasi.notifications(id) ON DELETE CASCADE,
    channel VARCHAR(50) NOT NULL, -- email, sms, push, in_app
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, processing, completed, failed
    priority VARCHAR(50) NOT NULL DEFAULT 'normal', -- low, normal, high, urgent
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    next_retry_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for notification queue
CREATE INDEX idx_notification_queue_status ON notifikasi.notification_queue(status);
CREATE INDEX idx_notification_queue_next_retry ON notifikasi.notification_queue(next_retry_at) WHERE status = 'pending';
CREATE INDEX idx_notification_queue_notification_id ON notifikasi.notification_queue(notification_id);
CREATE INDEX idx_notification_queue_created_at ON notifikasi.notification_queue(created_at);

-- Delivery status table
CREATE TABLE IF NOT EXISTS notifikasi.delivery_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id UUID NOT NULL REFERENCES notifikasi.notifications(id) ON DELETE CASCADE,
    channel VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL, -- delivered, failed, bounced, rejected
    delivered_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for delivery status
CREATE INDEX idx_delivery_status_notification_id ON notifikasi.delivery_status(notification_id);
CREATE INDEX idx_delivery_status_status ON notifikasi.delivery_status(status);
CREATE INDEX idx_delivery_status_created_at ON notifikasi.delivery_status(created_at);

-- Email templates table (for template-based emails)
CREATE TABLE IF NOT EXISTS notifikasi.email_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    subject VARCHAR(500) NOT NULL,
    body_template TEXT NOT NULL,
    variables JSONB,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- SMS templates table (for template-based SMS)
CREATE TABLE IF NOT EXISTS notifikasi.sms_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    body_template TEXT NOT NULL,
    variables JSONB,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default workflow notification templates
INSERT INTO notifikasi.email_templates (name, subject, body_template, variables, enabled) VALUES
('workflow_submitted', 'Pengajuan Baru: {{entity_name}}',
'Yth. {{user_name}},

Terdapat pengajuan baru yang memerlukan persetujuan Anda:

Jenis: {{entity_type}}
Nama: {{entity_name}}
Pengaju: {{requester_name}}
Tanggal: {{submission_date}}

Silakan login ke sistem untuk meninjau dan menyetujui pengajuan ini:
{{action_url}}

Batas waktu: {{deadline}}

Terima kasih,
SIMPEL - Sistem Informasi Manajemen Perlengkapan
Kejaksaan Republik Indonesia',
'{"user_name": "string", "entity_type": "string", "entity_name": "string", "requester_name": "string", "submission_date": "string", "action_url": "string", "deadline": "string"}',
true),

('workflow_approved', 'Pengajuan Disetujui: {{entity_name}}',
'Yth. {{user_name}},

Pengajuan Anda telah disetujui:

Jenis: {{entity_type}}
Nama: {{entity_name}}
Disetujui oleh: {{approver_name}}
Tanggal: {{approval_date}}

{{#if document_url}}
Dokumen resmi dapat diunduh di:
{{document_url}}
{{/if}}

Terima kasih,
SIMPEL - Sistem Informasi Manajemen Perlengkapan
Kejaksaan Republik Indonesia',
'{"user_name": "string", "entity_type": "string", "entity_name": "string", "approver_name": "string", "approval_date": "string", "document_url": "string"}',
true),

('workflow_rejected', 'Pengajuan Ditolak: {{entity_name}}',
'Yth. {{user_name}},

Pengajuan Anda telah ditolak:

Jenis: {{entity_type}}
Nama: {{entity_name}}
Ditolak oleh: {{rejector_name}}
Tanggal: {{rejection_date}}

Alasan penolakan:
{{rejection_reason}}

Silakan perbaiki dan ajukan kembali jika diperlukan.

Terima kasih,
SIMPEL - Sistem Informasi Manajemen Perlengkapan
Kejaksaan Republik Indonesia',
'{"user_name": "string", "entity_type": "string", "entity_name": "string", "rejector_name": "string", "rejection_date": "string", "rejection_reason": "string"}',
true),

('sla_breach', 'Peringatan: SLA Terlampaui - {{entity_name}}',
'Yth. {{user_name}},

PERINGATAN: Pengajuan berikut telah melampaui batas waktu SLA:

Jenis: {{entity_type}}
Nama: {{entity_name}}
Status: {{current_state}}
Batas waktu: {{sla_deadline}}
Terlambat: {{days_overdue}} hari

Mohon segera ditindaklanjuti:
{{action_url}}

Terima kasih,
SIMPEL - Sistem Informasi Manajemen Perlengkapan
Kejaksaan Republik Indonesia',
'{"user_name": "string", "entity_type": "string", "entity_name": "string", "current_state": "string", "sla_deadline": "string", "days_overdue": "number", "action_url": "string"}',
true),

('permit_expiry_reminder', 'Pengingat: Izin Pemakaian BMN Akan Berakhir',
'Yth. {{user_name}},

Izin pemakaian BMN Anda akan berakhir dalam {{days_remaining}} hari:

Nomor Izin: {{permit_number}}
BMN: {{bmn_name}}
Tanggal Berakhir: {{expiry_date}}

Silakan perpanjang izin jika masih diperlukan:
{{action_url}}

Untuk pertanyaan, hubungi bagian perlengkapan satker Anda.

Terima kasih,
SIMPEL - Sistem Informasi Manajemen Perlengkapan
Kejaksaan Republik Indonesia',
'{"user_name": "string", "permit_number": "string", "bmn_name": "string", "expiry_date": "string", "days_remaining": "number", "action_url": "string"}',
true);

-- Insert default SMS templates
INSERT INTO notifikasi.sms_templates (name, body_template, variables, enabled) VALUES
('workflow_submitted_sms', 'SIMPEL: Pengajuan baru {{entity_name}} memerlukan persetujuan Anda. Login ke sistem untuk meninjau.',
'{"entity_name": "string"}',
true),

('workflow_approved_sms', 'SIMPEL: Pengajuan {{entity_name}} telah disetujui. Dokumen dapat diunduh di sistem.',
'{"entity_name": "string"}',
true),

('workflow_rejected_sms', 'SIMPEL: Pengajuan {{entity_name}} ditolak. Alasan: {{rejection_reason}}',
'{"entity_name": "string", "rejection_reason": "string"}',
true),

('permit_expiry_reminder_sms', 'SIMPEL: Izin pemakaian BMN {{bmn_name}} ({{permit_number}}) akan berakhir dalam {{days_remaining}} hari. Perpanjang segera.',
'{"bmn_name": "string", "permit_number": "string", "days_remaining": "number"}',
true);

-- Comments
COMMENT ON TABLE notifikasi.notification_queue IS 'Queue for processing notifications with retry logic';
COMMENT ON TABLE notifikasi.delivery_status IS 'Delivery status tracking for notifications';
COMMENT ON TABLE notifikasi.email_templates IS 'Email templates for notifications';
COMMENT ON TABLE notifikasi.sms_templates IS 'SMS templates for notifications';

COMMENT ON COLUMN notifikasi.notification_queue.retry_count IS 'Number of retry attempts';
COMMENT ON COLUMN notifikasi.notification_queue.next_retry_at IS 'Next retry timestamp (exponential backoff)';
COMMENT ON COLUMN notifikasi.delivery_status.status IS 'Delivery status: delivered, failed, bounced, rejected';
