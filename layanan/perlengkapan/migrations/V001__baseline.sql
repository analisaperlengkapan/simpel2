-- ============================================================================
-- V001__baseline.sql — SQUASHED perlengkapan baseline (F5-B, 2026-06-09)
-- Replaces the legacy V001..V038 accreted migrations (did NOT fresh-apply).
-- Owns schemas: perlengkapan, dokumen, notifikasi, cache.
-- PREREQ (bring-up order): integrasi-migrate + authenc-migrate run FIRST
--   (cross-schema reads integrasi.siman_aset; FK -> authenc.users).
-- Generated via pg_dump --schema-only of a validated fresh-apply (Docker pg15).
-- ============================================================================

CREATE EXTENSION IF NOT EXISTS pg_trgm SCHEMA public;
CREATE EXTENSION IF NOT EXISTS btree_gin SCHEMA public;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp" SCHEMA public;

-- Shared trigger helper (perlengkapan triggers reference public.update_updated_at_column;
-- defined here so the baseline is self-contained). CREATE OR REPLACE = safe if another
-- service also defines it in public.
CREATE OR REPLACE FUNCTION public.update_updated_at_column() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

--
-- PostgreSQL database dump
--


-- Dumped from database version 15.18
-- Dumped by pg_dump version 15.18

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: dokumen; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA IF NOT EXISTS dokumen;


--
-- Name: notifikasi; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA IF NOT EXISTS notifikasi;


--
-- Name: perlengkapan; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA IF NOT EXISTS perlengkapan;


--
-- Name: update_entity_updated_at(); Type: FUNCTION; Schema: perlengkapan; Owner: -
--

CREATE FUNCTION perlengkapan.update_entity_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;


--
-- Name: update_kebutuhan_bmn_updated_at(); Type: FUNCTION; Schema: perlengkapan; Owner: -
--

CREATE FUNCTION perlengkapan.update_kebutuhan_bmn_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: archive_collections; Type: TABLE; Schema: dokumen; Owner: -
--

CREATE TABLE dokumen.archive_collections (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    owner_id uuid
);


--
-- Name: TABLE archive_collections; Type: COMMENT; Schema: dokumen; Owner: -
--

COMMENT ON TABLE dokumen.archive_collections IS 'Archive collections for organizing archived documents';


--
-- Name: archive_documents; Type: TABLE; Schema: dokumen; Owner: -
--

CREATE TABLE dokumen.archive_documents (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    collection_id uuid NOT NULL,
    document_id uuid NOT NULL,
    added_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE archive_documents; Type: COMMENT; Schema: dokumen; Owner: -
--

COMMENT ON TABLE dokumen.archive_documents IS 'Documents in archive collections';


--
-- Name: audit_log; Type: TABLE; Schema: dokumen; Owner: -
--

CREATE TABLE dokumen.audit_log (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    document_id uuid,
    user_id uuid,
    action character varying(100) NOT NULL,
    details jsonb,
    ip_address character varying(45),
    user_agent text,
    "timestamp" timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE audit_log; Type: COMMENT; Schema: dokumen; Owner: -
--

COMMENT ON TABLE dokumen.audit_log IS 'Audit trail for all document operations';


--
-- Name: document_permissions; Type: TABLE; Schema: dokumen; Owner: -
--

CREATE TABLE dokumen.document_permissions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    document_id uuid NOT NULL,
    user_id uuid NOT NULL,
    role character varying(50) NOT NULL,
    granted_by uuid,
    granted_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: document_tags; Type: TABLE; Schema: dokumen; Owner: -
--

CREATE TABLE dokumen.document_tags (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    document_id uuid NOT NULL,
    tag character varying(100) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: document_templates; Type: TABLE; Schema: dokumen; Owner: -
--

CREATE TABLE dokumen.document_templates (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    template_type character varying(100) NOT NULL,
    content text NOT NULL,
    format character varying(50) DEFAULT 'html'::character varying NOT NULL,
    output_format character varying(50) DEFAULT 'pdf'::character varying NOT NULL,
    version integer DEFAULT 1 NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    parent_template_id uuid,
    variables jsonb DEFAULT '{}'::jsonb NOT NULL,
    sample_data jsonb,
    letterhead_config jsonb,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_by uuid,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE document_templates; Type: COMMENT; Schema: dokumen; Owner: -
--

COMMENT ON TABLE dokumen.document_templates IS 'Document templates for automated generation - includes workflow templates';


--
-- Name: COLUMN document_templates.content; Type: COMMENT; Schema: dokumen; Owner: -
--

COMMENT ON COLUMN dokumen.document_templates.content IS 'Handlebars template content';


--
-- Name: COLUMN document_templates.variables; Type: COMMENT; Schema: dokumen; Owner: -
--

COMMENT ON COLUMN dokumen.document_templates.variables IS 'Expected template variables with types and descriptions';


--
-- Name: COLUMN document_templates.letterhead_config; Type: COMMENT; Schema: dokumen; Owner: -
--

COMMENT ON COLUMN dokumen.document_templates.letterhead_config IS 'Configuration for official letterhead (logo, header, footer)';


--
-- Name: document_versions; Type: TABLE; Schema: dokumen; Owner: -
--

CREATE TABLE dokumen.document_versions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    document_id uuid NOT NULL,
    version integer NOT NULL,
    storage_path character varying(500) NOT NULL,
    size bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    checksum character varying(64),
    encrypted boolean DEFAULT false NOT NULL,
    metadata jsonb
);


--
-- Name: TABLE document_versions; Type: COMMENT; Schema: dokumen; Owner: -
--

COMMENT ON TABLE dokumen.document_versions IS 'Document version history';


--
-- Name: documents; Type: TABLE; Schema: dokumen; Owner: -
--

CREATE TABLE dokumen.documents (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    filename character varying(255) NOT NULL,
    content_type character varying(100) NOT NULL,
    size bigint NOT NULL,
    storage_path character varying(500) NOT NULL,
    owner_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    is_archived boolean DEFAULT false NOT NULL,
    checksum character varying(64),
    encrypted boolean DEFAULT false NOT NULL,
    current_version integer DEFAULT 1 NOT NULL,
    metadata jsonb
);


--
-- Name: TABLE documents; Type: COMMENT; Schema: dokumen; Owner: -
--

COMMENT ON TABLE dokumen.documents IS 'Main documents table with archival support';


--
-- Name: COLUMN documents.is_archived; Type: COMMENT; Schema: dokumen; Owner: -
--

COMMENT ON COLUMN dokumen.documents.is_archived IS 'Whether document is archived (moved to archive storage)';


--
-- Name: COLUMN documents.checksum; Type: COMMENT; Schema: dokumen; Owner: -
--

COMMENT ON COLUMN dokumen.documents.checksum IS 'SHA-256 checksum for integrity verification';


--
-- Name: COLUMN documents.metadata; Type: COMMENT; Schema: dokumen; Owner: -
--

COMMENT ON COLUMN dokumen.documents.metadata IS 'Document metadata including document_type, satker_id, workflow_state, archive_path, retention_years';


--
-- Name: generated_documents; Type: TABLE; Schema: dokumen; Owner: -
--

CREATE TABLE dokumen.generated_documents (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    template_id uuid NOT NULL,
    document_number character varying(255) NOT NULL,
    filename character varying(255) NOT NULL,
    storage_path character varying(500) NOT NULL,
    format character varying(50) NOT NULL,
    size bigint NOT NULL,
    checksum character varying(64) NOT NULL,
    generated_data jsonb NOT NULL,
    generated_by uuid NOT NULL,
    generated_at timestamp with time zone DEFAULT now() NOT NULL,
    status character varying(50) DEFAULT 'generated'::character varying NOT NULL,
    metadata jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE generated_documents; Type: COMMENT; Schema: dokumen; Owner: -
--

COMMENT ON TABLE dokumen.generated_documents IS 'Generated documents from templates';


--
-- Name: COLUMN generated_documents.checksum; Type: COMMENT; Schema: dokumen; Owner: -
--

COMMENT ON COLUMN dokumen.generated_documents.checksum IS 'SHA-256 checksum for integrity verification';


--
-- Name: ocr_results; Type: TABLE; Schema: dokumen; Owner: -
--

CREATE TABLE dokumen.ocr_results (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    document_id uuid NOT NULL,
    status character varying(50) DEFAULT 'pending'::character varying NOT NULL,
    text text,
    accuracy real,
    processed_at timestamp with time zone,
    error_message text
);


--
-- Name: template_versions; Type: TABLE; Schema: dokumen; Owner: -
--

CREATE TABLE dokumen.template_versions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    template_id uuid NOT NULL,
    version integer NOT NULL,
    content text NOT NULL,
    variables jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    change_notes text
);


--
-- Name: TABLE template_versions; Type: COMMENT; Schema: dokumen; Owner: -
--

COMMENT ON TABLE dokumen.template_versions IS 'Version history of document templates';


--
-- Name: delivery_status; Type: TABLE; Schema: notifikasi; Owner: -
--

CREATE TABLE notifikasi.delivery_status (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    notification_id uuid NOT NULL,
    channel character varying(50) NOT NULL,
    status character varying(50) NOT NULL,
    delivered_at timestamp with time zone,
    error_message text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE delivery_status; Type: COMMENT; Schema: notifikasi; Owner: -
--

COMMENT ON TABLE notifikasi.delivery_status IS 'Delivery status tracking for notifications';


--
-- Name: COLUMN delivery_status.status; Type: COMMENT; Schema: notifikasi; Owner: -
--

COMMENT ON COLUMN notifikasi.delivery_status.status IS 'Delivery status: delivered, failed, bounced, rejected';


--
-- Name: email_templates; Type: TABLE; Schema: notifikasi; Owner: -
--

CREATE TABLE notifikasi.email_templates (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(255) NOT NULL,
    subject character varying(500) NOT NULL,
    body_template text NOT NULL,
    variables jsonb,
    enabled boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE email_templates; Type: COMMENT; Schema: notifikasi; Owner: -
--

COMMENT ON TABLE notifikasi.email_templates IS 'Email templates for notifications';


--
-- Name: in_app_notifications; Type: TABLE; Schema: notifikasi; Owner: -
--

CREATE TABLE notifikasi.in_app_notifications (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    notification_type character varying(50) NOT NULL,
    title character varying(255) NOT NULL,
    message text NOT NULL,
    priority character varying(20) DEFAULT 'normal'::character varying NOT NULL,
    category character varying(50) DEFAULT 'info'::character varying,
    action_url text,
    metadata jsonb,
    read boolean DEFAULT false NOT NULL,
    read_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone
);


--
-- Name: TABLE in_app_notifications; Type: COMMENT; Schema: notifikasi; Owner: -
--

COMMENT ON TABLE notifikasi.in_app_notifications IS 'In-app notifications for users';


--
-- Name: COLUMN in_app_notifications.notification_type; Type: COMMENT; Schema: notifikasi; Owner: -
--

COMMENT ON COLUMN notifikasi.in_app_notifications.notification_type IS 'Type of notification (workflow_state_change, document_ready, sla_breach, izin_expiry)';


--
-- Name: COLUMN in_app_notifications.priority; Type: COMMENT; Schema: notifikasi; Owner: -
--

COMMENT ON COLUMN notifikasi.in_app_notifications.priority IS 'Priority level (low, normal, high, urgent)';


--
-- Name: COLUMN in_app_notifications.category; Type: COMMENT; Schema: notifikasi; Owner: -
--

COMMENT ON COLUMN notifikasi.in_app_notifications.category IS 'Visual category (info, warning, error, success, system)';


--
-- Name: COLUMN in_app_notifications.metadata; Type: COMMENT; Schema: notifikasi; Owner: -
--

COMMENT ON COLUMN notifikasi.in_app_notifications.metadata IS 'Additional notification data in JSON format';


--
-- Name: notification_queue; Type: TABLE; Schema: notifikasi; Owner: -
--

CREATE TABLE notifikasi.notification_queue (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    notification_id uuid NOT NULL,
    channel character varying(50) NOT NULL,
    status character varying(50) DEFAULT 'pending'::character varying NOT NULL,
    priority character varying(50) DEFAULT 'normal'::character varying NOT NULL,
    retry_count integer DEFAULT 0 NOT NULL,
    max_retries integer DEFAULT 3 NOT NULL,
    next_retry_at timestamp with time zone,
    error_message text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE notification_queue; Type: COMMENT; Schema: notifikasi; Owner: -
--

COMMENT ON TABLE notifikasi.notification_queue IS 'Queue for processing notifications with retry logic';


--
-- Name: COLUMN notification_queue.retry_count; Type: COMMENT; Schema: notifikasi; Owner: -
--

COMMENT ON COLUMN notifikasi.notification_queue.retry_count IS 'Number of retry attempts';


--
-- Name: COLUMN notification_queue.next_retry_at; Type: COMMENT; Schema: notifikasi; Owner: -
--

COMMENT ON COLUMN notifikasi.notification_queue.next_retry_at IS 'Next retry timestamp (exponential backoff)';


--
-- Name: sms_templates; Type: TABLE; Schema: notifikasi; Owner: -
--

CREATE TABLE notifikasi.sms_templates (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(255) NOT NULL,
    body_template text NOT NULL,
    variables jsonb,
    enabled boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE sms_templates; Type: COMMENT; Schema: notifikasi; Owner: -
--

COMMENT ON TABLE notifikasi.sms_templates IS 'SMS templates for notifications';


--
-- Name: user_notification_preferences; Type: TABLE; Schema: notifikasi; Owner: -
--

CREATE TABLE notifikasi.user_notification_preferences (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    in_app_enabled boolean DEFAULT true NOT NULL,
    email_enabled boolean DEFAULT true NOT NULL,
    sms_enabled boolean DEFAULT false NOT NULL,
    push_enabled boolean DEFAULT true NOT NULL,
    batch_non_urgent boolean DEFAULT false NOT NULL,
    daily_digest_time time without time zone DEFAULT '08:00:00'::time without time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE user_notification_preferences; Type: COMMENT; Schema: notifikasi; Owner: -
--

COMMENT ON TABLE notifikasi.user_notification_preferences IS 'User notification channel preferences';


--
-- Name: COLUMN user_notification_preferences.batch_non_urgent; Type: COMMENT; Schema: notifikasi; Owner: -
--

COMMENT ON COLUMN notifikasi.user_notification_preferences.batch_non_urgent IS 'Whether to batch low-priority notifications into daily digest';


--
-- Name: COLUMN user_notification_preferences.daily_digest_time; Type: COMMENT; Schema: notifikasi; Owner: -
--

COMMENT ON COLUMN notifikasi.user_notification_preferences.daily_digest_time IS 'Time to send daily digest (WIB timezone)';


--
-- Name: analisis_kebutuhan; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.analisis_kebutuhan (
    id uuid NOT NULL,
    judul character varying NOT NULL,
    kategori character varying NOT NULL,
    deskripsi text,
    prioritas character varying DEFAULT 'sedang'::character varying NOT NULL,
    status character varying DEFAULT 'draft'::character varying NOT NULL,
    estimasi_biaya double precision,
    justifikasi text,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    created_by uuid,
    updated_by uuid
);


--
-- Name: audit_log; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.audit_log (
    id uuid NOT NULL,
    occurred_at timestamp with time zone DEFAULT now() NOT NULL,
    actor_user_id uuid,
    actor_username character varying(255),
    actor_ip character varying(64),
    action character varying(32) NOT NULL,
    action_name character varying(128),
    resource_type character varying(64) NOT NULL,
    resource_id character varying(255),
    module character varying(64) NOT NULL,
    success boolean DEFAULT true NOT NULL,
    message text,
    metadata jsonb,
    retention_until timestamp with time zone DEFAULT (now() + '10 years'::interval) NOT NULL
);


--
-- Name: TABLE audit_log; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.audit_log IS 'Cross-module audit trail produced via lib_perlengkapan::contracts::AuditSink';


--
-- Name: COLUMN audit_log.action; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.audit_log.action IS 'Stable action verb: create | read | update | delete | login | logout | approve | reject | submit | cancel | export | import | custom';


--
-- Name: COLUMN audit_log.action_name; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.audit_log.action_name IS 'Free-form action name when action = custom (e.g. workflow.delegate)';


--
-- Name: COLUMN audit_log.module; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.audit_log.module IS 'Source module: workflow | dokumen | notifikasi | bantuan | pemakaian | penghapusan | …';


--
-- Name: COLUMN audit_log.retention_until; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.audit_log.retention_until IS 'Statutory retention horizon. BMN modules: occurred_at + 10y (PMK); operational modules (bantuan/notifikasi): occurred_at + 3y. Rows must not be purged before this instant.';


--
-- Name: batch_operation_log; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.batch_operation_log (
    id bigint NOT NULL,
    batch_id uuid NOT NULL,
    operation_type character varying(100) NOT NULL,
    total_items integer NOT NULL,
    successful_items integer NOT NULL,
    failed_items integer NOT NULL,
    user_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE batch_operation_log; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.batch_operation_log IS 'Audit log for batch operations on kebutuhan BMN';


--
-- Name: COLUMN batch_operation_log.batch_id; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.batch_operation_log.batch_id IS 'Unique identifier for the batch operation';


--
-- Name: COLUMN batch_operation_log.operation_type; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.batch_operation_log.operation_type IS 'Type of batch operation (batch_approve, batch_reject, batch_update_status)';


--
-- Name: COLUMN batch_operation_log.total_items; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.batch_operation_log.total_items IS 'Total number of items in the batch';


--
-- Name: COLUMN batch_operation_log.successful_items; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.batch_operation_log.successful_items IS 'Number of items successfully processed';


--
-- Name: COLUMN batch_operation_log.failed_items; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.batch_operation_log.failed_items IS 'Number of items that failed processing';


--
-- Name: COLUMN batch_operation_log.user_id; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.batch_operation_log.user_id IS 'User who initiated the batch operation';


--
-- Name: COLUMN batch_operation_log.created_at; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.batch_operation_log.created_at IS 'Timestamp when the batch operation was executed';


--
-- Name: batch_operation_log_id_seq; Type: SEQUENCE; Schema: perlengkapan; Owner: -
--

CREATE SEQUENCE perlengkapan.batch_operation_log_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: batch_operation_log_id_seq; Type: SEQUENCE OWNED BY; Schema: perlengkapan; Owner: -
--

ALTER SEQUENCE perlengkapan.batch_operation_log_id_seq OWNED BY perlengkapan.batch_operation_log.id;


--
-- Name: export_jobs; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.export_jobs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    entity_type character varying(50) NOT NULL,
    filters jsonb,
    status character varying(20) DEFAULT 'queued'::character varying NOT NULL,
    progress real,
    document_id uuid,
    error_message text,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    started_at timestamp with time zone,
    completed_at timestamp with time zone,
    CONSTRAINT chk_progress CHECK (((progress IS NULL) OR ((progress >= (0)::double precision) AND (progress <= (100)::double precision)))),
    CONSTRAINT chk_status CHECK (((status)::text = ANY ((ARRAY['queued'::character varying, 'processing'::character varying, 'completed'::character varying, 'failed'::character varying])::text[])))
);


--
-- Name: TABLE export_jobs; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.export_jobs IS 'Async export job queue for large datasets';


--
-- Name: COLUMN export_jobs.entity_type; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.export_jobs.entity_type IS 'Type of entity being exported (kebutuhan_bmn, pakaian_dinas, etc.)';


--
-- Name: COLUMN export_jobs.filters; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.export_jobs.filters IS 'JSON-encoded filters applied to the export';


--
-- Name: COLUMN export_jobs.status; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.export_jobs.status IS 'Job status: queued, processing, completed, failed';


--
-- Name: COLUMN export_jobs.progress; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.export_jobs.progress IS 'Export progress percentage (0-100)';


--
-- Name: COLUMN export_jobs.document_id; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.export_jobs.document_id IS 'Reference to generated document in dokumen service';


--
-- Name: COLUMN export_jobs.error_message; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.export_jobs.error_message IS 'Error message if job failed';


--
-- Name: izin_pemakaian_bmn; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.izin_pemakaian_bmn (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    nomor_izin character varying(100),
    jenis_bmn character varying(50) NOT NULL,
    bmn_nup character varying(50) NOT NULL,
    bmn_kode_barang character varying(50) NOT NULL,
    bmn_nama_barang character varying(255) NOT NULL,
    bmn_merk character varying(100),
    bmn_tipe character varying(100),
    bmn_tahun_perolehan integer,
    no_polisi character varying(20),
    no_bpkb character varying(50),
    no_rangka character varying(50),
    no_mesin character varying(50),
    alamat text,
    lokasi_pemakaian text,
    no_stnk character varying(50),
    luas_tanah double precision,
    luas_bangunan double precision,
    serial_number character varying(100),
    spesifikasi jsonb,
    pegawai_nip character varying(30) NOT NULL,
    pegawai_nama character varying(255) NOT NULL,
    pegawai_jabatan character varying(255),
    pegawai_satker_id uuid NOT NULL,
    pegawai_satker_nama character varying(255),
    tanggal_mulai date NOT NULL,
    tanggal_selesai date NOT NULL,
    status character varying(50) DEFAULT 'DRAFT'::character varying NOT NULL,
    status_kode integer,
    approved_by uuid,
    approved_by_nama text,
    approved_at timestamp with time zone,
    rejection_reason text,
    catatan_approval text,
    catatan_revocation text,
    revoked_by uuid,
    revoked_by_nama text,
    revoked_at timestamp with time zone,
    revocation_reason text,
    updated_by_nama text,
    is_renewal boolean DEFAULT false,
    previous_permit_id uuid,
    file_pendukung jsonb DEFAULT '[]'::jsonb,
    keperluan text,
    keterangan text,
    created_by uuid,
    created_by_nama text,
    updated_by uuid,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    document_id uuid,
    document_url text,
    foto_pegawai text,
    konsep_surat_url text,
    konsep_surat_generated_at timestamp with time zone,
    signed_pdf_url text,
    signed_pdf_uploaded_at timestamp with time zone,
    is_completed boolean DEFAULT false,
    pegawai_golongan character varying(50),
    pegawai_pangkat character varying(100),
    pegawai_unit_kerja character varying(255),
    konsep_surat_pdf_url text,
    konsep_surat_pdf_generated_at timestamp with time zone,
    konsep_surat_docx_path text,
    konsep_surat_pdf_path text,
    validator_satker_id uuid,
    validator_satker_nama text,
    tanggal_validasi_satker timestamp with time zone,
    catatan_validator_satker text,
    approver_satker_id uuid,
    approver_satker_nama text,
    tanggal_approval_satker timestamp with time zone,
    catatan_approver_satker text,
    approved_via_legacy_flow boolean DEFAULT false NOT NULL,
    version integer DEFAULT 1 NOT NULL,
    CONSTRAINT chk_housing_fields CHECK ((((jenis_bmn)::text <> 'RUMAH_DINAS'::text) OR (alamat IS NOT NULL))),
    CONSTRAINT chk_tanggal_valid CHECK ((tanggal_selesai >= tanggal_mulai)),
    CONSTRAINT chk_vehicle_fields CHECK ((((jenis_bmn)::text <> 'KENDARAAN_BERMOTOR'::text) OR ((no_polisi IS NOT NULL) AND (no_bpkb IS NOT NULL)))),
    CONSTRAINT izin_pemakaian_bmn_jenis_bmn_check CHECK (((jenis_bmn)::text = ANY ((ARRAY['KENDARAAN_BERMOTOR'::character varying, 'RUMAH_NEGARA'::character varying, 'LAPTOP'::character varying, 'LAINNYA'::character varying])::text[]))),
    CONSTRAINT izin_pemakaian_bmn_status_check CHECK (((status)::text = ANY ((ARRAY['DRAFT'::character varying, 'SUBMITTED'::character varying, 'SUBMITTED_APPROVER_SATKER'::character varying, 'REVISI_OPERATOR'::character varying, 'APPROVED'::character varying, 'REJECTED'::character varying, 'ACTIVE'::character varying, 'EXPIRED'::character varying, 'REVOKED'::character varying, 'CANCELLED'::character varying])::text[])))
);


--
-- Name: TABLE izin_pemakaian_bmn; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.izin_pemakaian_bmn IS 'BMN usage permits for vehicles, housing, laptops, etc.';


--
-- Name: COLUMN izin_pemakaian_bmn.nomor_izin; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.nomor_izin IS 'Auto-generated permit number';


--
-- Name: COLUMN izin_pemakaian_bmn.bmn_nup; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.bmn_nup IS 'NUP (Nomor Urut Pendaftaran) from SIMAN';


--
-- Name: COLUMN izin_pemakaian_bmn.is_renewal; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.is_renewal IS 'TRUE if this is a renewal of previous permit';


--
-- Name: COLUMN izin_pemakaian_bmn.document_id; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.document_id IS 'Reference to generated permit document in dokumen service';


--
-- Name: COLUMN izin_pemakaian_bmn.document_url; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.document_url IS 'Download URL for the permit document';


--
-- Name: COLUMN izin_pemakaian_bmn.konsep_surat_url; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.konsep_surat_url IS 'Public download URL for the DOCX (editable) konsep surat';


--
-- Name: COLUMN izin_pemakaian_bmn.konsep_surat_pdf_url; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.konsep_surat_pdf_url IS 'Public download URL for the PDF (final) konsep surat';


--
-- Name: COLUMN izin_pemakaian_bmn.konsep_surat_docx_path; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.konsep_surat_docx_path IS 'Server-side filesystem path the DOCX route handler streams from';


--
-- Name: COLUMN izin_pemakaian_bmn.konsep_surat_pdf_path; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.konsep_surat_pdf_path IS 'Server-side filesystem path the PDF route handler streams from';


--
-- Name: COLUMN izin_pemakaian_bmn.validator_satker_id; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.validator_satker_id IS 'V035 (Fase 1.5): Validator internal satker yg meneruskan/menolak. NULL utk record legacy.';


--
-- Name: COLUMN izin_pemakaian_bmn.approver_satker_id; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.approver_satker_id IS 'V035 (Fase 1.5): Approver internal satker (Pengguna Barang Satker). NULL utk record legacy.';


--
-- Name: COLUMN izin_pemakaian_bmn.approved_via_legacy_flow; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.approved_via_legacy_flow IS 'V035 (Fase 1.5): TRUE jika record di-approve via alur lama Operator → Pimpinan langsung.';


--
-- Name: COLUMN izin_pemakaian_bmn.version; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.version IS 'V035 (Fase 1.5): Optimistic lock counter — increment tiap UPDATE workflow.';


--
-- Name: izin_pemakaian_bmn_aktivitas; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.izin_pemakaian_bmn_aktivitas (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    izin_pemakaian_id uuid NOT NULL,
    from_status character varying(50),
    to_status character varying(50) NOT NULL,
    from_status_kode integer,
    to_status_kode integer,
    user_id uuid,
    nip character varying(30),
    nama character varying(255),
    pangkat character varying(100),
    jabatan character varying(255),
    role character varying(100),
    aksi character varying(50) NOT NULL,
    komentar text,
    created_at timestamp with time zone DEFAULT now()
);


--
-- Name: TABLE izin_pemakaian_bmn_aktivitas; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.izin_pemakaian_bmn_aktivitas IS 'Workflow history audit trail for BMN usage permits';


--
-- Name: mapping_kodefikasi; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.mapping_kodefikasi (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    satker_id uuid NOT NULL,
    kode_barang_lama character varying(50) NOT NULL,
    nama_barang_lama character varying(255) NOT NULL,
    kode_barang_baru character varying(50),
    nama_barang_baru character varying(255),
    kode_barang_baru_id uuid,
    status_mapping character varying(50) DEFAULT 'PROPOSED'::character varying NOT NULL,
    catatan_mapping text,
    alasan_penolakan text,
    verified_by uuid,
    verified_at timestamp with time zone,
    approved_by uuid,
    approved_at timestamp with time zone,
    created_by uuid,
    updated_by uuid,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    CONSTRAINT mapping_kodefikasi_status_mapping_check CHECK (((status_mapping)::text = ANY ((ARRAY['PROPOSED'::character varying, 'VERIFIED'::character varying, 'APPROVED'::character varying, 'REJECTED'::character varying])::text[])))
);


--
-- Name: TABLE mapping_kodefikasi; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.mapping_kodefikasi IS 'Mapping of non-standard asset codes to standard SIMAK BMN codes';


--
-- Name: COLUMN mapping_kodefikasi.kode_barang_lama; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.mapping_kodefikasi.kode_barang_lama IS 'Non-standard code from satker';


--
-- Name: COLUMN mapping_kodefikasi.kode_barang_baru; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.mapping_kodefikasi.kode_barang_baru IS 'Standard SIMAK BMN code';


--
-- Name: ms_aktivitas_bmn; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.ms_aktivitas_bmn (
    id integer NOT NULL,
    kode integer NOT NULL,
    nama character varying(100) NOT NULL,
    deskripsi text,
    urutan integer DEFAULT 0,
    is_active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);


--
-- Name: TABLE ms_aktivitas_bmn; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.ms_aktivitas_bmn IS 'Master workflow status codes for BMN requests';


--
-- Name: ms_aktivitas_bmn_id_seq; Type: SEQUENCE; Schema: perlengkapan; Owner: -
--

CREATE SEQUENCE perlengkapan.ms_aktivitas_bmn_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: ms_aktivitas_bmn_id_seq; Type: SEQUENCE OWNED BY; Schema: perlengkapan; Owner: -
--

ALTER SEQUENCE perlengkapan.ms_aktivitas_bmn_id_seq OWNED BY perlengkapan.ms_aktivitas_bmn.id;


--
-- Name: ms_jenis_pakaian_dinas; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.ms_jenis_pakaian_dinas (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    nama character varying(255) NOT NULL,
    deskripsi text,
    is_active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: TABLE ms_jenis_pakaian_dinas; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.ms_jenis_pakaian_dinas IS 'Master table for uniform types (PDH, PDL, Toga, etc.)';


--
-- Name: COLUMN ms_jenis_pakaian_dinas.created_at; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.ms_jenis_pakaian_dinas.created_at IS 'Creation timestamp (TIMESTAMPTZ)';


--
-- Name: COLUMN ms_jenis_pakaian_dinas.updated_at; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.ms_jenis_pakaian_dinas.updated_at IS 'Last update timestamp (TIMESTAMPTZ)';


--
-- Name: ms_spesifikasi_pakaian_dinas; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.ms_spesifikasi_pakaian_dinas (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    jenis_pakaian_dinas_id uuid NOT NULL,
    nama character varying(255) NOT NULL,
    gender character varying(10) NOT NULL,
    ukuran_group character varying(50) NOT NULL,
    deskripsi text,
    is_active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT ms_spesifikasi_pakaian_dinas_gender_check CHECK (((gender)::text = ANY ((ARRAY['L'::character varying, 'P'::character varying, 'SEMUA'::character varying])::text[]))),
    CONSTRAINT ms_spesifikasi_pakaian_dinas_ukuran_group_check CHECK (((ukuran_group)::text = ANY ((ARRAY['BAJU'::character varying, 'CELANA'::character varying, 'SEPATU'::character varying])::text[])))
);


--
-- Name: TABLE ms_spesifikasi_pakaian_dinas; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.ms_spesifikasi_pakaian_dinas IS 'Specifications for each uniform type with size category';


--
-- Name: ms_spesifikasi_pakaian_dinas_foto; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.ms_spesifikasi_pakaian_dinas_foto (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    spesifikasi_id uuid NOT NULL,
    path character varying(500) NOT NULL,
    filename character varying(255) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: ms_subspesifikasi_pakaian_dinas; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.ms_subspesifikasi_pakaian_dinas (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    spesifikasi_id uuid NOT NULL,
    nama character varying(255) NOT NULL,
    gender character varying(10) NOT NULL,
    is_active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT ms_subspesifikasi_pakaian_dinas_gender_check CHECK (((gender)::text = ANY ((ARRAY['L'::character varying, 'P'::character varying, 'SEMUA'::character varying])::text[])))
);


--
-- Name: ms_ukuran; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.ms_ukuran (
    ukuran character varying(20) NOT NULL,
    "group" character varying(50) NOT NULL,
    urutan integer DEFAULT 0,
    CONSTRAINT ms_ukuran_group_check CHECK ((("group")::text = ANY ((ARRAY['BAJU'::character varying, 'CELANA'::character varying, 'SEPATU'::character varying])::text[])))
);


--
-- Name: ms_workflow_status; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.ms_workflow_status (
    kode integer NOT NULL,
    modul character varying(50) NOT NULL,
    nama character varying(100) NOT NULL,
    deskripsi text,
    urutan integer DEFAULT 0,
    is_terminal boolean DEFAULT false,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: parallel_approval_votes; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.parallel_approval_votes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    parallel_approval_id uuid NOT NULL,
    approver_user_id uuid NOT NULL,
    approver_nip character varying(30),
    approver_nama character varying(255),
    approver_jabatan character varying(255),
    approver_role character varying(100),
    vote character varying(20) NOT NULL,
    komentar text,
    voted_at timestamp with time zone DEFAULT now(),
    CONSTRAINT parallel_approval_votes_vote_check CHECK (((vote)::text = ANY ((ARRAY['APPROVE'::character varying, 'REJECT'::character varying, 'ABSTAIN'::character varying])::text[])))
);


--
-- Name: TABLE parallel_approval_votes; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.parallel_approval_votes IS 'Individual votes in parallel approval process';


--
-- Name: parallel_approvals; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.parallel_approvals (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    entity_type character varying(50) NOT NULL,
    entity_id uuid NOT NULL,
    approval_step character varying(100) NOT NULL,
    required_approvers integer NOT NULL,
    approval_threshold integer NOT NULL,
    status character varying(50) DEFAULT 'PENDING'::character varying NOT NULL,
    total_votes integer DEFAULT 0,
    approve_votes integer DEFAULT 0,
    reject_votes integer DEFAULT 0,
    started_at timestamp with time zone DEFAULT now(),
    completed_at timestamp with time zone,
    expires_at timestamp with time zone,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    CONSTRAINT chk_threshold_valid CHECK ((approval_threshold <= required_approvers)),
    CONSTRAINT parallel_approvals_check CHECK (((approval_threshold > 0) AND (approval_threshold <= required_approvers))),
    CONSTRAINT parallel_approvals_entity_type_check CHECK (((entity_type)::text = ANY ((ARRAY['kebutuhan_bmn'::character varying, 'pakaian_dinas'::character varying, 'pemakaian_bmn'::character varying])::text[]))),
    CONSTRAINT parallel_approvals_required_approvers_check CHECK ((required_approvers > 0)),
    CONSTRAINT parallel_approvals_status_check CHECK (((status)::text = ANY ((ARRAY['PENDING'::character varying, 'APPROVED'::character varying, 'REJECTED'::character varying, 'CANCELLED'::character varying])::text[])))
);


--
-- Name: TABLE parallel_approvals; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.parallel_approvals IS 'Parallel approval workflow tracking';


--
-- Name: COLUMN parallel_approvals.approval_threshold; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.parallel_approvals.approval_threshold IS 'Number of approvals needed to pass (e.g., 2 out of 3)';


--
-- Name: pegawai_pakaian_dinas; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.pegawai_pakaian_dinas (
    nip character varying(30) NOT NULL,
    nama character varying(500),
    ukuran_baju character varying(20),
    ukuran_celana character varying(20),
    ukuran_sepatu character varying(20),
    with_hijab boolean DEFAULT false,
    pangkat character varying(255),
    jabatan character varying(500),
    status character varying(50),
    last_pengajuan_satker_pegawai_id uuid,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    eselon character varying(20),
    jenis_kelamin character varying(1),
    jenis_pegawai character varying(32),
    mapped_unit_kerja text,
    kode_satker character varying(32),
    CONSTRAINT pegawai_pakaian_dinas_jenis_kelamin_check CHECK (((jenis_kelamin IS NULL) OR ((jenis_kelamin)::text = ANY ((ARRAY['L'::character varying, 'P'::character varying])::text[]))))
);


--
-- Name: TABLE pegawai_pakaian_dinas; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.pegawai_pakaian_dinas IS 'Persistent record of employee uniform sizes, updated on approval';


--
-- Name: COLUMN pegawai_pakaian_dinas.eselon; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.pegawai_pakaian_dinas.eselon IS 'Eselon I/II/III/IV or "Non-eselon" — filter for laporan rekap, not sourced from MySIMKARI.';


--
-- Name: COLUMN pegawai_pakaian_dinas.jenis_kelamin; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.pegawai_pakaian_dinas.jenis_kelamin IS 'L/P — drives gender variant of pakaian; not reliably present in MySIMKARI responses.';


--
-- Name: COLUMN pegawai_pakaian_dinas.jenis_pegawai; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.pegawai_pakaian_dinas.jenis_pegawai IS 'Klasifikasi pegawai ("TU" / "Jaksa" / …) — perlengkapan-owned filter.';


--
-- Name: COLUMN pegawai_pakaian_dinas.mapped_unit_kerja; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.pegawai_pakaian_dinas.mapped_unit_kerja IS 'Normalised unit kerja string used for grouping in laporan output.';


--
-- Name: pemakaian_bmn_items; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.pemakaian_bmn_items (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    izin_pemakaian_id uuid NOT NULL,
    bmn_nup character varying(100) NOT NULL,
    bmn_kode_barang character varying(100) NOT NULL,
    bmn_nama_barang character varying(500) NOT NULL,
    bmn_merk character varying(255),
    bmn_tahun_perolehan integer,
    bmn_kondisi character varying(50),
    detail_bmn jsonb DEFAULT '{}'::jsonb,
    keterangan text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: pengajuan_bmn_referensi_diizinkan; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.pengajuan_bmn_referensi_diizinkan (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    pengajuan_id uuid NOT NULL,
    kode_barang text NOT NULL,
    nama_barang text NOT NULL,
    keterangan text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE pengajuan_bmn_referensi_diizinkan; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.pengajuan_bmn_referensi_diizinkan IS 'V029 (Fase 1.6): Daftar kode_barang yg boleh diusulkan operator per pengajuan. Validator Pusat tetapkan saat create periode; backend validate di create_barang.';


--
-- Name: pengajuan_kebutuhan_bmn; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.pengajuan_kebutuhan_bmn (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    nama character varying(255) NOT NULL,
    deskripsi text,
    tahun integer NOT NULL,
    tgl_mulai date NOT NULL,
    tgl_selesai date NOT NULL,
    pilihan_satker character varying(20) DEFAULT 'semua'::character varying,
    id_jenis_asset jsonb DEFAULT '[]'::jsonb,
    is_appv_daskrimti boolean DEFAULT false,
    status_kode integer DEFAULT 2000 NOT NULL,
    created_by uuid,
    updated_by uuid,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    version integer DEFAULT 1,
    laporan_url text,
    laporan_format character varying(10),
    laporan_generated_at timestamp with time zone,
    scope_satker text DEFAULT 'semua'::text NOT NULL,
    wilayah_id text,
    CONSTRAINT chk_pengajuan_kebutuhan_bmn_scope_satker CHECK ((scope_satker = ANY (ARRAY['semua'::text, 'sebagian'::text, 'wilayah'::text]))),
    CONSTRAINT chk_pengajuan_kebutuhan_bmn_wilayah_required CHECK (((scope_satker <> 'wilayah'::text) OR (wilayah_id IS NOT NULL))),
    CONSTRAINT pengajuan_kebutuhan_bmn_pilihan_satker_check CHECK (((pilihan_satker)::text = ANY ((ARRAY['semua'::character varying, 'sebagian'::character varying])::text[]))),
    CONSTRAINT valid_date_range CHECK ((tgl_selesai >= tgl_mulai)),
    CONSTRAINT valid_tahun CHECK (((tahun >= 2020) AND (tahun <= 2100)))
);


--
-- Name: TABLE pengajuan_kebutuhan_bmn; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.pengajuan_kebutuhan_bmn IS 'Main entity for BMN needs analysis requests';


--
-- Name: COLUMN pengajuan_kebutuhan_bmn.scope_satker; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.pengajuan_kebutuhan_bmn.scope_satker IS 'Cakupan satker: semua | sebagian | wilayah (V029). Menggantikan pilihan_satker (legacy, dipertahankan utk backward compat).';


--
-- Name: COLUMN pengajuan_kebutuhan_bmn.wilayah_id; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.pengajuan_kebutuhan_bmn.wilayah_id IS 'Nama wilayah Kejaksaan Tinggi (text label, match integrasi.mysimkari_satker.wilayah). WAJIB jika scope_satker = wilayah. Resolver mengisi pengajuan_kebutuhan_bmn_satker_terpilih dgn semua kode_satker di wilayah ini.';


--
-- Name: pengajuan_kebutuhan_bmn_asset; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.pengajuan_kebutuhan_bmn_asset (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    pengajuan_id uuid NOT NULL,
    kode_barang character varying(50),
    nm_barang character varying(255),
    ms_jenis_asset_id integer,
    keterangan text,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);


--
-- Name: TABLE pengajuan_kebutuhan_bmn_asset; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.pengajuan_kebutuhan_bmn_asset IS 'Asset types included in a BMN request';


--
-- Name: pengajuan_kebutuhan_bmn_satker; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.pengajuan_kebutuhan_bmn_satker (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    pengajuan_id uuid NOT NULL,
    satker_id character varying(20) NOT NULL,
    satker_pusat_id character varying(20),
    satker_nama character varying(255),
    status_kode integer DEFAULT 2001 NOT NULL,
    prioritas integer DEFAULT 0,
    created_by uuid,
    updated_by uuid,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    catatan_satker text,
    lampiran_surat_permohonan text,
    lampiran_pendukung jsonb DEFAULT '[]'::jsonb,
    catatan_validator_wilayah text,
    catatan_validator_pusat text,
    tanggal_submit_wilayah timestamp with time zone,
    tanggal_submit_pusat timestamp with time zone,
    validator_wilayah_id uuid,
    validator_pusat_id uuid,
    data_eksisting_siman jsonb DEFAULT '{}'::jsonb,
    data_pegawai_mysimkari jsonb DEFAULT '{}'::jsonb,
    rekap_eselon jsonb DEFAULT '{}'::jsonb,
    rekap_non_eselon jsonb DEFAULT '{}'::jsonb,
    hasil_analisis jsonb DEFAULT '{}'::jsonb,
    is_approved boolean,
    alasan_keputusan text,
    data_eksisting_siman_per_barang jsonb DEFAULT '{}'::jsonb,
    analisis_snapshot_at_submit jsonb
);


--
-- Name: TABLE pengajuan_kebutuhan_bmn_satker; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.pengajuan_kebutuhan_bmn_satker IS 'Per-satker tracking for BMN requests';


--
-- Name: COLUMN pengajuan_kebutuhan_bmn_satker.data_eksisting_siman_per_barang; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.pengajuan_kebutuhan_bmn_satker.data_eksisting_siman_per_barang IS 'V029 (Fase 1.6): Pre-fetch SIMAN existing assets per kode_barang dlm allowed-list. Diisi async saat satker join; ditampilkan side-by-side dgn usulan operator (transparansi dari hulu).';


--
-- Name: COLUMN pengajuan_kebutuhan_bmn_satker.analisis_snapshot_at_submit; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.pengajuan_kebutuhan_bmn_satker.analisis_snapshot_at_submit IS 'V029 (Fase 1.6): Snapshot lengkap data analisis (usulan + eksisting + kondisi) di-freeze saat operator submit ke wilayah. Validator Wilayah & Pusat melihat data konsisten.';


--
-- Name: pengajuan_kebutuhan_bmn_satker_aktivitas; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    pengajuan_satker_id uuid NOT NULL,
    from_status_kode integer,
    to_status_kode integer NOT NULL,
    user_id uuid,
    nip character varying(30),
    nama character varying(255),
    pangkat character varying(100),
    jabatan character varying(255),
    role character varying(100),
    aksi character varying(50) NOT NULL,
    komentar text,
    created_at timestamp with time zone DEFAULT now(),
    document_id uuid,
    document_url text
);


--
-- Name: TABLE pengajuan_kebutuhan_bmn_satker_aktivitas; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas IS 'Workflow history audit trail';


--
-- Name: COLUMN pengajuan_kebutuhan_bmn_satker_aktivitas.document_id; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas.document_id IS 'ID of generated document (SK, surat izin, etc.) from dokumen service';


--
-- Name: COLUMN pengajuan_kebutuhan_bmn_satker_aktivitas.document_url; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas.document_url IS 'Download URL for the generated document';


--
-- Name: pengajuan_kebutuhan_bmn_satker_barang; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.pengajuan_kebutuhan_bmn_satker_barang (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    pengajuan_satker_id uuid NOT NULL,
    nama character varying(255) NOT NULL,
    kode_barang character varying(50),
    jumlah integer DEFAULT 1 NOT NULL,
    satuan character varying(50) DEFAULT 'Unit'::character varying,
    jml_setuju integer DEFAULT 0,
    alasan text,
    keterangan text,
    prioritas integer DEFAULT 0,
    skor double precision DEFAULT 0,
    file_pendukung jsonb DEFAULT '[]'::jsonb,
    existing_count integer DEFAULT 0,
    existing_condition text,
    created_by uuid,
    updated_by uuid,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    CONSTRAINT pengajuan_kebutuhan_bmn_satker_barang_jml_setuju_check CHECK ((jml_setuju >= 0)),
    CONSTRAINT pengajuan_kebutuhan_bmn_satker_barang_jumlah_check CHECK ((jumlah > 0))
);


--
-- Name: TABLE pengajuan_kebutuhan_bmn_satker_barang; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.pengajuan_kebutuhan_bmn_satker_barang IS 'Individual goods requested per satker';


--
-- Name: pengajuan_pakaian_dinas; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.pengajuan_pakaian_dinas (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    nama character varying(255) NOT NULL,
    deskripsi text,
    tgl_mulai date,
    tgl_selesai date,
    is_reguler boolean DEFAULT true,
    tahun integer NOT NULL,
    pilihan_satker character varying(20) NOT NULL,
    dengan_unit_kerja boolean DEFAULT false,
    jenis_pakaian_dinas_id uuid,
    status_kode integer DEFAULT 1000 NOT NULL,
    created_by uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    document_id uuid,
    document_url text,
    CONSTRAINT chk_pengajuan_pakaian_dinas_periode_order CHECK (((tgl_mulai IS NULL) OR (tgl_selesai IS NULL) OR (tgl_mulai <= tgl_selesai))),
    CONSTRAINT pengajuan_pakaian_dinas_pilihan_satker_check CHECK (((pilihan_satker)::text = ANY ((ARRAY['all'::character varying, 'sebagian'::character varying])::text[])))
);


--
-- Name: TABLE pengajuan_pakaian_dinas; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.pengajuan_pakaian_dinas IS 'Main request header for uniform procurement';


--
-- Name: COLUMN pengajuan_pakaian_dinas.tgl_mulai; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.pengajuan_pakaian_dinas.tgl_mulai IS 'Periode pengajuan: tanggal mulai. Wajib utk pengajuan baru (V029, Fase 1.8); legacy row boleh NULL.';


--
-- Name: COLUMN pengajuan_pakaian_dinas.tgl_selesai; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.pengajuan_pakaian_dinas.tgl_selesai IS 'Periode pengajuan: tanggal selesai (≥ tgl_mulai, enforced via CHECK constraint).';


--
-- Name: COLUMN pengajuan_pakaian_dinas.created_at; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.pengajuan_pakaian_dinas.created_at IS 'Creation timestamp (TIMESTAMPTZ)';


--
-- Name: COLUMN pengajuan_pakaian_dinas.updated_at; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.pengajuan_pakaian_dinas.updated_at IS 'Last update timestamp (TIMESTAMPTZ)';


--
-- Name: COLUMN pengajuan_pakaian_dinas.document_id; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.pengajuan_pakaian_dinas.document_id IS 'Reference to generated rekapitulasi document';


--
-- Name: COLUMN pengajuan_pakaian_dinas.document_url; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.pengajuan_pakaian_dinas.document_url IS 'URL to download rekapitulasi document';


--
-- Name: pengajuan_pakaian_dinas_aktivitas; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.pengajuan_pakaian_dinas_aktivitas (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    pengajuan_id uuid NOT NULL,
    aktivitas_id integer NOT NULL,
    user_id uuid NOT NULL,
    catatan text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE pengajuan_pakaian_dinas_aktivitas; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.pengajuan_pakaian_dinas_aktivitas IS 'Activity log for pakaian dinas workflow transitions';


--
-- Name: pengajuan_pakaian_dinas_pakaian; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.pengajuan_pakaian_dinas_pakaian (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    pengajuan_id uuid NOT NULL,
    jenis_pakaian_id uuid NOT NULL,
    jenis_pakaian_nama character varying(255) NOT NULL,
    spesifikasi_id uuid NOT NULL,
    spesifikasi_nama character varying(255) NOT NULL,
    spesifikasi_ukuran_group character varying(50) NOT NULL,
    subspesifikasi_id uuid,
    subspesifikasi_nama character varying(255),
    subspesifikasi_gender character varying(10)
);


--
-- Name: pengajuan_pakaian_dinas_satker; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.pengajuan_pakaian_dinas_satker (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    pengajuan_id uuid NOT NULL,
    satker_id uuid NOT NULL,
    satker_pusat_id uuid,
    id_kejati uuid,
    id_kejari uuid,
    id_cabjari uuid,
    status_kode integer DEFAULT 1000 NOT NULL,
    created_by uuid,
    updated_by uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: pengajuan_pakaian_dinas_satker_aktivitas; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    pengajuan_satker_id uuid NOT NULL,
    status_kode integer NOT NULL,
    komentar text,
    nip character varying(30),
    nama character varying(500),
    pangkat character varying(255),
    jabatan character varying(500),
    role character varying(100),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: pengajuan_pakaian_dinas_satker_pegawai; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.pengajuan_pakaian_dinas_satker_pegawai (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    pengajuan_satker_id uuid NOT NULL,
    nip character varying(30) NOT NULL,
    nama character varying(500) NOT NULL,
    pangkat character varying(255),
    jabatan character varying(500),
    eselon character varying(20),
    jenis_kelamin character varying(1) NOT NULL,
    gol_kd character varying(20),
    jenis character varying(100),
    with_hijab boolean DEFAULT false,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pengajuan_pakaian_dinas_satker_pegawai_jenis_kelamin_check CHECK (((jenis_kelamin)::text = ANY ((ARRAY['L'::character varying, 'P'::character varying])::text[])))
);


--
-- Name: pengajuan_pakaian_dinas_satker_pegawai_ukuran; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran (
    pengajuan_satker_id uuid NOT NULL,
    pegawai_id uuid NOT NULL,
    pakaian_id uuid NOT NULL,
    ukuran character varying(20) NOT NULL
);


--
-- Name: pengajuan_pakaian_dinas_satker_terpilih; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.pengajuan_pakaian_dinas_satker_terpilih (
    pengajuan_id uuid NOT NULL,
    satker_id uuid NOT NULL,
    satker_pusat_id uuid,
    is_show_in_form boolean DEFAULT true
);


--
-- Name: penghapusan_bmn; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.penghapusan_bmn (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    satker_id uuid NOT NULL,
    asset_id uuid NOT NULL,
    kode_barang character varying(50) NOT NULL,
    nama_barang character varying(255) NOT NULL,
    nup character varying(50) NOT NULL,
    tanggal_penghapusan date NOT NULL,
    alasan text NOT NULL,
    metode_penghapusan character varying(100) NOT NULL,
    nilai_residu double precision,
    status character varying(50) DEFAULT 'DRAFT'::character varying NOT NULL,
    status_kode integer,
    document_id uuid,
    document_url text,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    lampiran_persyaratan text,
    lampiran_pendukung jsonb DEFAULT '[]'::jsonb,
    catatan_operator text,
    catatan_validator_wilayah text,
    catatan_validator_pusat text,
    validator_wilayah_id uuid,
    validator_pusat_id uuid,
    tanggal_submit_wilayah timestamp with time zone,
    tanggal_verifikasi_wilayah timestamp with time zone,
    tanggal_submit_pusat timestamp with time zone,
    tanggal_verifikasi_pusat timestamp with time zone,
    konsep_sk_url text,
    konsep_sk_generated_at timestamp with time zone,
    signed_sk_pdf_url text,
    signed_sk_pdf_uploaded_at timestamp with time zone,
    is_completed boolean DEFAULT false,
    konsep_sk_pdf_url text,
    konsep_sk_pdf_generated_at timestamp with time zone,
    konsep_sk_docx_path text,
    konsep_sk_pdf_path text,
    nilai_perolehan double precision,
    nilai_perolehan_dari_backfill boolean DEFAULT false NOT NULL,
    surat_usulan_file_url text,
    surat_usulan_uploaded_at timestamp with time zone,
    kewenangan_penetap_sk text DEFAULT 'PUSAT'::text NOT NULL,
    penetap_sk_jabatan text,
    konsep_sk_wilayah_url text,
    konsep_sk_wilayah_pdf_url text,
    konsep_sk_wilayah_generated_at timestamp with time zone,
    signed_sk_wilayah_pdf_url text,
    signed_sk_wilayah_pdf_uploaded_at timestamp with time zone,
    CONSTRAINT chk_penghapusan_bmn_kewenangan CHECK ((kewenangan_penetap_sk = ANY (ARRAY['PUSAT'::text, 'WILAYAH'::text])))
);


--
-- Name: TABLE penghapusan_bmn; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.penghapusan_bmn IS 'BMN disposal records with workflow support';


--
-- Name: COLUMN penghapusan_bmn.metode_penghapusan; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.metode_penghapusan IS 'Disposal method: DIJUAL (sold), DIHIBAHKAN (donated), DIMUSNAHKAN (destroyed)';


--
-- Name: COLUMN penghapusan_bmn.nilai_residu; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.nilai_residu IS 'DEPRECATED (V029). Gunakan nilai_perolehan. Dipertahankan utk audit data lama; akan di-drop setelah verifikasi.';


--
-- Name: COLUMN penghapusan_bmn.status; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.status IS 'Workflow status: DRAFT, SUBMITTED, REVIEWED, APPROVED, REJECTED, CANCELLED';


--
-- Name: COLUMN penghapusan_bmn.document_id; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.document_id IS 'Generated SK Penghapusan document ID';


--
-- Name: COLUMN penghapusan_bmn.document_url; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.document_url IS 'URL to download SK Penghapusan document';


--
-- Name: COLUMN penghapusan_bmn.konsep_sk_url; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.konsep_sk_url IS 'Public download URL for the DOCX (editable) konsep SK';


--
-- Name: COLUMN penghapusan_bmn.konsep_sk_pdf_url; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.konsep_sk_pdf_url IS 'Public download URL for the PDF (final) konsep SK';


--
-- Name: COLUMN penghapusan_bmn.konsep_sk_docx_path; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.konsep_sk_docx_path IS 'Server-side filesystem path the DOCX route handler streams from';


--
-- Name: COLUMN penghapusan_bmn.konsep_sk_pdf_path; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.konsep_sk_pdf_path IS 'Server-side filesystem path the PDF route handler streams from';


--
-- Name: COLUMN penghapusan_bmn.nilai_perolehan; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.nilai_perolehan IS 'Nilai perolehan aset (harga pembelian). Menggantikan nilai_residu yang keliru secara semantik.';


--
-- Name: COLUMN penghapusan_bmn.nilai_perolehan_dari_backfill; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.nilai_perolehan_dari_backfill IS 'TRUE jika nilai_perolehan diisi dari backfill kolom legacy nilai_residu — perlu diverifikasi user.';


--
-- Name: COLUMN penghapusan_bmn.surat_usulan_file_url; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.surat_usulan_file_url IS 'URL Surat Usulan (1 file wajib) yg di-upload via /penghapusan-bmn/{id}/lampiran.';


--
-- Name: COLUMN penghapusan_bmn.kewenangan_penetap_sk; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.kewenangan_penetap_sk IS 'Otoritas penetap SK: PUSAT (Jaksa Agung Muda Pembinaan) atau WILAYAH (Kepala Kejaksaan Tinggi). Menentukan branching workflow & lokasi SK signed disimpan (kolom kanonik vs kolom _wilayah).';


--
-- Name: COLUMN penghapusan_bmn.penetap_sk_jabatan; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.penetap_sk_jabatan IS 'Label jabatan penetap SK (cth: "Jaksa Agung Muda Pembinaan" atau "Kepala Kejaksaan Tinggi DKI Jakarta"). Disimpan utk audit & cetak template.';


--
-- Name: COLUMN penghapusan_bmn.konsep_sk_wilayah_url; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.konsep_sk_wilayah_url IS 'URL konsep SK DOCX (editable) jalur WILAYAH. NULL jika kewenangan=PUSAT.';


--
-- Name: COLUMN penghapusan_bmn.signed_sk_wilayah_pdf_url; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.signed_sk_wilayah_pdf_url IS 'URL SK PDF yg sudah ditandatangani Kepala Kejati. NULL jika kewenangan=PUSAT.';


--
-- Name: penghapusan_bmn_aktivitas; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.penghapusan_bmn_aktivitas (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    penghapusan_id uuid NOT NULL,
    aktivitas_id integer NOT NULL,
    user_id uuid NOT NULL,
    catatan text,
    document_id uuid,
    document_url text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE penghapusan_bmn_aktivitas; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.penghapusan_bmn_aktivitas IS 'Workflow activity log for penghapusan BMN';


--
-- Name: COLUMN penghapusan_bmn_aktivitas.document_id; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn_aktivitas.document_id IS 'Document generated during this activity (e.g., SK Penghapusan)';


--
-- Name: COLUMN penghapusan_bmn_aktivitas.document_url; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn_aktivitas.document_url IS 'URL to download document generated during this activity';


--
-- Name: penghapusan_bmn_item; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.penghapusan_bmn_item (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    penghapusan_id uuid NOT NULL,
    asset_id uuid,
    kode_barang character varying(50) NOT NULL,
    nama_barang character varying(255) NOT NULL,
    nup character varying(50) NOT NULL,
    nilai_perolehan double precision,
    nilai_perolehan_dari_backfill boolean DEFAULT false NOT NULL,
    kondisi character varying(50),
    urutan integer DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE penghapusan_bmn_item; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.penghapusan_bmn_item IS 'Item BMN dalam satu Usulan SK Penghapusan (Fase 2.8). 1 usulan → N item. Tabel utama tetap menyimpan item pertama untuk backward compatibility.';


--
-- Name: COLUMN penghapusan_bmn_item.urutan; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.penghapusan_bmn_item.urutan IS 'Urutan tampil/cetak item dalam SK (1-based).';


--
-- Name: penghapusan_bmn_lampiran; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.penghapusan_bmn_lampiran (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    penghapusan_id uuid NOT NULL,
    nama text NOT NULL,
    file_url text NOT NULL,
    content_type text,
    size_bytes bigint,
    uploaded_by uuid,
    uploaded_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE penghapusan_bmn_lampiran; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.penghapusan_bmn_lampiran IS 'Lampiran pendukung Usulan SK Penghapusan BMN (multi-file, opsional). File disimpan via DocumentStorage; baris ini hanya metadata.';


--
-- Name: riwayat_pemenuhan; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.riwayat_pemenuhan (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    kebutuhan_bmn_id uuid,
    roadmap_id uuid,
    satker_id uuid NOT NULL,
    tahun_anggaran integer NOT NULL,
    kode_barang character varying(50) NOT NULL,
    nama_barang character varying(255) NOT NULL,
    jumlah_terpenuhi integer NOT NULL,
    nilai_perolehan double precision,
    sumber_data character varying(50) NOT NULL,
    sumber_keterangan text,
    tanggal_pemenuhan date NOT NULL,
    nomor_dokumen character varying(100),
    file_dokumen jsonb DEFAULT '[]'::jsonb,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    CONSTRAINT riwayat_pemenuhan_jumlah_terpenuhi_check CHECK ((jumlah_terpenuhi > 0)),
    CONSTRAINT riwayat_pemenuhan_sumber_data_check CHECK (((sumber_data)::text = ANY ((ARRAY['SIMAN'::character varying, 'HIBAH'::character varying, 'PNBP'::character varying, 'APBN'::character varying, 'APBD'::character varying, 'LAINNYA'::character varying])::text[])))
);


--
-- Name: TABLE riwayat_pemenuhan; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.riwayat_pemenuhan IS 'History of BMN requirement fulfillment from various sources';


--
-- Name: COLUMN riwayat_pemenuhan.sumber_data; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.riwayat_pemenuhan.sumber_data IS 'Source: SIMAN (procurement), HIBAH (donation), PNBP, APBN, APBD, LAINNYA';


--
-- Name: roadmap_sarpras; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.roadmap_sarpras (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    satker_id uuid NOT NULL,
    periode_mulai integer NOT NULL,
    periode_akhir integer NOT NULL,
    kode_barang character varying(50) NOT NULL,
    nama_barang character varying(255) NOT NULL,
    tahun_rencana integer NOT NULL,
    jumlah_kebutuhan integer NOT NULL,
    jumlah_terpenuhi integer DEFAULT 0,
    estimasi_anggaran double precision,
    realisasi_anggaran double precision DEFAULT 0,
    status_pemenuhan character varying(50) DEFAULT 'PLANNED'::character varying,
    prioritas integer DEFAULT 0,
    keterangan text,
    created_by uuid,
    updated_by uuid,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    CONSTRAINT chk_jumlah_valid CHECK ((jumlah_terpenuhi <= jumlah_kebutuhan)),
    CONSTRAINT chk_periode_valid CHECK ((periode_akhir = (periode_mulai + 4))),
    CONSTRAINT chk_tahun_in_periode CHECK (((tahun_rencana >= periode_mulai) AND (tahun_rencana <= periode_akhir))),
    CONSTRAINT roadmap_sarpras_jumlah_kebutuhan_check CHECK ((jumlah_kebutuhan > 0)),
    CONSTRAINT roadmap_sarpras_jumlah_terpenuhi_check CHECK ((jumlah_terpenuhi >= 0)),
    CONSTRAINT roadmap_sarpras_status_pemenuhan_check CHECK (((status_pemenuhan)::text = ANY ((ARRAY['PLANNED'::character varying, 'IN_PROGRESS'::character varying, 'COMPLETED'::character varying, 'CANCELLED'::character varying])::text[])))
);


--
-- Name: TABLE roadmap_sarpras; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.roadmap_sarpras IS '5-year infrastructure and facilities roadmap planning';


--
-- Name: COLUMN roadmap_sarpras.periode_mulai; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.roadmap_sarpras.periode_mulai IS 'Start year of 5-year period (e.g., 2025)';


--
-- Name: COLUMN roadmap_sarpras.periode_akhir; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.roadmap_sarpras.periode_akhir IS 'End year of 5-year period (e.g., 2029)';


--
-- Name: COLUMN roadmap_sarpras.tahun_rencana; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.roadmap_sarpras.tahun_rencana IS 'Specific year within the period for this item';


--
-- Name: v_izin_pemakaian_aktif; Type: VIEW; Schema: perlengkapan; Owner: -
--

CREATE VIEW perlengkapan.v_izin_pemakaian_aktif AS
 SELECT i.id,
    i.nomor_izin,
    i.jenis_bmn,
    i.bmn_nup,
    i.bmn_nama_barang,
    i.no_polisi,
    i.pegawai_nip,
    i.pegawai_nama,
    i.pegawai_jabatan,
    i.pegawai_satker_nama,
    i.tanggal_mulai,
    i.tanggal_selesai,
    i.status,
    (i.tanggal_selesai - CURRENT_DATE) AS days_until_expiry,
        CASE
            WHEN (i.tanggal_selesai < CURRENT_DATE) THEN 'EXPIRED'::text
            WHEN (i.tanggal_selesai < (CURRENT_DATE + '30 days'::interval)) THEN 'EXPIRING_SOON'::text
            ELSE 'ACTIVE'::text
        END AS expiry_status
   FROM perlengkapan.izin_pemakaian_bmn i
  WHERE ((i.status)::text = 'ACTIVE'::text)
  ORDER BY i.tanggal_selesai;


--
-- Name: VIEW v_izin_pemakaian_aktif; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON VIEW perlengkapan.v_izin_pemakaian_aktif IS 'Active BMN usage permits with expiry status';


--
-- Name: v_mapping_progress; Type: VIEW; Schema: perlengkapan; Owner: -
--

CREATE VIEW perlengkapan.v_mapping_progress AS
 SELECT mapping_kodefikasi.satker_id,
    count(*) AS total_mapping,
    count(*) FILTER (WHERE ((mapping_kodefikasi.status_mapping)::text = 'PROPOSED'::text)) AS proposed_count,
    count(*) FILTER (WHERE ((mapping_kodefikasi.status_mapping)::text = 'VERIFIED'::text)) AS verified_count,
    count(*) FILTER (WHERE ((mapping_kodefikasi.status_mapping)::text = 'APPROVED'::text)) AS approved_count,
    count(*) FILTER (WHERE ((mapping_kodefikasi.status_mapping)::text = 'REJECTED'::text)) AS rejected_count,
    round((((count(*) FILTER (WHERE ((mapping_kodefikasi.status_mapping)::text = 'APPROVED'::text)))::numeric / (NULLIF(count(*), 0))::numeric) * (100)::numeric), 2) AS approval_rate
   FROM perlengkapan.mapping_kodefikasi
  GROUP BY mapping_kodefikasi.satker_id
  ORDER BY mapping_kodefikasi.satker_id;


--
-- Name: VIEW v_mapping_progress; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON VIEW perlengkapan.v_mapping_progress IS 'Progress of code mapping by satker';


--
-- Name: v_roadmap_realization; Type: VIEW; Schema: perlengkapan; Owner: -
--

CREATE VIEW perlengkapan.v_roadmap_realization AS
 SELECT r.id,
    r.satker_id,
    r.periode_mulai,
    r.periode_akhir,
    r.tahun_rencana,
    r.kode_barang,
    r.nama_barang,
    r.jumlah_kebutuhan,
    r.jumlah_terpenuhi,
    r.estimasi_anggaran,
    r.realisasi_anggaran,
    r.status_pemenuhan,
        CASE
            WHEN (r.jumlah_kebutuhan > 0) THEN round((((r.jumlah_terpenuhi)::numeric / (r.jumlah_kebutuhan)::numeric) * (100)::numeric), 2)
            ELSE (0)::numeric
        END AS persentase_pemenuhan,
        CASE
            WHEN (r.estimasi_anggaran > (0)::double precision) THEN round((((r.realisasi_anggaran)::numeric / (r.estimasi_anggaran)::numeric) * (100)::numeric), 2)
            ELSE (0)::numeric
        END AS persentase_realisasi_anggaran,
    ( SELECT count(*) AS count
           FROM perlengkapan.riwayat_pemenuhan rp
          WHERE (rp.roadmap_id = r.id)) AS jumlah_pemenuhan
   FROM perlengkapan.roadmap_sarpras r
  ORDER BY r.satker_id, r.tahun_rencana;


--
-- Name: VIEW v_roadmap_realization; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON VIEW perlengkapan.v_roadmap_realization IS 'Roadmap planning vs actual realization comparison';


--
-- Name: vw_kebutuhan_bmn_summary; Type: VIEW; Schema: perlengkapan; Owner: -
--

CREATE VIEW perlengkapan.vw_kebutuhan_bmn_summary AS
 SELECT p.id,
    p.nama,
    p.tahun,
    p.status_kode,
    m.nama AS status_nama,
    count(DISTINCT ps.id) AS total_satker,
    count(DISTINCT psb.id) AS total_barang,
    COALESCE(sum(psb.jumlah), (0)::bigint) AS total_jumlah_diminta,
    COALESCE(sum(psb.jml_setuju), (0)::bigint) AS total_jumlah_disetujui,
    p.created_at,
    p.updated_at
   FROM (((perlengkapan.pengajuan_kebutuhan_bmn p
     LEFT JOIN perlengkapan.ms_aktivitas_bmn m ON ((p.status_kode = m.kode)))
     LEFT JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker ps ON ((p.id = ps.pengajuan_id)))
     LEFT JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker_barang psb ON ((ps.id = psb.pengajuan_satker_id)))
  GROUP BY p.id, p.nama, p.tahun, p.status_kode, m.nama, p.created_at, p.updated_at;


--
-- Name: workflow_definitions; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.workflow_definitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(100) NOT NULL,
    description text,
    version integer DEFAULT 1 NOT NULL,
    config jsonb NOT NULL,
    is_active boolean DEFAULT true,
    created_by uuid,
    updated_by uuid,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);


--
-- Name: TABLE workflow_definitions; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.workflow_definitions IS 'Workflow configuration definitions with versioning';


--
-- Name: COLUMN workflow_definitions.config; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.workflow_definitions.config IS 'JSONB containing transitions, SLA, roles, etc.';


--
-- Name: workflow_delegations; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.workflow_delegations (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    delegator_user_id uuid NOT NULL,
    delegator_nip character varying(30),
    delegator_nama character varying(255),
    delegator_role character varying(100) NOT NULL,
    delegate_user_id uuid NOT NULL,
    delegate_nip character varying(30),
    delegate_nama character varying(255),
    workflow_name character varying(100),
    entity_type character varying(50),
    valid_from timestamp with time zone DEFAULT now() NOT NULL,
    valid_until timestamp with time zone NOT NULL,
    status character varying(50) DEFAULT 'ACTIVE'::character varying NOT NULL,
    revoked_at timestamp with time zone,
    revoked_by uuid,
    revocation_reason text,
    reason text NOT NULL,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    CONSTRAINT no_self_delegation CHECK ((delegator_user_id <> delegate_user_id)),
    CONSTRAINT valid_delegation_period CHECK ((valid_until > valid_from)),
    CONSTRAINT workflow_delegations_status_check CHECK (((status)::text = ANY ((ARRAY['ACTIVE'::character varying, 'EXPIRED'::character varying, 'REVOKED'::character varying, 'CANCELLED'::character varying])::text[])))
);


--
-- Name: TABLE workflow_delegations; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.workflow_delegations IS 'Temporary delegation of workflow approval authority';


--
-- Name: COLUMN workflow_delegations.workflow_name; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.workflow_delegations.workflow_name IS 'NULL means delegation applies to all workflows';


--
-- Name: COLUMN workflow_delegations.entity_type; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.workflow_delegations.entity_type IS 'NULL means delegation applies to all entity types';


--
-- Name: workflow_escalations; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.workflow_escalations (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workflow_instance_id uuid NOT NULL,
    escalation_level integer DEFAULT 1 NOT NULL,
    escalated_state character varying(100) NOT NULL,
    sla_minutes integer NOT NULL,
    sla_deadline timestamp with time zone NOT NULL,
    actual_duration_minutes integer,
    escalated_to_user_id uuid,
    escalated_to_role character varying(100),
    notification_sent boolean DEFAULT false,
    notification_sent_at timestamp with time zone,
    resolved boolean DEFAULT false,
    resolved_at timestamp with time zone,
    resolution_action character varying(100),
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);


--
-- Name: TABLE workflow_escalations; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.workflow_escalations IS 'SLA breach escalations and notifications';


--
-- Name: COLUMN workflow_escalations.escalation_level; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.workflow_escalations.escalation_level IS 'Level of escalation (1=first, 2=second, etc.)';


--
-- Name: workflow_instances; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.workflow_instances (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workflow_definition_id uuid NOT NULL,
    workflow_name character varying(100) NOT NULL,
    workflow_version integer NOT NULL,
    entity_type character varying(50) NOT NULL,
    entity_id uuid NOT NULL,
    current_state character varying(100) NOT NULL,
    current_state_code integer,
    state_entered_at timestamp with time zone DEFAULT now(),
    sla_minutes integer,
    sla_deadline timestamp with time zone,
    is_sla_breached boolean DEFAULT false,
    sla_breach_notified_at timestamp with time zone,
    status character varying(50) DEFAULT 'ACTIVE'::character varying NOT NULL,
    completed_at timestamp with time zone,
    completion_state character varying(100),
    metadata jsonb DEFAULT '{}'::jsonb,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    CONSTRAINT workflow_instances_entity_type_check CHECK (((entity_type)::text = ANY ((ARRAY['kebutuhan_bmn'::character varying, 'pakaian_dinas'::character varying, 'pemakaian_bmn'::character varying, 'roadmap_sarpras'::character varying])::text[]))),
    CONSTRAINT workflow_instances_status_check CHECK (((status)::text = ANY ((ARRAY['ACTIVE'::character varying, 'COMPLETED'::character varying, 'CANCELLED'::character varying, 'SUSPENDED'::character varying])::text[])))
);


--
-- Name: TABLE workflow_instances; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.workflow_instances IS 'Active workflow instance tracking with SLA monitoring';


--
-- Name: COLUMN workflow_instances.entity_type; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.workflow_instances.entity_type IS 'Type of entity this workflow is attached to';


--
-- Name: COLUMN workflow_instances.sla_deadline; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.workflow_instances.sla_deadline IS 'Calculated deadline based on state_entered_at + sla_minutes';


--
-- Name: workflow_transitions; Type: TABLE; Schema: perlengkapan; Owner: -
--

CREATE TABLE perlengkapan.workflow_transitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workflow_instance_id uuid NOT NULL,
    from_state character varying(100),
    from_state_code integer,
    to_state character varying(100) NOT NULL,
    to_state_code integer,
    actor_user_id uuid NOT NULL,
    actor_nip character varying(30),
    actor_nama character varying(255),
    actor_jabatan character varying(255),
    actor_role character varying(100),
    action character varying(100) NOT NULL,
    komentar text,
    is_delegated boolean DEFAULT false,
    delegated_from_user_id uuid,
    delegation_id uuid,
    metadata jsonb DEFAULT '{}'::jsonb,
    transitioned_at timestamp with time zone DEFAULT now(),
    ip_address inet
);


--
-- Name: TABLE workflow_transitions; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON TABLE perlengkapan.workflow_transitions IS 'Immutable audit trail of all workflow state transitions';


--
-- Name: COLUMN workflow_transitions.is_delegated; Type: COMMENT; Schema: perlengkapan; Owner: -
--

COMMENT ON COLUMN perlengkapan.workflow_transitions.is_delegated IS 'Whether this action was performed via delegation';


--
-- Name: batch_operation_log id; Type: DEFAULT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.batch_operation_log ALTER COLUMN id SET DEFAULT nextval('perlengkapan.batch_operation_log_id_seq'::regclass);


--
-- Name: ms_aktivitas_bmn id; Type: DEFAULT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.ms_aktivitas_bmn ALTER COLUMN id SET DEFAULT nextval('perlengkapan.ms_aktivitas_bmn_id_seq'::regclass);


--
-- Name: archive_collections archive_collections_pkey; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.archive_collections
    ADD CONSTRAINT archive_collections_pkey PRIMARY KEY (id);


--
-- Name: archive_documents archive_documents_collection_id_document_id_key; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.archive_documents
    ADD CONSTRAINT archive_documents_collection_id_document_id_key UNIQUE (collection_id, document_id);


--
-- Name: archive_documents archive_documents_pkey; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.archive_documents
    ADD CONSTRAINT archive_documents_pkey PRIMARY KEY (id);


--
-- Name: audit_log audit_log_pkey; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.audit_log
    ADD CONSTRAINT audit_log_pkey PRIMARY KEY (id);


--
-- Name: document_permissions document_permissions_document_id_user_id_key; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.document_permissions
    ADD CONSTRAINT document_permissions_document_id_user_id_key UNIQUE (document_id, user_id);


--
-- Name: document_permissions document_permissions_pkey; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.document_permissions
    ADD CONSTRAINT document_permissions_pkey PRIMARY KEY (id);


--
-- Name: document_tags document_tags_document_id_tag_key; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.document_tags
    ADD CONSTRAINT document_tags_document_id_tag_key UNIQUE (document_id, tag);


--
-- Name: document_tags document_tags_pkey; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.document_tags
    ADD CONSTRAINT document_tags_pkey PRIMARY KEY (id);


--
-- Name: document_templates document_templates_name_version_key; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.document_templates
    ADD CONSTRAINT document_templates_name_version_key UNIQUE (name, version);


--
-- Name: document_templates document_templates_pkey; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.document_templates
    ADD CONSTRAINT document_templates_pkey PRIMARY KEY (id);


--
-- Name: document_versions document_versions_document_id_version_key; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.document_versions
    ADD CONSTRAINT document_versions_document_id_version_key UNIQUE (document_id, version);


--
-- Name: document_versions document_versions_pkey; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.document_versions
    ADD CONSTRAINT document_versions_pkey PRIMARY KEY (id);


--
-- Name: documents documents_pkey; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.documents
    ADD CONSTRAINT documents_pkey PRIMARY KEY (id);


--
-- Name: generated_documents generated_documents_document_number_key; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.generated_documents
    ADD CONSTRAINT generated_documents_document_number_key UNIQUE (document_number);


--
-- Name: generated_documents generated_documents_pkey; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.generated_documents
    ADD CONSTRAINT generated_documents_pkey PRIMARY KEY (id);


--
-- Name: ocr_results ocr_results_pkey; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.ocr_results
    ADD CONSTRAINT ocr_results_pkey PRIMARY KEY (id);


--
-- Name: template_versions template_versions_pkey; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.template_versions
    ADD CONSTRAINT template_versions_pkey PRIMARY KEY (id);


--
-- Name: template_versions template_versions_template_id_version_key; Type: CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.template_versions
    ADD CONSTRAINT template_versions_template_id_version_key UNIQUE (template_id, version);


--
-- Name: delivery_status delivery_status_pkey; Type: CONSTRAINT; Schema: notifikasi; Owner: -
--

ALTER TABLE ONLY notifikasi.delivery_status
    ADD CONSTRAINT delivery_status_pkey PRIMARY KEY (id);


--
-- Name: email_templates email_templates_name_key; Type: CONSTRAINT; Schema: notifikasi; Owner: -
--

ALTER TABLE ONLY notifikasi.email_templates
    ADD CONSTRAINT email_templates_name_key UNIQUE (name);


--
-- Name: email_templates email_templates_pkey; Type: CONSTRAINT; Schema: notifikasi; Owner: -
--

ALTER TABLE ONLY notifikasi.email_templates
    ADD CONSTRAINT email_templates_pkey PRIMARY KEY (id);


--
-- Name: in_app_notifications in_app_notifications_pkey; Type: CONSTRAINT; Schema: notifikasi; Owner: -
--

ALTER TABLE ONLY notifikasi.in_app_notifications
    ADD CONSTRAINT in_app_notifications_pkey PRIMARY KEY (id);


--
-- Name: notification_queue notification_queue_pkey; Type: CONSTRAINT; Schema: notifikasi; Owner: -
--

ALTER TABLE ONLY notifikasi.notification_queue
    ADD CONSTRAINT notification_queue_pkey PRIMARY KEY (id);


--
-- Name: sms_templates sms_templates_name_key; Type: CONSTRAINT; Schema: notifikasi; Owner: -
--

ALTER TABLE ONLY notifikasi.sms_templates
    ADD CONSTRAINT sms_templates_name_key UNIQUE (name);


--
-- Name: sms_templates sms_templates_pkey; Type: CONSTRAINT; Schema: notifikasi; Owner: -
--

ALTER TABLE ONLY notifikasi.sms_templates
    ADD CONSTRAINT sms_templates_pkey PRIMARY KEY (id);


--
-- Name: user_notification_preferences user_notification_preferences_pkey; Type: CONSTRAINT; Schema: notifikasi; Owner: -
--

ALTER TABLE ONLY notifikasi.user_notification_preferences
    ADD CONSTRAINT user_notification_preferences_pkey PRIMARY KEY (id);


--
-- Name: user_notification_preferences user_notification_preferences_user_id_key; Type: CONSTRAINT; Schema: notifikasi; Owner: -
--

ALTER TABLE ONLY notifikasi.user_notification_preferences
    ADD CONSTRAINT user_notification_preferences_user_id_key UNIQUE (user_id);


--
-- Name: analisis_kebutuhan analisis_kebutuhan_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.analisis_kebutuhan
    ADD CONSTRAINT analisis_kebutuhan_pkey PRIMARY KEY (id);


--
-- Name: audit_log audit_log_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.audit_log
    ADD CONSTRAINT audit_log_pkey PRIMARY KEY (id);


--
-- Name: batch_operation_log batch_operation_log_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.batch_operation_log
    ADD CONSTRAINT batch_operation_log_pkey PRIMARY KEY (id);


--
-- Name: export_jobs export_jobs_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.export_jobs
    ADD CONSTRAINT export_jobs_pkey PRIMARY KEY (id);


--
-- Name: izin_pemakaian_bmn_aktivitas izin_pemakaian_bmn_aktivitas_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.izin_pemakaian_bmn_aktivitas
    ADD CONSTRAINT izin_pemakaian_bmn_aktivitas_pkey PRIMARY KEY (id);


--
-- Name: izin_pemakaian_bmn izin_pemakaian_bmn_nomor_izin_key; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.izin_pemakaian_bmn
    ADD CONSTRAINT izin_pemakaian_bmn_nomor_izin_key UNIQUE (nomor_izin);


--
-- Name: izin_pemakaian_bmn izin_pemakaian_bmn_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.izin_pemakaian_bmn
    ADD CONSTRAINT izin_pemakaian_bmn_pkey PRIMARY KEY (id);


--
-- Name: mapping_kodefikasi mapping_kodefikasi_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.mapping_kodefikasi
    ADD CONSTRAINT mapping_kodefikasi_pkey PRIMARY KEY (id);


--
-- Name: ms_aktivitas_bmn ms_aktivitas_bmn_kode_key; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.ms_aktivitas_bmn
    ADD CONSTRAINT ms_aktivitas_bmn_kode_key UNIQUE (kode);


--
-- Name: ms_aktivitas_bmn ms_aktivitas_bmn_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.ms_aktivitas_bmn
    ADD CONSTRAINT ms_aktivitas_bmn_pkey PRIMARY KEY (id);


--
-- Name: ms_jenis_pakaian_dinas ms_jenis_pakaian_dinas_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.ms_jenis_pakaian_dinas
    ADD CONSTRAINT ms_jenis_pakaian_dinas_pkey PRIMARY KEY (id);


--
-- Name: ms_spesifikasi_pakaian_dinas_foto ms_spesifikasi_pakaian_dinas_foto_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.ms_spesifikasi_pakaian_dinas_foto
    ADD CONSTRAINT ms_spesifikasi_pakaian_dinas_foto_pkey PRIMARY KEY (id);


--
-- Name: ms_spesifikasi_pakaian_dinas ms_spesifikasi_pakaian_dinas_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.ms_spesifikasi_pakaian_dinas
    ADD CONSTRAINT ms_spesifikasi_pakaian_dinas_pkey PRIMARY KEY (id);


--
-- Name: ms_subspesifikasi_pakaian_dinas ms_subspesifikasi_pakaian_dinas_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.ms_subspesifikasi_pakaian_dinas
    ADD CONSTRAINT ms_subspesifikasi_pakaian_dinas_pkey PRIMARY KEY (id);


--
-- Name: ms_ukuran ms_ukuran_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.ms_ukuran
    ADD CONSTRAINT ms_ukuran_pkey PRIMARY KEY (ukuran, "group");


--
-- Name: ms_workflow_status ms_workflow_status_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.ms_workflow_status
    ADD CONSTRAINT ms_workflow_status_pkey PRIMARY KEY (kode);


--
-- Name: parallel_approval_votes parallel_approval_votes_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.parallel_approval_votes
    ADD CONSTRAINT parallel_approval_votes_pkey PRIMARY KEY (id);


--
-- Name: parallel_approvals parallel_approvals_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.parallel_approvals
    ADD CONSTRAINT parallel_approvals_pkey PRIMARY KEY (id);


--
-- Name: pegawai_pakaian_dinas pegawai_pakaian_dinas_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pegawai_pakaian_dinas
    ADD CONSTRAINT pegawai_pakaian_dinas_pkey PRIMARY KEY (nip);


--
-- Name: pemakaian_bmn_items pemakaian_bmn_items_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pemakaian_bmn_items
    ADD CONSTRAINT pemakaian_bmn_items_pkey PRIMARY KEY (id);


--
-- Name: pengajuan_bmn_referensi_diizinkan pengajuan_bmn_referensi_diizinkan_pengajuan_id_kode_barang_key; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_bmn_referensi_diizinkan
    ADD CONSTRAINT pengajuan_bmn_referensi_diizinkan_pengajuan_id_kode_barang_key UNIQUE (pengajuan_id, kode_barang);


--
-- Name: pengajuan_bmn_referensi_diizinkan pengajuan_bmn_referensi_diizinkan_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_bmn_referensi_diizinkan
    ADD CONSTRAINT pengajuan_bmn_referensi_diizinkan_pkey PRIMARY KEY (id);


--
-- Name: pengajuan_kebutuhan_bmn_asset pengajuan_kebutuhan_bmn_asset_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_kebutuhan_bmn_asset
    ADD CONSTRAINT pengajuan_kebutuhan_bmn_asset_pkey PRIMARY KEY (id);


--
-- Name: pengajuan_kebutuhan_bmn pengajuan_kebutuhan_bmn_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_kebutuhan_bmn
    ADD CONSTRAINT pengajuan_kebutuhan_bmn_pkey PRIMARY KEY (id);


--
-- Name: pengajuan_kebutuhan_bmn_satker_aktivitas pengajuan_kebutuhan_bmn_satker_aktivitas_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
    ADD CONSTRAINT pengajuan_kebutuhan_bmn_satker_aktivitas_pkey PRIMARY KEY (id);


--
-- Name: pengajuan_kebutuhan_bmn_satker_barang pengajuan_kebutuhan_bmn_satker_barang_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
    ADD CONSTRAINT pengajuan_kebutuhan_bmn_satker_barang_pkey PRIMARY KEY (id);


--
-- Name: pengajuan_kebutuhan_bmn_satker pengajuan_kebutuhan_bmn_satker_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_kebutuhan_bmn_satker
    ADD CONSTRAINT pengajuan_kebutuhan_bmn_satker_pkey PRIMARY KEY (id);


--
-- Name: pengajuan_pakaian_dinas_aktivitas pengajuan_pakaian_dinas_aktivitas_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_aktivitas
    ADD CONSTRAINT pengajuan_pakaian_dinas_aktivitas_pkey PRIMARY KEY (id);


--
-- Name: pengajuan_pakaian_dinas_pakaian pengajuan_pakaian_dinas_pakaian_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_pakaian
    ADD CONSTRAINT pengajuan_pakaian_dinas_pakaian_pkey PRIMARY KEY (id);


--
-- Name: pengajuan_pakaian_dinas pengajuan_pakaian_dinas_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas
    ADD CONSTRAINT pengajuan_pakaian_dinas_pkey PRIMARY KEY (id);


--
-- Name: pengajuan_pakaian_dinas_satker_aktivitas pengajuan_pakaian_dinas_satker_aktivitas_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas
    ADD CONSTRAINT pengajuan_pakaian_dinas_satker_aktivitas_pkey PRIMARY KEY (id);


--
-- Name: pengajuan_pakaian_dinas_satker_pegawai pengajuan_pakaian_dinas_satker_pegawai_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_satker_pegawai
    ADD CONSTRAINT pengajuan_pakaian_dinas_satker_pegawai_pkey PRIMARY KEY (id);


--
-- Name: pengajuan_pakaian_dinas_satker_pegawai_ukuran pengajuan_pakaian_dinas_satker_pegawai_ukuran_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran
    ADD CONSTRAINT pengajuan_pakaian_dinas_satker_pegawai_ukuran_pkey PRIMARY KEY (pegawai_id, pakaian_id);


--
-- Name: pengajuan_pakaian_dinas_satker pengajuan_pakaian_dinas_satker_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_satker
    ADD CONSTRAINT pengajuan_pakaian_dinas_satker_pkey PRIMARY KEY (id);


--
-- Name: pengajuan_pakaian_dinas_satker_terpilih pengajuan_pakaian_dinas_satker_terpilih_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_satker_terpilih
    ADD CONSTRAINT pengajuan_pakaian_dinas_satker_terpilih_pkey PRIMARY KEY (pengajuan_id, satker_id);


--
-- Name: penghapusan_bmn_aktivitas penghapusan_bmn_aktivitas_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.penghapusan_bmn_aktivitas
    ADD CONSTRAINT penghapusan_bmn_aktivitas_pkey PRIMARY KEY (id);


--
-- Name: penghapusan_bmn_item penghapusan_bmn_item_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.penghapusan_bmn_item
    ADD CONSTRAINT penghapusan_bmn_item_pkey PRIMARY KEY (id);


--
-- Name: penghapusan_bmn_lampiran penghapusan_bmn_lampiran_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.penghapusan_bmn_lampiran
    ADD CONSTRAINT penghapusan_bmn_lampiran_pkey PRIMARY KEY (id);


--
-- Name: penghapusan_bmn penghapusan_bmn_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.penghapusan_bmn
    ADD CONSTRAINT penghapusan_bmn_pkey PRIMARY KEY (id);


--
-- Name: riwayat_pemenuhan riwayat_pemenuhan_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.riwayat_pemenuhan
    ADD CONSTRAINT riwayat_pemenuhan_pkey PRIMARY KEY (id);


--
-- Name: roadmap_sarpras roadmap_sarpras_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.roadmap_sarpras
    ADD CONSTRAINT roadmap_sarpras_pkey PRIMARY KEY (id);


--
-- Name: parallel_approval_votes unique_approver_vote; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.parallel_approval_votes
    ADD CONSTRAINT unique_approver_vote UNIQUE (parallel_approval_id, approver_user_id);


--
-- Name: workflow_instances unique_entity_workflow; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.workflow_instances
    ADD CONSTRAINT unique_entity_workflow UNIQUE (entity_type, entity_id);


--
-- Name: pengajuan_kebutuhan_bmn_satker unique_satker_per_pengajuan; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_kebutuhan_bmn_satker
    ADD CONSTRAINT unique_satker_per_pengajuan UNIQUE (pengajuan_id, satker_id);


--
-- Name: workflow_definitions unique_workflow_name_version; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.workflow_definitions
    ADD CONSTRAINT unique_workflow_name_version UNIQUE (name, version);


--
-- Name: workflow_definitions workflow_definitions_name_key; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.workflow_definitions
    ADD CONSTRAINT workflow_definitions_name_key UNIQUE (name);


--
-- Name: workflow_definitions workflow_definitions_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.workflow_definitions
    ADD CONSTRAINT workflow_definitions_pkey PRIMARY KEY (id);


--
-- Name: workflow_delegations workflow_delegations_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.workflow_delegations
    ADD CONSTRAINT workflow_delegations_pkey PRIMARY KEY (id);


--
-- Name: workflow_escalations workflow_escalations_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.workflow_escalations
    ADD CONSTRAINT workflow_escalations_pkey PRIMARY KEY (id);


--
-- Name: workflow_instances workflow_instances_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.workflow_instances
    ADD CONSTRAINT workflow_instances_pkey PRIMARY KEY (id);


--
-- Name: workflow_transitions workflow_transitions_pkey; Type: CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.workflow_transitions
    ADD CONSTRAINT workflow_transitions_pkey PRIMARY KEY (id);


--
-- Name: idx_archive_collections_owner; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_archive_collections_owner ON dokumen.archive_collections USING btree (owner_id);


--
-- Name: idx_archive_documents_collection; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_archive_documents_collection ON dokumen.archive_documents USING btree (collection_id);


--
-- Name: idx_archive_documents_document; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_archive_documents_document ON dokumen.archive_documents USING btree (document_id);


--
-- Name: idx_audit_log_document; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_audit_log_document ON dokumen.audit_log USING btree (document_id, "timestamp" DESC);


--
-- Name: idx_audit_log_timestamp; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_audit_log_timestamp ON dokumen.audit_log USING btree ("timestamp" DESC);


--
-- Name: idx_audit_log_user; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_audit_log_user ON dokumen.audit_log USING btree (user_id, "timestamp" DESC);


--
-- Name: idx_document_permissions_document; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_document_permissions_document ON dokumen.document_permissions USING btree (document_id);


--
-- Name: idx_document_permissions_user; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_document_permissions_user ON dokumen.document_permissions USING btree (user_id);


--
-- Name: idx_document_tags_document; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_document_tags_document ON dokumen.document_tags USING btree (document_id);


--
-- Name: idx_document_tags_tag; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_document_tags_tag ON dokumen.document_tags USING btree (tag);


--
-- Name: idx_document_versions_document; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_document_versions_document ON dokumen.document_versions USING btree (document_id, version DESC);


--
-- Name: idx_documents_archived; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_documents_archived ON dokumen.documents USING btree (is_archived);


--
-- Name: idx_documents_created_at; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_documents_created_at ON dokumen.documents USING btree (created_at DESC);


--
-- Name: idx_documents_metadata_gin; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_documents_metadata_gin ON dokumen.documents USING gin (metadata);


--
-- Name: idx_documents_owner; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_documents_owner ON dokumen.documents USING btree (owner_id);


--
-- Name: idx_generated_docs_generated_at; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_generated_docs_generated_at ON dokumen.generated_documents USING btree (generated_at DESC);


--
-- Name: idx_generated_docs_generated_by; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_generated_docs_generated_by ON dokumen.generated_documents USING btree (generated_by);


--
-- Name: idx_generated_docs_number; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_generated_docs_number ON dokumen.generated_documents USING btree (document_number);


--
-- Name: idx_generated_docs_status; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_generated_docs_status ON dokumen.generated_documents USING btree (status);


--
-- Name: idx_generated_docs_template; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_generated_docs_template ON dokumen.generated_documents USING btree (template_id);


--
-- Name: idx_ocr_results_document; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_ocr_results_document ON dokumen.ocr_results USING btree (document_id);


--
-- Name: idx_ocr_results_status; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_ocr_results_status ON dokumen.ocr_results USING btree (status);


--
-- Name: idx_template_versions_template; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_template_versions_template ON dokumen.template_versions USING btree (template_id, version DESC);


--
-- Name: idx_templates_active; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_templates_active ON dokumen.document_templates USING btree (is_active);


--
-- Name: idx_templates_created_at; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_templates_created_at ON dokumen.document_templates USING btree (created_at DESC);


--
-- Name: idx_templates_type; Type: INDEX; Schema: dokumen; Owner: -
--

CREATE INDEX idx_templates_type ON dokumen.document_templates USING btree (template_type);


--
-- Name: idx_delivery_status_created_at; Type: INDEX; Schema: notifikasi; Owner: -
--

CREATE INDEX idx_delivery_status_created_at ON notifikasi.delivery_status USING btree (created_at);


--
-- Name: idx_delivery_status_notification_id; Type: INDEX; Schema: notifikasi; Owner: -
--

CREATE INDEX idx_delivery_status_notification_id ON notifikasi.delivery_status USING btree (notification_id);


--
-- Name: idx_delivery_status_status; Type: INDEX; Schema: notifikasi; Owner: -
--

CREATE INDEX idx_delivery_status_status ON notifikasi.delivery_status USING btree (status);


--
-- Name: idx_in_app_notifications_created_at; Type: INDEX; Schema: notifikasi; Owner: -
--

CREATE INDEX idx_in_app_notifications_created_at ON notifikasi.in_app_notifications USING btree (created_at DESC);


--
-- Name: idx_in_app_notifications_priority; Type: INDEX; Schema: notifikasi; Owner: -
--

CREATE INDEX idx_in_app_notifications_priority ON notifikasi.in_app_notifications USING btree (priority);


--
-- Name: idx_in_app_notifications_read; Type: INDEX; Schema: notifikasi; Owner: -
--

CREATE INDEX idx_in_app_notifications_read ON notifikasi.in_app_notifications USING btree (user_id, read);


--
-- Name: idx_in_app_notifications_user_id; Type: INDEX; Schema: notifikasi; Owner: -
--

CREATE INDEX idx_in_app_notifications_user_id ON notifikasi.in_app_notifications USING btree (user_id);


--
-- Name: idx_notification_queue_created_at; Type: INDEX; Schema: notifikasi; Owner: -
--

CREATE INDEX idx_notification_queue_created_at ON notifikasi.notification_queue USING btree (created_at);


--
-- Name: idx_notification_queue_next_retry; Type: INDEX; Schema: notifikasi; Owner: -
--

CREATE INDEX idx_notification_queue_next_retry ON notifikasi.notification_queue USING btree (next_retry_at) WHERE ((status)::text = 'pending'::text);


--
-- Name: idx_notification_queue_notification_id; Type: INDEX; Schema: notifikasi; Owner: -
--

CREATE INDEX idx_notification_queue_notification_id ON notifikasi.notification_queue USING btree (notification_id);


--
-- Name: idx_notification_queue_status; Type: INDEX; Schema: notifikasi; Owner: -
--

CREATE INDEX idx_notification_queue_status ON notifikasi.notification_queue USING btree (status);


--
-- Name: idx_user_notification_preferences_user_id; Type: INDEX; Schema: notifikasi; Owner: -
--

CREATE INDEX idx_user_notification_preferences_user_id ON notifikasi.user_notification_preferences USING btree (user_id);


--
-- Name: idx_aktivitas_document_id; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_aktivitas_document_id ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas USING btree (document_id) WHERE (document_id IS NOT NULL);


--
-- Name: idx_analisis_kebutuhan_created_by; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_analisis_kebutuhan_created_by ON perlengkapan.analisis_kebutuhan USING btree (created_by);


--
-- Name: idx_analisis_kebutuhan_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_analisis_kebutuhan_status ON perlengkapan.analisis_kebutuhan USING btree (status);


--
-- Name: idx_audit_log_actor; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_audit_log_actor ON perlengkapan.audit_log USING btree (actor_user_id, occurred_at DESC);


--
-- Name: idx_audit_log_module; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_audit_log_module ON perlengkapan.audit_log USING btree (module, occurred_at DESC);


--
-- Name: idx_audit_log_occurred; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_audit_log_occurred ON perlengkapan.audit_log USING btree (occurred_at DESC);


--
-- Name: idx_audit_log_resource; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_audit_log_resource ON perlengkapan.audit_log USING btree (resource_type, resource_id);


--
-- Name: idx_audit_log_retention; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_audit_log_retention ON perlengkapan.audit_log USING btree (retention_until);


--
-- Name: idx_batch_operation_log_batch_id; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_batch_operation_log_batch_id ON perlengkapan.batch_operation_log USING btree (batch_id);


--
-- Name: idx_batch_operation_log_created_at; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_batch_operation_log_created_at ON perlengkapan.batch_operation_log USING btree (created_at DESC);


--
-- Name: idx_batch_operation_log_operation_type; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_batch_operation_log_operation_type ON perlengkapan.batch_operation_log USING btree (operation_type);


--
-- Name: idx_batch_operation_log_user_id; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_batch_operation_log_user_id ON perlengkapan.batch_operation_log USING btree (user_id);


--
-- Name: idx_delegation_delegate; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_delegation_delegate ON perlengkapan.workflow_delegations USING btree (delegate_user_id);


--
-- Name: idx_delegation_delegator; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_delegation_delegator ON perlengkapan.workflow_delegations USING btree (delegator_user_id);


--
-- Name: idx_delegation_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_delegation_status ON perlengkapan.workflow_delegations USING btree (status);


--
-- Name: idx_delegation_validity; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_delegation_validity ON perlengkapan.workflow_delegations USING btree (valid_from, valid_until) WHERE ((status)::text = 'ACTIVE'::text);


--
-- Name: idx_delegation_workflow; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_delegation_workflow ON perlengkapan.workflow_delegations USING btree (workflow_name) WHERE (workflow_name IS NOT NULL);


--
-- Name: idx_escalation_instance; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_escalation_instance ON perlengkapan.workflow_escalations USING btree (workflow_instance_id);


--
-- Name: idx_escalation_level; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_escalation_level ON perlengkapan.workflow_escalations USING btree (escalation_level);


--
-- Name: idx_escalation_notification; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_escalation_notification ON perlengkapan.workflow_escalations USING btree (notification_sent) WHERE (notification_sent = false);


--
-- Name: idx_escalation_resolved; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_escalation_resolved ON perlengkapan.workflow_escalations USING btree (resolved) WHERE (resolved = false);


--
-- Name: idx_export_jobs_created_at; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_export_jobs_created_at ON perlengkapan.export_jobs USING btree (created_at DESC);


--
-- Name: idx_export_jobs_created_by; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_export_jobs_created_by ON perlengkapan.export_jobs USING btree (created_by);


--
-- Name: idx_export_jobs_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_export_jobs_status ON perlengkapan.export_jobs USING btree (status);


--
-- Name: idx_izin_active_expiring; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_active_expiring ON perlengkapan.izin_pemakaian_bmn USING btree (tanggal_selesai, pegawai_satker_id) WHERE ((status)::text = 'ACTIVE'::text);


--
-- Name: idx_izin_aktivitas_created; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_aktivitas_created ON perlengkapan.izin_pemakaian_bmn_aktivitas USING btree (created_at DESC);


--
-- Name: idx_izin_aktivitas_izin; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_aktivitas_izin ON perlengkapan.izin_pemakaian_bmn_aktivitas USING btree (izin_pemakaian_id);


--
-- Name: idx_izin_aktivitas_recent; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_aktivitas_recent ON perlengkapan.izin_pemakaian_bmn_aktivitas USING btree (created_at DESC);


--
-- Name: idx_izin_aktivitas_user; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_aktivitas_user ON perlengkapan.izin_pemakaian_bmn_aktivitas USING btree (user_id);


--
-- Name: idx_izin_document_id; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_document_id ON perlengkapan.izin_pemakaian_bmn USING btree (document_id);


--
-- Name: idx_izin_file_pendukung_gin; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_file_pendukung_gin ON perlengkapan.izin_pemakaian_bmn USING gin (file_pendukung);


--
-- Name: idx_izin_jenis; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_jenis ON perlengkapan.izin_pemakaian_bmn USING btree (jenis_bmn);


--
-- Name: idx_izin_jenis_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_jenis_status ON perlengkapan.izin_pemakaian_bmn USING btree (jenis_bmn, status);


--
-- Name: idx_izin_nama_barang_trgm; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_nama_barang_trgm ON perlengkapan.izin_pemakaian_bmn USING gin (bmn_nama_barang public.gin_trgm_ops);


--
-- Name: idx_izin_no_polisi; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_no_polisi ON perlengkapan.izin_pemakaian_bmn USING btree (no_polisi) WHERE (no_polisi IS NOT NULL);


--
-- Name: idx_izin_nomor; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_nomor ON perlengkapan.izin_pemakaian_bmn USING btree (nomor_izin);


--
-- Name: idx_izin_nup; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_nup ON perlengkapan.izin_pemakaian_bmn USING btree (bmn_nup);


--
-- Name: idx_izin_nup_covering; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_nup_covering ON perlengkapan.izin_pemakaian_bmn USING btree (bmn_nup) INCLUDE (nomor_izin, jenis_bmn, status, pegawai_nip, tanggal_selesai);


--
-- Name: idx_izin_nup_jenis; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_nup_jenis ON perlengkapan.izin_pemakaian_bmn USING btree (bmn_nup, jenis_bmn);


--
-- Name: idx_izin_nup_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_nup_status ON perlengkapan.izin_pemakaian_bmn USING btree (bmn_nup, status);


--
-- Name: idx_izin_pegawai_covering; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_pegawai_covering ON perlengkapan.izin_pemakaian_bmn USING btree (pegawai_nip, status) INCLUDE (nomor_izin, jenis_bmn, bmn_nama_barang, tanggal_mulai, tanggal_selesai);


--
-- Name: idx_izin_pegawai_jenis_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_pegawai_jenis_status ON perlengkapan.izin_pemakaian_bmn USING btree (pegawai_nip, jenis_bmn, status);


--
-- Name: idx_izin_pegawai_nama_trgm; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_pegawai_nama_trgm ON perlengkapan.izin_pemakaian_bmn USING gin (pegawai_nama public.gin_trgm_ops);


--
-- Name: idx_izin_pegawai_nip; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_pegawai_nip ON perlengkapan.izin_pemakaian_bmn USING btree (pegawai_nip);


--
-- Name: idx_izin_pegawai_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_pegawai_status ON perlengkapan.izin_pemakaian_bmn USING btree (pegawai_nip, status);


--
-- Name: idx_izin_pemakaian_approver_satker; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_pemakaian_approver_satker ON perlengkapan.izin_pemakaian_bmn USING btree (approver_satker_id) WHERE (approver_satker_id IS NOT NULL);


--
-- Name: idx_izin_pemakaian_status_satker; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_pemakaian_status_satker ON perlengkapan.izin_pemakaian_bmn USING btree (status_kode, pegawai_satker_id);


--
-- Name: idx_izin_pemakaian_validator_satker; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_pemakaian_validator_satker ON perlengkapan.izin_pemakaian_bmn USING btree (validator_satker_id) WHERE (validator_satker_id IS NOT NULL);


--
-- Name: idx_izin_satker; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_satker ON perlengkapan.izin_pemakaian_bmn USING btree (pegawai_satker_id);


--
-- Name: idx_izin_satker_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_satker_status ON perlengkapan.izin_pemakaian_bmn USING btree (pegawai_satker_id, status);


--
-- Name: idx_izin_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_status ON perlengkapan.izin_pemakaian_bmn USING btree (status);


--
-- Name: idx_izin_status_tanggal_selesai; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_status_tanggal_selesai ON perlengkapan.izin_pemakaian_bmn USING btree (status, tanggal_selesai) WHERE ((status)::text = ANY ((ARRAY['ACTIVE'::character varying, 'APPROVED'::character varying])::text[]));


--
-- Name: idx_izin_tanggal_mulai; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_tanggal_mulai ON perlengkapan.izin_pemakaian_bmn USING btree (tanggal_mulai DESC);


--
-- Name: idx_izin_tanggal_selesai; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_tanggal_selesai ON perlengkapan.izin_pemakaian_bmn USING btree (tanggal_selesai) WHERE ((status)::text = 'ACTIVE'::text);


--
-- Name: idx_izin_tanggal_selesai_asc; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_izin_tanggal_selesai_asc ON perlengkapan.izin_pemakaian_bmn USING btree (tanggal_selesai) WHERE ((status)::text = 'ACTIVE'::text);


--
-- Name: idx_mapping_kode_baru; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_mapping_kode_baru ON perlengkapan.mapping_kodefikasi USING btree (kode_barang_baru);


--
-- Name: idx_mapping_kode_lama; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_mapping_kode_lama ON perlengkapan.mapping_kodefikasi USING btree (kode_barang_lama);


--
-- Name: idx_mapping_nama_baru_fts_indonesian; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_mapping_nama_baru_fts_indonesian ON perlengkapan.mapping_kodefikasi USING gin (to_tsvector('indonesian'::regconfig, (COALESCE(nama_barang_baru, ''::character varying))::text));


--
-- Name: idx_mapping_nama_baru_trgm; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_mapping_nama_baru_trgm ON perlengkapan.mapping_kodefikasi USING gin (nama_barang_baru public.gin_trgm_ops);


--
-- Name: idx_mapping_nama_lama_fts_indonesian; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_mapping_nama_lama_fts_indonesian ON perlengkapan.mapping_kodefikasi USING gin (to_tsvector('indonesian'::regconfig, (nama_barang_lama)::text));


--
-- Name: idx_mapping_nama_lama_trgm; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_mapping_nama_lama_trgm ON perlengkapan.mapping_kodefikasi USING gin (nama_barang_lama public.gin_trgm_ops);


--
-- Name: idx_mapping_pending; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_mapping_pending ON perlengkapan.mapping_kodefikasi USING btree (satker_id, created_at DESC) WHERE ((status_mapping)::text = ANY ((ARRAY['PROPOSED'::character varying, 'VERIFIED'::character varying])::text[]));


--
-- Name: idx_mapping_satker; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_mapping_satker ON perlengkapan.mapping_kodefikasi USING btree (satker_id);


--
-- Name: idx_mapping_satker_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_mapping_satker_status ON perlengkapan.mapping_kodefikasi USING btree (satker_id, status_mapping);


--
-- Name: idx_mapping_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_mapping_status ON perlengkapan.mapping_kodefikasi USING btree (status_mapping);


--
-- Name: idx_ms_jenis_pakaian_dinas_active; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ms_jenis_pakaian_dinas_active ON perlengkapan.ms_jenis_pakaian_dinas USING btree (is_active);


--
-- Name: idx_ms_jenis_pakaian_dinas_nama; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ms_jenis_pakaian_dinas_nama ON perlengkapan.ms_jenis_pakaian_dinas USING btree (nama);


--
-- Name: idx_ms_spesifikasi_foto_spec; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ms_spesifikasi_foto_spec ON perlengkapan.ms_spesifikasi_pakaian_dinas_foto USING btree (spesifikasi_id);


--
-- Name: idx_ms_spesifikasi_gender; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ms_spesifikasi_gender ON perlengkapan.ms_spesifikasi_pakaian_dinas USING btree (gender);


--
-- Name: idx_ms_spesifikasi_jenis; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ms_spesifikasi_jenis ON perlengkapan.ms_spesifikasi_pakaian_dinas USING btree (jenis_pakaian_dinas_id);


--
-- Name: idx_ms_spesifikasi_ukuran_group; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ms_spesifikasi_ukuran_group ON perlengkapan.ms_spesifikasi_pakaian_dinas USING btree (ukuran_group);


--
-- Name: idx_ms_subspesifikasi_spec; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ms_subspesifikasi_spec ON perlengkapan.ms_subspesifikasi_pakaian_dinas USING btree (spesifikasi_id);


--
-- Name: idx_pakaian_dinas_aktivitas_created; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pakaian_dinas_aktivitas_created ON perlengkapan.pengajuan_pakaian_dinas_aktivitas USING btree (created_at);


--
-- Name: idx_pakaian_dinas_aktivitas_pengajuan; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pakaian_dinas_aktivitas_pengajuan ON perlengkapan.pengajuan_pakaian_dinas_aktivitas USING btree (pengajuan_id);


--
-- Name: idx_pakaian_dinas_aktivitas_user; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pakaian_dinas_aktivitas_user ON perlengkapan.pengajuan_pakaian_dinas_aktivitas USING btree (user_id);


--
-- Name: idx_parallel_entity; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_parallel_entity ON perlengkapan.parallel_approvals USING btree (entity_type, entity_id);


--
-- Name: idx_parallel_entity_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_parallel_entity_status ON perlengkapan.parallel_approvals USING btree (entity_type, entity_id, status);


--
-- Name: idx_parallel_expires; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_parallel_expires ON perlengkapan.parallel_approvals USING btree (expires_at) WHERE ((status)::text = 'PENDING'::text);


--
-- Name: idx_parallel_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_parallel_status ON perlengkapan.parallel_approvals USING btree (status);


--
-- Name: idx_parallel_status_expires; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_parallel_status_expires ON perlengkapan.parallel_approvals USING btree (status, expires_at) WHERE ((status)::text = 'PENDING'::text);


--
-- Name: idx_parallel_votes_approval; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_parallel_votes_approval ON perlengkapan.parallel_approval_votes USING btree (parallel_approval_id);


--
-- Name: idx_parallel_votes_approver; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_parallel_votes_approver ON perlengkapan.parallel_approval_votes USING btree (approver_user_id);


--
-- Name: idx_parallel_votes_vote; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_parallel_votes_vote ON perlengkapan.parallel_approval_votes USING btree (vote);


--
-- Name: idx_pegawai_pakaian_dinas_eselon; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pegawai_pakaian_dinas_eselon ON perlengkapan.pegawai_pakaian_dinas USING btree (eselon) WHERE (eselon IS NOT NULL);


--
-- Name: idx_pegawai_pakaian_dinas_jenis_pegawai; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pegawai_pakaian_dinas_jenis_pegawai ON perlengkapan.pegawai_pakaian_dinas USING btree (jenis_pegawai) WHERE (jenis_pegawai IS NOT NULL);


--
-- Name: idx_pegawai_pakaian_dinas_kode_satker; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pegawai_pakaian_dinas_kode_satker ON perlengkapan.pegawai_pakaian_dinas USING btree (kode_satker) WHERE (kode_satker IS NOT NULL);


--
-- Name: idx_pegawai_pakaian_dinas_nip; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pegawai_pakaian_dinas_nip ON perlengkapan.pegawai_pakaian_dinas USING btree (nip);


--
-- Name: idx_pemakaian_bmn_items_izin_id; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pemakaian_bmn_items_izin_id ON perlengkapan.pemakaian_bmn_items USING btree (izin_pemakaian_id);


--
-- Name: idx_pemakaian_bmn_items_nup; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pemakaian_bmn_items_nup ON perlengkapan.pemakaian_bmn_items USING btree (bmn_nup);


--
-- Name: idx_pengajuan_bmn_referensi_pengajuan; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pengajuan_bmn_referensi_pengajuan ON perlengkapan.pengajuan_bmn_referensi_diizinkan USING btree (pengajuan_id);


--
-- Name: idx_pengajuan_pakaian_dinas_created_by; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pengajuan_pakaian_dinas_created_by ON perlengkapan.pengajuan_pakaian_dinas USING btree (created_by);


--
-- Name: idx_pengajuan_pakaian_dinas_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pengajuan_pakaian_dinas_status ON perlengkapan.pengajuan_pakaian_dinas USING btree (status_kode);


--
-- Name: idx_pengajuan_pakaian_dinas_tahun; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pengajuan_pakaian_dinas_tahun ON perlengkapan.pengajuan_pakaian_dinas USING btree (tahun);


--
-- Name: idx_penghapusan_bmn_aktivitas_aktivitas; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_penghapusan_bmn_aktivitas_aktivitas ON perlengkapan.penghapusan_bmn_aktivitas USING btree (aktivitas_id);


--
-- Name: idx_penghapusan_bmn_aktivitas_created_at; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_penghapusan_bmn_aktivitas_created_at ON perlengkapan.penghapusan_bmn_aktivitas USING btree (created_at DESC);


--
-- Name: idx_penghapusan_bmn_aktivitas_penghapusan; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_penghapusan_bmn_aktivitas_penghapusan ON perlengkapan.penghapusan_bmn_aktivitas USING btree (penghapusan_id);


--
-- Name: idx_penghapusan_bmn_aktivitas_user; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_penghapusan_bmn_aktivitas_user ON perlengkapan.penghapusan_bmn_aktivitas USING btree (user_id);


--
-- Name: idx_penghapusan_bmn_asset; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_penghapusan_bmn_asset ON perlengkapan.penghapusan_bmn USING btree (asset_id);


--
-- Name: idx_penghapusan_bmn_created_at; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_penghapusan_bmn_created_at ON perlengkapan.penghapusan_bmn USING btree (created_at DESC);


--
-- Name: idx_penghapusan_bmn_item_penghapusan_id; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_penghapusan_bmn_item_penghapusan_id ON perlengkapan.penghapusan_bmn_item USING btree (penghapusan_id);


--
-- Name: idx_penghapusan_bmn_lampiran_penghapusan_id; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_penghapusan_bmn_lampiran_penghapusan_id ON perlengkapan.penghapusan_bmn_lampiran USING btree (penghapusan_id);


--
-- Name: idx_penghapusan_bmn_satker; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_penghapusan_bmn_satker ON perlengkapan.penghapusan_bmn USING btree (satker_id);


--
-- Name: idx_penghapusan_bmn_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_penghapusan_bmn_status ON perlengkapan.penghapusan_bmn USING btree (status);


--
-- Name: idx_penghapusan_bmn_tahun; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_penghapusan_bmn_tahun ON perlengkapan.penghapusan_bmn USING btree (EXTRACT(year FROM tanggal_penghapusan));


--
-- Name: idx_pkb_active_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_active_status ON perlengkapan.pengajuan_kebutuhan_bmn USING btree (status_kode, created_at DESC) WHERE (status_kode <> ALL (ARRAY[2006, 2007, 2008, 2009]));


--
-- Name: idx_pkb_aktivitas_created; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_aktivitas_created ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas USING btree (created_at DESC);


--
-- Name: idx_pkb_aktivitas_recent; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_aktivitas_recent ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas USING btree (created_at DESC);


--
-- Name: idx_pkb_aktivitas_satker; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_aktivitas_satker ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas USING btree (pengajuan_satker_id);


--
-- Name: idx_pkb_aktivitas_satker_fk; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_aktivitas_satker_fk ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas USING btree (pengajuan_satker_id);


--
-- Name: idx_pkb_aktivitas_user; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_aktivitas_user ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas USING btree (user_id);


--
-- Name: idx_pkb_asset_kode; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_asset_kode ON perlengkapan.pengajuan_kebutuhan_bmn_asset USING btree (kode_barang);


--
-- Name: idx_pkb_asset_pengajuan; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_asset_pengajuan ON perlengkapan.pengajuan_kebutuhan_bmn_asset USING btree (pengajuan_id);


--
-- Name: idx_pkb_asset_pengajuan_fk; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_asset_pengajuan_fk ON perlengkapan.pengajuan_kebutuhan_bmn_asset USING btree (pengajuan_id);


--
-- Name: idx_pkb_barang_combined_fts_indonesian; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_barang_combined_fts_indonesian ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING gin (to_tsvector('indonesian'::regconfig, (((nama)::text || ' '::text) || COALESCE(keterangan, ''::text))));


--
-- Name: idx_pkb_barang_covering; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_barang_covering ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING btree (kode_barang) INCLUDE (nama, jumlah, jml_setuju, prioritas, skor);


--
-- Name: idx_pkb_barang_file_pendukung_gin; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_barang_file_pendukung_gin ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING gin (file_pendukung);


--
-- Name: idx_pkb_barang_high_priority; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_barang_high_priority ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING btree (prioritas DESC, skor DESC) WHERE (prioritas >= 7);


--
-- Name: idx_pkb_barang_jumlah; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_barang_jumlah ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING btree (jumlah DESC);


--
-- Name: idx_pkb_barang_keterangan_fts_indonesian; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_barang_keterangan_fts_indonesian ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING gin (to_tsvector('indonesian'::regconfig, COALESCE(keterangan, ''::text)));


--
-- Name: idx_pkb_barang_keterangan_trgm; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_barang_keterangan_trgm ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING gin (keterangan public.gin_trgm_ops);


--
-- Name: idx_pkb_barang_kode; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_barang_kode ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING btree (kode_barang);


--
-- Name: idx_pkb_barang_kode_jumlah; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_barang_kode_jumlah ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING btree (kode_barang, jumlah);


--
-- Name: idx_pkb_barang_nama_fts_indonesian; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_barang_nama_fts_indonesian ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING gin (to_tsvector('indonesian'::regconfig, (nama)::text));


--
-- Name: idx_pkb_barang_nama_trgm; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_barang_nama_trgm ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING gin (nama public.gin_trgm_ops);


--
-- Name: idx_pkb_barang_prioritas; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_barang_prioritas ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING btree (prioritas);


--
-- Name: idx_pkb_barang_prioritas_skor; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_barang_prioritas_skor ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING btree (prioritas DESC, skor DESC);


--
-- Name: idx_pkb_barang_satker; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_barang_satker ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING btree (pengajuan_satker_id);


--
-- Name: idx_pkb_barang_satker_fk; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_barang_satker_fk ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING btree (pengajuan_satker_id);


--
-- Name: idx_pkb_barang_skor; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_barang_skor ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING btree (skor DESC);


--
-- Name: idx_pkb_combined_fts_indonesian; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_combined_fts_indonesian ON perlengkapan.pengajuan_kebutuhan_bmn USING gin (to_tsvector('indonesian'::regconfig, (((nama)::text || ' '::text) || COALESCE(deskripsi, ''::text))));


--
-- Name: idx_pkb_created_at; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_created_at ON perlengkapan.pengajuan_kebutuhan_bmn USING btree (created_at DESC);


--
-- Name: idx_pkb_created_at_desc; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_created_at_desc ON perlengkapan.pengajuan_kebutuhan_bmn USING btree (created_at DESC);


--
-- Name: idx_pkb_created_by; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_created_by ON perlengkapan.pengajuan_kebutuhan_bmn USING btree (created_by);


--
-- Name: idx_pkb_deskripsi_fts_indonesian; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_deskripsi_fts_indonesian ON perlengkapan.pengajuan_kebutuhan_bmn USING gin (to_tsvector('indonesian'::regconfig, COALESCE(deskripsi, ''::text)));


--
-- Name: idx_pkb_deskripsi_trgm; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_deskripsi_trgm ON perlengkapan.pengajuan_kebutuhan_bmn USING gin (deskripsi public.gin_trgm_ops);


--
-- Name: idx_pkb_id_jenis_asset_gin; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_id_jenis_asset_gin ON perlengkapan.pengajuan_kebutuhan_bmn USING gin (id_jenis_asset);


--
-- Name: idx_pkb_nama_fts_indonesian; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_nama_fts_indonesian ON perlengkapan.pengajuan_kebutuhan_bmn USING gin (to_tsvector('indonesian'::regconfig, (nama)::text));


--
-- Name: idx_pkb_nama_trgm; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_nama_trgm ON perlengkapan.pengajuan_kebutuhan_bmn USING gin (nama public.gin_trgm_ops);


--
-- Name: idx_pkb_satker_covering; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_satker_covering ON perlengkapan.pengajuan_kebutuhan_bmn_satker USING btree (satker_id, status_kode) INCLUDE (satker_nama, prioritas, created_at);


--
-- Name: idx_pkb_satker_id; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_satker_id ON perlengkapan.pengajuan_kebutuhan_bmn_satker USING btree (satker_id);


--
-- Name: idx_pkb_satker_id_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_satker_id_status ON perlengkapan.pengajuan_kebutuhan_bmn_satker USING btree (satker_id, status_kode);


--
-- Name: idx_pkb_satker_nama_trgm; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_satker_nama_trgm ON perlengkapan.pengajuan_kebutuhan_bmn_satker USING gin (satker_nama public.gin_trgm_ops);


--
-- Name: idx_pkb_satker_pengajuan; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_satker_pengajuan ON perlengkapan.pengajuan_kebutuhan_bmn_satker USING btree (pengajuan_id);


--
-- Name: idx_pkb_satker_pengajuan_fk; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_satker_pengajuan_fk ON perlengkapan.pengajuan_kebutuhan_bmn_satker USING btree (pengajuan_id);


--
-- Name: idx_pkb_satker_pengajuan_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_satker_pengajuan_status ON perlengkapan.pengajuan_kebutuhan_bmn_satker USING btree (pengajuan_id, status_kode);


--
-- Name: idx_pkb_satker_prioritas; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_satker_prioritas ON perlengkapan.pengajuan_kebutuhan_bmn_satker USING btree (prioritas);


--
-- Name: idx_pkb_satker_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_satker_status ON perlengkapan.pengajuan_kebutuhan_bmn_satker USING btree (status_kode);


--
-- Name: idx_pkb_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_status ON perlengkapan.pengajuan_kebutuhan_bmn USING btree (status_kode);


--
-- Name: idx_pkb_status_created; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_status_created ON perlengkapan.pengajuan_kebutuhan_bmn USING btree (status_kode, created_at DESC);


--
-- Name: idx_pkb_tahun; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_tahun ON perlengkapan.pengajuan_kebutuhan_bmn USING btree (tahun);


--
-- Name: idx_pkb_tahun_created; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_tahun_created ON perlengkapan.pengajuan_kebutuhan_bmn USING btree (tahun, created_at DESC);


--
-- Name: idx_pkb_tahun_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_tahun_status ON perlengkapan.pengajuan_kebutuhan_bmn USING btree (tahun, status_kode);


--
-- Name: idx_pkb_tahun_status_covering; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_tahun_status_covering ON perlengkapan.pengajuan_kebutuhan_bmn USING btree (tahun, status_kode) INCLUDE (nama, created_at, updated_at);


--
-- Name: idx_pkb_updated_at_desc; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_pkb_updated_at_desc ON perlengkapan.pengajuan_kebutuhan_bmn USING btree (updated_at DESC);


--
-- Name: idx_ppd_active_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_active_status ON perlengkapan.pengajuan_pakaian_dinas USING btree (status_kode, created_at DESC) WHERE (status_kode <> ALL (ARRAY[2006, 2007, 2008, 2009]));


--
-- Name: idx_ppd_aktivitas_recent; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_aktivitas_recent ON perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas USING btree (created_at DESC);


--
-- Name: idx_ppd_aktivitas_satker; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_aktivitas_satker ON perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas USING btree (pengajuan_satker_id);


--
-- Name: idx_ppd_aktivitas_satker_fk; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_aktivitas_satker_fk ON perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas USING btree (pengajuan_satker_id);


--
-- Name: idx_ppd_created_at_desc; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_created_at_desc ON perlengkapan.pengajuan_pakaian_dinas USING btree (created_at DESC);


--
-- Name: idx_ppd_foto_spec_fk; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_foto_spec_fk ON perlengkapan.ms_spesifikasi_pakaian_dinas_foto USING btree (spesifikasi_id);


--
-- Name: idx_ppd_jenis_nama_fts_indonesian; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_jenis_nama_fts_indonesian ON perlengkapan.ms_jenis_pakaian_dinas USING gin (to_tsvector('indonesian'::regconfig, (nama)::text));


--
-- Name: idx_ppd_jenis_nama_trgm; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_jenis_nama_trgm ON perlengkapan.ms_jenis_pakaian_dinas USING gin (nama public.gin_trgm_ops);


--
-- Name: idx_ppd_nama_fts_indonesian; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_nama_fts_indonesian ON perlengkapan.pengajuan_pakaian_dinas USING gin (to_tsvector('indonesian'::regconfig, (nama)::text));


--
-- Name: idx_ppd_nama_trgm; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_nama_trgm ON perlengkapan.pengajuan_pakaian_dinas USING gin (nama public.gin_trgm_ops);


--
-- Name: idx_ppd_pakaian_pengajuan; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_pakaian_pengajuan ON perlengkapan.pengajuan_pakaian_dinas_pakaian USING btree (pengajuan_id);


--
-- Name: idx_ppd_pakaian_pengajuan_fk; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_pakaian_pengajuan_fk ON perlengkapan.pengajuan_pakaian_dinas_pakaian USING btree (pengajuan_id);


--
-- Name: idx_ppd_pegawai_covering; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_pegawai_covering ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai USING btree (nip) INCLUDE (nama, jenis_kelamin, jabatan);


--
-- Name: idx_ppd_pegawai_nama_trgm; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_pegawai_nama_trgm ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai USING gin (nama public.gin_trgm_ops);


--
-- Name: idx_ppd_pegawai_nip_jk; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_pegawai_nip_jk ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai USING btree (nip, jenis_kelamin);


--
-- Name: idx_ppd_pegawai_satker_fk; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_pegawai_satker_fk ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai USING btree (pengajuan_satker_id);


--
-- Name: idx_ppd_satker_id_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_satker_id_status ON perlengkapan.pengajuan_pakaian_dinas_satker USING btree (satker_id, status_kode);


--
-- Name: idx_ppd_satker_pegawai_nip; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_satker_pegawai_nip ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai USING btree (nip);


--
-- Name: idx_ppd_satker_pegawai_satker; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_satker_pegawai_satker ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai USING btree (pengajuan_satker_id);


--
-- Name: idx_ppd_satker_pengajuan; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_satker_pengajuan ON perlengkapan.pengajuan_pakaian_dinas_satker USING btree (pengajuan_id);


--
-- Name: idx_ppd_satker_pengajuan_fk; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_satker_pengajuan_fk ON perlengkapan.pengajuan_pakaian_dinas_satker USING btree (pengajuan_id);


--
-- Name: idx_ppd_satker_pengajuan_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_satker_pengajuan_status ON perlengkapan.pengajuan_pakaian_dinas_satker USING btree (pengajuan_id, status_kode);


--
-- Name: idx_ppd_satker_satker; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_satker_satker ON perlengkapan.pengajuan_pakaian_dinas_satker USING btree (satker_id);


--
-- Name: idx_ppd_satker_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_satker_status ON perlengkapan.pengajuan_pakaian_dinas_satker USING btree (status_kode);


--
-- Name: idx_ppd_satker_terpilih_pengajuan; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_satker_terpilih_pengajuan ON perlengkapan.pengajuan_pakaian_dinas_satker_terpilih USING btree (pengajuan_id);


--
-- Name: idx_ppd_satker_terpilih_pengajuan_fk; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_satker_terpilih_pengajuan_fk ON perlengkapan.pengajuan_pakaian_dinas_satker_terpilih USING btree (pengajuan_id);


--
-- Name: idx_ppd_spesifikasi_jenis_fk; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_spesifikasi_jenis_fk ON perlengkapan.ms_spesifikasi_pakaian_dinas USING btree (jenis_pakaian_dinas_id);


--
-- Name: idx_ppd_spesifikasi_nama_fts_indonesian; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_spesifikasi_nama_fts_indonesian ON perlengkapan.ms_spesifikasi_pakaian_dinas USING gin (to_tsvector('indonesian'::regconfig, (nama)::text));


--
-- Name: idx_ppd_spesifikasi_nama_trgm; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_spesifikasi_nama_trgm ON perlengkapan.ms_spesifikasi_pakaian_dinas USING gin (nama public.gin_trgm_ops);


--
-- Name: idx_ppd_subspesifikasi_spec_fk; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_subspesifikasi_spec_fk ON perlengkapan.ms_subspesifikasi_pakaian_dinas USING btree (spesifikasi_id);


--
-- Name: idx_ppd_tahun_covering; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_tahun_covering ON perlengkapan.pengajuan_pakaian_dinas USING btree (tahun, status_kode) INCLUDE (nama, created_at);


--
-- Name: idx_ppd_tahun_created; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_tahun_created ON perlengkapan.pengajuan_pakaian_dinas USING btree (tahun, created_at DESC);


--
-- Name: idx_ppd_tahun_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_tahun_status ON perlengkapan.pengajuan_pakaian_dinas USING btree (tahun, status_kode);


--
-- Name: idx_ppd_ukuran_pakaian_fk; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_ukuran_pakaian_fk ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran USING btree (pakaian_id);


--
-- Name: idx_ppd_ukuran_pegawai_fk; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_ukuran_pegawai_fk ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran USING btree (pegawai_id);


--
-- Name: idx_ppd_ukuran_satker; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_ukuran_satker ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran USING btree (pengajuan_satker_id);


--
-- Name: idx_ppd_ukuran_satker_fk; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_ppd_ukuran_satker_fk ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran USING btree (pengajuan_satker_id);


--
-- Name: idx_riwayat_file_dokumen_gin; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_riwayat_file_dokumen_gin ON perlengkapan.riwayat_pemenuhan USING gin (file_dokumen);


--
-- Name: idx_riwayat_kebutuhan; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_riwayat_kebutuhan ON perlengkapan.riwayat_pemenuhan USING btree (kebutuhan_bmn_id);


--
-- Name: idx_riwayat_kode_barang; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_riwayat_kode_barang ON perlengkapan.riwayat_pemenuhan USING btree (kode_barang);


--
-- Name: idx_riwayat_nilai_perolehan; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_riwayat_nilai_perolehan ON perlengkapan.riwayat_pemenuhan USING btree (nilai_perolehan DESC) WHERE (nilai_perolehan IS NOT NULL);


--
-- Name: idx_riwayat_roadmap; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_riwayat_roadmap ON perlengkapan.riwayat_pemenuhan USING btree (roadmap_id);


--
-- Name: idx_riwayat_satker; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_riwayat_satker ON perlengkapan.riwayat_pemenuhan USING btree (satker_id);


--
-- Name: idx_riwayat_satker_tahun; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_riwayat_satker_tahun ON perlengkapan.riwayat_pemenuhan USING btree (satker_id, tahun_anggaran);


--
-- Name: idx_riwayat_satker_tahun_kode; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_riwayat_satker_tahun_kode ON perlengkapan.riwayat_pemenuhan USING btree (satker_id, tahun_anggaran, kode_barang);


--
-- Name: idx_riwayat_sumber; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_riwayat_sumber ON perlengkapan.riwayat_pemenuhan USING btree (sumber_data);


--
-- Name: idx_riwayat_tahun; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_riwayat_tahun ON perlengkapan.riwayat_pemenuhan USING btree (tahun_anggaran);


--
-- Name: idx_riwayat_tahun_sumber; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_riwayat_tahun_sumber ON perlengkapan.riwayat_pemenuhan USING btree (tahun_anggaran, sumber_data);


--
-- Name: idx_riwayat_tanggal; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_riwayat_tanggal ON perlengkapan.riwayat_pemenuhan USING btree (tanggal_pemenuhan DESC);


--
-- Name: idx_riwayat_tanggal_pemenuhan_desc; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_riwayat_tanggal_pemenuhan_desc ON perlengkapan.riwayat_pemenuhan USING btree (tanggal_pemenuhan DESC);


--
-- Name: idx_riwayat_tanggal_sumber; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_riwayat_tanggal_sumber ON perlengkapan.riwayat_pemenuhan USING btree (tanggal_pemenuhan DESC, sumber_data);


--
-- Name: idx_roadmap_active; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_roadmap_active ON perlengkapan.roadmap_sarpras USING btree (satker_id, tahun_rencana) WHERE ((status_pemenuhan)::text = ANY ((ARRAY['PLANNED'::character varying, 'IN_PROGRESS'::character varying])::text[]));


--
-- Name: idx_roadmap_created_at_desc; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_roadmap_created_at_desc ON perlengkapan.roadmap_sarpras USING btree (created_at DESC);


--
-- Name: idx_roadmap_high_priority; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_roadmap_high_priority ON perlengkapan.roadmap_sarpras USING btree (prioritas DESC, tahun_rencana) WHERE (prioritas >= 7);


--
-- Name: idx_roadmap_jumlah_kebutuhan; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_roadmap_jumlah_kebutuhan ON perlengkapan.roadmap_sarpras USING btree (jumlah_kebutuhan DESC);


--
-- Name: idx_roadmap_kode_barang; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_roadmap_kode_barang ON perlengkapan.roadmap_sarpras USING btree (kode_barang);


--
-- Name: idx_roadmap_kode_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_roadmap_kode_status ON perlengkapan.roadmap_sarpras USING btree (kode_barang, status_pemenuhan);


--
-- Name: idx_roadmap_nama_barang_fts_indonesian; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_roadmap_nama_barang_fts_indonesian ON perlengkapan.roadmap_sarpras USING gin (to_tsvector('indonesian'::regconfig, (nama_barang)::text));


--
-- Name: idx_roadmap_nama_barang_trgm; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_roadmap_nama_barang_trgm ON perlengkapan.roadmap_sarpras USING gin (nama_barang public.gin_trgm_ops);


--
-- Name: idx_roadmap_periode; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_roadmap_periode ON perlengkapan.roadmap_sarpras USING btree (periode_mulai, periode_akhir);


--
-- Name: idx_roadmap_periode_tahun; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_roadmap_periode_tahun ON perlengkapan.roadmap_sarpras USING btree (periode_mulai, periode_akhir, tahun_rencana);


--
-- Name: idx_roadmap_satker; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_roadmap_satker ON perlengkapan.roadmap_sarpras USING btree (satker_id);


--
-- Name: idx_roadmap_satker_tahun; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_roadmap_satker_tahun ON perlengkapan.roadmap_sarpras USING btree (satker_id, tahun_rencana);


--
-- Name: idx_roadmap_satker_tahun_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_roadmap_satker_tahun_status ON perlengkapan.roadmap_sarpras USING btree (satker_id, tahun_rencana, status_pemenuhan);


--
-- Name: idx_roadmap_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_roadmap_status ON perlengkapan.roadmap_sarpras USING btree (status_pemenuhan);


--
-- Name: idx_roadmap_tahun; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_roadmap_tahun ON perlengkapan.roadmap_sarpras USING btree (tahun_rencana);


--
-- Name: idx_workflow_def_active; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_workflow_def_active ON perlengkapan.workflow_definitions USING btree (is_active) WHERE (is_active = true);


--
-- Name: idx_workflow_def_config; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_workflow_def_config ON perlengkapan.workflow_definitions USING gin (config);


--
-- Name: idx_workflow_def_name; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_workflow_def_name ON perlengkapan.workflow_definitions USING btree (name);


--
-- Name: idx_workflow_inst_entity; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_workflow_inst_entity ON perlengkapan.workflow_instances USING btree (entity_type, entity_id);


--
-- Name: idx_workflow_inst_metadata; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_workflow_inst_metadata ON perlengkapan.workflow_instances USING gin (metadata);


--
-- Name: idx_workflow_inst_sla; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_workflow_inst_sla ON perlengkapan.workflow_instances USING btree (sla_deadline) WHERE (((status)::text = 'ACTIVE'::text) AND (is_sla_breached = false));


--
-- Name: idx_workflow_inst_state; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_workflow_inst_state ON perlengkapan.workflow_instances USING btree (current_state);


--
-- Name: idx_workflow_inst_status; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_workflow_inst_status ON perlengkapan.workflow_instances USING btree (status);


--
-- Name: idx_workflow_inst_workflow; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_workflow_inst_workflow ON perlengkapan.workflow_instances USING btree (workflow_definition_id);


--
-- Name: idx_workflow_trans_actor; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_workflow_trans_actor ON perlengkapan.workflow_transitions USING btree (actor_user_id);


--
-- Name: idx_workflow_trans_delegation; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_workflow_trans_delegation ON perlengkapan.workflow_transitions USING btree (delegation_id) WHERE (delegation_id IS NOT NULL);


--
-- Name: idx_workflow_trans_instance; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_workflow_trans_instance ON perlengkapan.workflow_transitions USING btree (workflow_instance_id);


--
-- Name: idx_workflow_trans_metadata; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_workflow_trans_metadata ON perlengkapan.workflow_transitions USING gin (metadata);


--
-- Name: idx_workflow_trans_time; Type: INDEX; Schema: perlengkapan; Owner: -
--

CREATE INDEX idx_workflow_trans_time ON perlengkapan.workflow_transitions USING btree (transitioned_at DESC);


--
-- Name: izin_pemakaian_bmn trg_izin_pemakaian_updated_at; Type: TRIGGER; Schema: perlengkapan; Owner: -
--

CREATE TRIGGER trg_izin_pemakaian_updated_at BEFORE UPDATE ON perlengkapan.izin_pemakaian_bmn FOR EACH ROW EXECUTE FUNCTION perlengkapan.update_entity_updated_at();


--
-- Name: mapping_kodefikasi trg_mapping_updated_at; Type: TRIGGER; Schema: perlengkapan; Owner: -
--

CREATE TRIGGER trg_mapping_updated_at BEFORE UPDATE ON perlengkapan.mapping_kodefikasi FOR EACH ROW EXECUTE FUNCTION perlengkapan.update_entity_updated_at();


--
-- Name: parallel_approvals trg_parallel_approvals_updated_at; Type: TRIGGER; Schema: perlengkapan; Owner: -
--

CREATE TRIGGER trg_parallel_approvals_updated_at BEFORE UPDATE ON perlengkapan.parallel_approvals FOR EACH ROW EXECUTE FUNCTION perlengkapan.update_entity_updated_at();


--
-- Name: pengajuan_kebutuhan_bmn_satker_barang trg_pkb_barang_updated_at; Type: TRIGGER; Schema: perlengkapan; Owner: -
--

CREATE TRIGGER trg_pkb_barang_updated_at BEFORE UPDATE ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang FOR EACH ROW EXECUTE FUNCTION perlengkapan.update_kebutuhan_bmn_updated_at();


--
-- Name: pengajuan_kebutuhan_bmn_satker trg_pkb_satker_updated_at; Type: TRIGGER; Schema: perlengkapan; Owner: -
--

CREATE TRIGGER trg_pkb_satker_updated_at BEFORE UPDATE ON perlengkapan.pengajuan_kebutuhan_bmn_satker FOR EACH ROW EXECUTE FUNCTION perlengkapan.update_kebutuhan_bmn_updated_at();


--
-- Name: pengajuan_kebutuhan_bmn trg_pkb_updated_at; Type: TRIGGER; Schema: perlengkapan; Owner: -
--

CREATE TRIGGER trg_pkb_updated_at BEFORE UPDATE ON perlengkapan.pengajuan_kebutuhan_bmn FOR EACH ROW EXECUTE FUNCTION perlengkapan.update_kebutuhan_bmn_updated_at();


--
-- Name: riwayat_pemenuhan trg_riwayat_updated_at; Type: TRIGGER; Schema: perlengkapan; Owner: -
--

CREATE TRIGGER trg_riwayat_updated_at BEFORE UPDATE ON perlengkapan.riwayat_pemenuhan FOR EACH ROW EXECUTE FUNCTION perlengkapan.update_entity_updated_at();


--
-- Name: roadmap_sarpras trg_roadmap_updated_at; Type: TRIGGER; Schema: perlengkapan; Owner: -
--

CREATE TRIGGER trg_roadmap_updated_at BEFORE UPDATE ON perlengkapan.roadmap_sarpras FOR EACH ROW EXECUTE FUNCTION perlengkapan.update_entity_updated_at();


--
-- Name: ms_jenis_pakaian_dinas trigger_ms_jenis_pakaian_dinas_updated_at; Type: TRIGGER; Schema: perlengkapan; Owner: -
--

CREATE TRIGGER trigger_ms_jenis_pakaian_dinas_updated_at BEFORE UPDATE ON perlengkapan.ms_jenis_pakaian_dinas FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: ms_spesifikasi_pakaian_dinas trigger_ms_spesifikasi_pakaian_dinas_updated_at; Type: TRIGGER; Schema: perlengkapan; Owner: -
--

CREATE TRIGGER trigger_ms_spesifikasi_pakaian_dinas_updated_at BEFORE UPDATE ON perlengkapan.ms_spesifikasi_pakaian_dinas FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: ms_subspesifikasi_pakaian_dinas trigger_ms_subspesifikasi_pakaian_dinas_updated_at; Type: TRIGGER; Schema: perlengkapan; Owner: -
--

CREATE TRIGGER trigger_ms_subspesifikasi_pakaian_dinas_updated_at BEFORE UPDATE ON perlengkapan.ms_subspesifikasi_pakaian_dinas FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: pegawai_pakaian_dinas trigger_pegawai_pakaian_dinas_updated_at; Type: TRIGGER; Schema: perlengkapan; Owner: -
--

CREATE TRIGGER trigger_pegawai_pakaian_dinas_updated_at BEFORE UPDATE ON perlengkapan.pegawai_pakaian_dinas FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: pengajuan_pakaian_dinas_satker trigger_pengajuan_pakaian_dinas_satker_updated_at; Type: TRIGGER; Schema: perlengkapan; Owner: -
--

CREATE TRIGGER trigger_pengajuan_pakaian_dinas_satker_updated_at BEFORE UPDATE ON perlengkapan.pengajuan_pakaian_dinas_satker FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: pengajuan_pakaian_dinas trigger_pengajuan_pakaian_dinas_updated_at; Type: TRIGGER; Schema: perlengkapan; Owner: -
--

CREATE TRIGGER trigger_pengajuan_pakaian_dinas_updated_at BEFORE UPDATE ON perlengkapan.pengajuan_pakaian_dinas FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: archive_documents archive_documents_collection_id_fkey; Type: FK CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.archive_documents
    ADD CONSTRAINT archive_documents_collection_id_fkey FOREIGN KEY (collection_id) REFERENCES dokumen.archive_collections(id) ON DELETE CASCADE;


--
-- Name: archive_documents archive_documents_document_id_fkey; Type: FK CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.archive_documents
    ADD CONSTRAINT archive_documents_document_id_fkey FOREIGN KEY (document_id) REFERENCES dokumen.documents(id) ON DELETE CASCADE;


--
-- Name: audit_log audit_log_document_id_fkey; Type: FK CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.audit_log
    ADD CONSTRAINT audit_log_document_id_fkey FOREIGN KEY (document_id) REFERENCES dokumen.documents(id) ON DELETE SET NULL;


--
-- Name: document_permissions document_permissions_document_id_fkey; Type: FK CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.document_permissions
    ADD CONSTRAINT document_permissions_document_id_fkey FOREIGN KEY (document_id) REFERENCES dokumen.documents(id) ON DELETE CASCADE;


--
-- Name: document_tags document_tags_document_id_fkey; Type: FK CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.document_tags
    ADD CONSTRAINT document_tags_document_id_fkey FOREIGN KEY (document_id) REFERENCES dokumen.documents(id) ON DELETE CASCADE;


--
-- Name: document_templates document_templates_parent_template_id_fkey; Type: FK CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.document_templates
    ADD CONSTRAINT document_templates_parent_template_id_fkey FOREIGN KEY (parent_template_id) REFERENCES dokumen.document_templates(id);


--
-- Name: document_versions document_versions_document_id_fkey; Type: FK CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.document_versions
    ADD CONSTRAINT document_versions_document_id_fkey FOREIGN KEY (document_id) REFERENCES dokumen.documents(id) ON DELETE CASCADE;


--
-- Name: generated_documents generated_documents_template_id_fkey; Type: FK CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.generated_documents
    ADD CONSTRAINT generated_documents_template_id_fkey FOREIGN KEY (template_id) REFERENCES dokumen.document_templates(id);


--
-- Name: ocr_results ocr_results_document_id_fkey; Type: FK CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.ocr_results
    ADD CONSTRAINT ocr_results_document_id_fkey FOREIGN KEY (document_id) REFERENCES dokumen.documents(id) ON DELETE CASCADE;


--
-- Name: template_versions template_versions_template_id_fkey; Type: FK CONSTRAINT; Schema: dokumen; Owner: -
--

ALTER TABLE ONLY dokumen.template_versions
    ADD CONSTRAINT template_versions_template_id_fkey FOREIGN KEY (template_id) REFERENCES dokumen.document_templates(id);


--
-- Name: delivery_status delivery_status_notification_id_fkey; Type: FK CONSTRAINT; Schema: notifikasi; Owner: -
--

ALTER TABLE ONLY notifikasi.delivery_status
    ADD CONSTRAINT delivery_status_notification_id_fkey FOREIGN KEY (notification_id) REFERENCES notifikasi.in_app_notifications(id) ON DELETE CASCADE;


--
-- Name: notification_queue notification_queue_notification_id_fkey; Type: FK CONSTRAINT; Schema: notifikasi; Owner: -
--

ALTER TABLE ONLY notifikasi.notification_queue
    ADD CONSTRAINT notification_queue_notification_id_fkey FOREIGN KEY (notification_id) REFERENCES notifikasi.in_app_notifications(id) ON DELETE CASCADE;


--
-- Name: batch_operation_log batch_operation_log_user_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.batch_operation_log
    ADD CONSTRAINT batch_operation_log_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id);


--
-- Name: pengajuan_pakaian_dinas_satker_aktivitas fk_ppd_aktivitas_status_kode; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas
    ADD CONSTRAINT fk_ppd_aktivitas_status_kode FOREIGN KEY (status_kode) REFERENCES perlengkapan.ms_aktivitas_bmn(kode);


--
-- Name: pengajuan_pakaian_dinas_satker fk_ppd_satker_status_kode; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_satker
    ADD CONSTRAINT fk_ppd_satker_status_kode FOREIGN KEY (status_kode) REFERENCES perlengkapan.ms_aktivitas_bmn(kode);


--
-- Name: pengajuan_pakaian_dinas fk_ppd_status_kode; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas
    ADD CONSTRAINT fk_ppd_status_kode FOREIGN KEY (status_kode) REFERENCES perlengkapan.ms_aktivitas_bmn(kode);


--
-- Name: izin_pemakaian_bmn_aktivitas izin_pemakaian_bmn_aktivitas_izin_pemakaian_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.izin_pemakaian_bmn_aktivitas
    ADD CONSTRAINT izin_pemakaian_bmn_aktivitas_izin_pemakaian_id_fkey FOREIGN KEY (izin_pemakaian_id) REFERENCES perlengkapan.izin_pemakaian_bmn(id) ON DELETE CASCADE;


--
-- Name: izin_pemakaian_bmn_aktivitas izin_pemakaian_bmn_aktivitas_to_status_kode_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.izin_pemakaian_bmn_aktivitas
    ADD CONSTRAINT izin_pemakaian_bmn_aktivitas_to_status_kode_fkey FOREIGN KEY (to_status_kode) REFERENCES perlengkapan.ms_aktivitas_bmn(kode);


--
-- Name: izin_pemakaian_bmn izin_pemakaian_bmn_previous_permit_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.izin_pemakaian_bmn
    ADD CONSTRAINT izin_pemakaian_bmn_previous_permit_id_fkey FOREIGN KEY (previous_permit_id) REFERENCES perlengkapan.izin_pemakaian_bmn(id) ON DELETE SET NULL;


--
-- Name: izin_pemakaian_bmn izin_pemakaian_bmn_status_kode_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.izin_pemakaian_bmn
    ADD CONSTRAINT izin_pemakaian_bmn_status_kode_fkey FOREIGN KEY (status_kode) REFERENCES perlengkapan.ms_aktivitas_bmn(kode);


--
-- Name: ms_spesifikasi_pakaian_dinas_foto ms_spesifikasi_pakaian_dinas_foto_spesifikasi_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.ms_spesifikasi_pakaian_dinas_foto
    ADD CONSTRAINT ms_spesifikasi_pakaian_dinas_foto_spesifikasi_id_fkey FOREIGN KEY (spesifikasi_id) REFERENCES perlengkapan.ms_spesifikasi_pakaian_dinas(id) ON DELETE CASCADE;


--
-- Name: ms_spesifikasi_pakaian_dinas ms_spesifikasi_pakaian_dinas_jenis_pakaian_dinas_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.ms_spesifikasi_pakaian_dinas
    ADD CONSTRAINT ms_spesifikasi_pakaian_dinas_jenis_pakaian_dinas_id_fkey FOREIGN KEY (jenis_pakaian_dinas_id) REFERENCES perlengkapan.ms_jenis_pakaian_dinas(id) ON DELETE CASCADE;


--
-- Name: ms_subspesifikasi_pakaian_dinas ms_subspesifikasi_pakaian_dinas_spesifikasi_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.ms_subspesifikasi_pakaian_dinas
    ADD CONSTRAINT ms_subspesifikasi_pakaian_dinas_spesifikasi_id_fkey FOREIGN KEY (spesifikasi_id) REFERENCES perlengkapan.ms_spesifikasi_pakaian_dinas(id) ON DELETE CASCADE;


--
-- Name: parallel_approval_votes parallel_approval_votes_parallel_approval_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.parallel_approval_votes
    ADD CONSTRAINT parallel_approval_votes_parallel_approval_id_fkey FOREIGN KEY (parallel_approval_id) REFERENCES perlengkapan.parallel_approvals(id) ON DELETE CASCADE;


--
-- Name: pemakaian_bmn_items pemakaian_bmn_items_izin_pemakaian_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pemakaian_bmn_items
    ADD CONSTRAINT pemakaian_bmn_items_izin_pemakaian_id_fkey FOREIGN KEY (izin_pemakaian_id) REFERENCES perlengkapan.izin_pemakaian_bmn(id) ON DELETE CASCADE;


--
-- Name: pengajuan_bmn_referensi_diizinkan pengajuan_bmn_referensi_diizinkan_pengajuan_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_bmn_referensi_diizinkan
    ADD CONSTRAINT pengajuan_bmn_referensi_diizinkan_pengajuan_id_fkey FOREIGN KEY (pengajuan_id) REFERENCES perlengkapan.pengajuan_kebutuhan_bmn(id) ON DELETE CASCADE;


--
-- Name: pengajuan_kebutuhan_bmn_asset pengajuan_kebutuhan_bmn_asset_pengajuan_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_kebutuhan_bmn_asset
    ADD CONSTRAINT pengajuan_kebutuhan_bmn_asset_pengajuan_id_fkey FOREIGN KEY (pengajuan_id) REFERENCES perlengkapan.pengajuan_kebutuhan_bmn(id) ON DELETE CASCADE;


--
-- Name: pengajuan_kebutuhan_bmn_satker_aktivitas pengajuan_kebutuhan_bmn_satker_aktivit_pengajuan_satker_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
    ADD CONSTRAINT pengajuan_kebutuhan_bmn_satker_aktivit_pengajuan_satker_id_fkey FOREIGN KEY (pengajuan_satker_id) REFERENCES perlengkapan.pengajuan_kebutuhan_bmn_satker(id) ON DELETE CASCADE;


--
-- Name: pengajuan_kebutuhan_bmn_satker_aktivitas pengajuan_kebutuhan_bmn_satker_aktivitas_to_status_kode_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
    ADD CONSTRAINT pengajuan_kebutuhan_bmn_satker_aktivitas_to_status_kode_fkey FOREIGN KEY (to_status_kode) REFERENCES perlengkapan.ms_aktivitas_bmn(kode);


--
-- Name: pengajuan_kebutuhan_bmn_satker_barang pengajuan_kebutuhan_bmn_satker_barang_pengajuan_satker_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
    ADD CONSTRAINT pengajuan_kebutuhan_bmn_satker_barang_pengajuan_satker_id_fkey FOREIGN KEY (pengajuan_satker_id) REFERENCES perlengkapan.pengajuan_kebutuhan_bmn_satker(id) ON DELETE CASCADE;


--
-- Name: pengajuan_kebutuhan_bmn_satker pengajuan_kebutuhan_bmn_satker_pengajuan_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_kebutuhan_bmn_satker
    ADD CONSTRAINT pengajuan_kebutuhan_bmn_satker_pengajuan_id_fkey FOREIGN KEY (pengajuan_id) REFERENCES perlengkapan.pengajuan_kebutuhan_bmn(id) ON DELETE CASCADE;


--
-- Name: pengajuan_kebutuhan_bmn_satker pengajuan_kebutuhan_bmn_satker_status_kode_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_kebutuhan_bmn_satker
    ADD CONSTRAINT pengajuan_kebutuhan_bmn_satker_status_kode_fkey FOREIGN KEY (status_kode) REFERENCES perlengkapan.ms_aktivitas_bmn(kode);


--
-- Name: pengajuan_kebutuhan_bmn pengajuan_kebutuhan_bmn_status_kode_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_kebutuhan_bmn
    ADD CONSTRAINT pengajuan_kebutuhan_bmn_status_kode_fkey FOREIGN KEY (status_kode) REFERENCES perlengkapan.ms_aktivitas_bmn(kode);


--
-- Name: pengajuan_pakaian_dinas_aktivitas pengajuan_pakaian_dinas_aktivitas_aktivitas_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_aktivitas
    ADD CONSTRAINT pengajuan_pakaian_dinas_aktivitas_aktivitas_id_fkey FOREIGN KEY (aktivitas_id) REFERENCES perlengkapan.ms_aktivitas_bmn(id);


--
-- Name: pengajuan_pakaian_dinas_aktivitas pengajuan_pakaian_dinas_aktivitas_pengajuan_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_aktivitas
    ADD CONSTRAINT pengajuan_pakaian_dinas_aktivitas_pengajuan_id_fkey FOREIGN KEY (pengajuan_id) REFERENCES perlengkapan.pengajuan_pakaian_dinas(id) ON DELETE CASCADE;


--
-- Name: pengajuan_pakaian_dinas pengajuan_pakaian_dinas_jenis_pakaian_dinas_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas
    ADD CONSTRAINT pengajuan_pakaian_dinas_jenis_pakaian_dinas_id_fkey FOREIGN KEY (jenis_pakaian_dinas_id) REFERENCES perlengkapan.ms_jenis_pakaian_dinas(id);


--
-- Name: pengajuan_pakaian_dinas_pakaian pengajuan_pakaian_dinas_pakaian_pengajuan_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_pakaian
    ADD CONSTRAINT pengajuan_pakaian_dinas_pakaian_pengajuan_id_fkey FOREIGN KEY (pengajuan_id) REFERENCES perlengkapan.pengajuan_pakaian_dinas(id) ON DELETE CASCADE;


--
-- Name: pengajuan_pakaian_dinas_satker_aktivitas pengajuan_pakaian_dinas_satker_aktivit_pengajuan_satker_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas
    ADD CONSTRAINT pengajuan_pakaian_dinas_satker_aktivit_pengajuan_satker_id_fkey FOREIGN KEY (pengajuan_satker_id) REFERENCES perlengkapan.pengajuan_pakaian_dinas_satker(id) ON DELETE CASCADE;


--
-- Name: pengajuan_pakaian_dinas_satker_pegawai_ukuran pengajuan_pakaian_dinas_satker_pegawa_pengajuan_satker_id_fkey1; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran
    ADD CONSTRAINT pengajuan_pakaian_dinas_satker_pegawa_pengajuan_satker_id_fkey1 FOREIGN KEY (pengajuan_satker_id) REFERENCES perlengkapan.pengajuan_pakaian_dinas_satker(id) ON DELETE CASCADE;


--
-- Name: pengajuan_pakaian_dinas_satker_pegawai pengajuan_pakaian_dinas_satker_pegawai_pengajuan_satker_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_satker_pegawai
    ADD CONSTRAINT pengajuan_pakaian_dinas_satker_pegawai_pengajuan_satker_id_fkey FOREIGN KEY (pengajuan_satker_id) REFERENCES perlengkapan.pengajuan_pakaian_dinas_satker(id) ON DELETE CASCADE;


--
-- Name: pengajuan_pakaian_dinas_satker_pegawai_ukuran pengajuan_pakaian_dinas_satker_pegawai_ukuran_pakaian_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran
    ADD CONSTRAINT pengajuan_pakaian_dinas_satker_pegawai_ukuran_pakaian_id_fkey FOREIGN KEY (pakaian_id) REFERENCES perlengkapan.pengajuan_pakaian_dinas_pakaian(id) ON DELETE CASCADE;


--
-- Name: pengajuan_pakaian_dinas_satker_pegawai_ukuran pengajuan_pakaian_dinas_satker_pegawai_ukuran_pegawai_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran
    ADD CONSTRAINT pengajuan_pakaian_dinas_satker_pegawai_ukuran_pegawai_id_fkey FOREIGN KEY (pegawai_id) REFERENCES perlengkapan.pengajuan_pakaian_dinas_satker_pegawai(id) ON DELETE CASCADE;


--
-- Name: pengajuan_pakaian_dinas_satker pengajuan_pakaian_dinas_satker_pengajuan_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_satker
    ADD CONSTRAINT pengajuan_pakaian_dinas_satker_pengajuan_id_fkey FOREIGN KEY (pengajuan_id) REFERENCES perlengkapan.pengajuan_pakaian_dinas(id) ON DELETE CASCADE;


--
-- Name: pengajuan_pakaian_dinas_satker_terpilih pengajuan_pakaian_dinas_satker_terpilih_pengajuan_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.pengajuan_pakaian_dinas_satker_terpilih
    ADD CONSTRAINT pengajuan_pakaian_dinas_satker_terpilih_pengajuan_id_fkey FOREIGN KEY (pengajuan_id) REFERENCES perlengkapan.pengajuan_pakaian_dinas(id) ON DELETE CASCADE;


--
-- Name: penghapusan_bmn_aktivitas penghapusan_bmn_aktivitas_aktivitas_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.penghapusan_bmn_aktivitas
    ADD CONSTRAINT penghapusan_bmn_aktivitas_aktivitas_id_fkey FOREIGN KEY (aktivitas_id) REFERENCES perlengkapan.ms_aktivitas_bmn(id);


--
-- Name: penghapusan_bmn_aktivitas penghapusan_bmn_aktivitas_penghapusan_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.penghapusan_bmn_aktivitas
    ADD CONSTRAINT penghapusan_bmn_aktivitas_penghapusan_id_fkey FOREIGN KEY (penghapusan_id) REFERENCES perlengkapan.penghapusan_bmn(id) ON DELETE CASCADE;


--
-- Name: penghapusan_bmn_item penghapusan_bmn_item_penghapusan_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.penghapusan_bmn_item
    ADD CONSTRAINT penghapusan_bmn_item_penghapusan_id_fkey FOREIGN KEY (penghapusan_id) REFERENCES perlengkapan.penghapusan_bmn(id) ON DELETE CASCADE;


--
-- Name: penghapusan_bmn_lampiran penghapusan_bmn_lampiran_penghapusan_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.penghapusan_bmn_lampiran
    ADD CONSTRAINT penghapusan_bmn_lampiran_penghapusan_id_fkey FOREIGN KEY (penghapusan_id) REFERENCES perlengkapan.penghapusan_bmn(id) ON DELETE CASCADE;


--
-- Name: riwayat_pemenuhan riwayat_pemenuhan_kebutuhan_bmn_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.riwayat_pemenuhan
    ADD CONSTRAINT riwayat_pemenuhan_kebutuhan_bmn_id_fkey FOREIGN KEY (kebutuhan_bmn_id) REFERENCES perlengkapan.pengajuan_kebutuhan_bmn(id) ON DELETE SET NULL;


--
-- Name: riwayat_pemenuhan riwayat_pemenuhan_roadmap_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.riwayat_pemenuhan
    ADD CONSTRAINT riwayat_pemenuhan_roadmap_id_fkey FOREIGN KEY (roadmap_id) REFERENCES perlengkapan.roadmap_sarpras(id) ON DELETE SET NULL;


--
-- Name: workflow_escalations workflow_escalations_workflow_instance_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.workflow_escalations
    ADD CONSTRAINT workflow_escalations_workflow_instance_id_fkey FOREIGN KEY (workflow_instance_id) REFERENCES perlengkapan.workflow_instances(id) ON DELETE CASCADE;


--
-- Name: workflow_instances workflow_instances_workflow_definition_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.workflow_instances
    ADD CONSTRAINT workflow_instances_workflow_definition_id_fkey FOREIGN KEY (workflow_definition_id) REFERENCES perlengkapan.workflow_definitions(id);


--
-- Name: workflow_transitions workflow_transitions_workflow_instance_id_fkey; Type: FK CONSTRAINT; Schema: perlengkapan; Owner: -
--

ALTER TABLE ONLY perlengkapan.workflow_transitions
    ADD CONSTRAINT workflow_transitions_workflow_instance_id_fkey FOREIGN KEY (workflow_instance_id) REFERENCES perlengkapan.workflow_instances(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

