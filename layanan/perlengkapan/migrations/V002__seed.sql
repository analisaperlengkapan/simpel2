-- V002__seed.sql — perlengkapan seed/master data (squashed F5-B 2026-06-09)
-- Lookup/master rows from legacy V018 (role_access), V019 (statuses),
-- V023 (templates), V034/V035 (aktivitas) etc. Data-only dump of fresh-apply.

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
-- Data for Name: archive_collections; Type: TABLE DATA; Schema: dokumen; Owner: -
--



--
-- Data for Name: documents; Type: TABLE DATA; Schema: dokumen; Owner: -
--



--
-- Data for Name: archive_documents; Type: TABLE DATA; Schema: dokumen; Owner: -
--



--
-- Data for Name: audit_log; Type: TABLE DATA; Schema: dokumen; Owner: -
--



--
-- Data for Name: document_permissions; Type: TABLE DATA; Schema: dokumen; Owner: -
--



--
-- Data for Name: document_tags; Type: TABLE DATA; Schema: dokumen; Owner: -
--



--
-- Data for Name: document_templates; Type: TABLE DATA; Schema: dokumen; Owner: -
--

INSERT INTO dokumen.document_templates (id, name, description, template_type, content, format, output_format, version, is_active, parent_template_id, variables, sample_data, letterhead_config, created_by, created_at, updated_by, updated_at) VALUES ('00000000-0000-0000-0000-000000000001', 'SK Kebutuhan BMN', 'Surat Keputusan untuk persetujuan kebutuhan Barang Milik Negara', 'sk_kebutuhan_bmn', '<html>
<head>
    <meta charset="UTF-8">
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { text-align: center; margin-bottom: 30px; }
        .title { font-size: 18px; font-weight: bold; text-decoration: underline; }
        .content { margin-top: 20px; line-height: 1.6; }
        .signature { margin-top: 50px; text-align: right; }
    </style>
</head>
<body>
    <div class="header">
        <h2>KEJAKSAAN REPUBLIK INDONESIA</h2>
        <h3>{{satker_nama}}</h3>
    </div>

    <div class="content">
        <p class="title">SURAT KEPUTUSAN</p>
        <p class="title">TENTANG PERSETUJUAN KEBUTUHAN BARANG MILIK NEGARA</p>
        <p class="title">TAHUN ANGGARAN {{tahun_anggaran}}</p>

        <p>Menimbang:</p>
        <ol type="a">
            <li>Bahwa dalam rangka pelaksanaan tugas dan fungsi {{satker_nama}}, diperlukan pengadaan Barang Milik Negara;</li>
            <li>Bahwa berdasarkan pertimbangan sebagaimana dimaksud dalam huruf a, perlu menetapkan Surat Keputusan tentang Persetujuan Kebutuhan Barang Milik Negara.</li>
        </ol>

        <p>Mengingat:</p>
        <ol>
            <li>Peraturan Pemerintah Nomor 27 Tahun 2014 tentang Pengelolaan Barang Milik Negara/Daerah;</li>
            <li>Peraturan Menteri Keuangan tentang Pedoman Pengelolaan Barang Milik Negara;</li>
        </ol>

        <p style="text-align: center; font-weight: bold;">MEMUTUSKAN:</p>

        <p>Menetapkan:</p>
        <p>PERTAMA: Menyetujui kebutuhan Barang Milik Negara sebagaimana tercantum dalam lampiran Surat Keputusan ini.</p>
        <p>KEDUA: Surat Keputusan ini mulai berlaku pada tanggal ditetapkan.</p>

        <p style="margin-top: 30px;">Ditetapkan di Jakarta</p>
        <p>pada tanggal {{approval_date}}</p>
    </div>

    <div class="signature">
        <p>Kepala {{satker_nama}}</p>
        <br><br><br>
        <p>_______________________</p>
    </div>
</body>
</html>', 'html', 'pdf', 1, true, NULL, '{"satker_nama": "string", "approval_date": "string", "document_type": "string", "tahun_anggaran": "integer"}', '{"satker_nama": "Kejaksaan Tinggi DKI Jakarta", "approval_date": "11 Februari 2026", "document_type": "SK Kebutuhan BMN", "tahun_anggaran": 2026}', '{"logo_url": "/assets/logo-kejaksaan.png", "header_text": "KEJAKSAAN REPUBLIK INDONESIA"}', '00000000-0000-0000-0000-000000000000', '2026-06-09 12:09:57.613898+00', NULL, '2026-06-09 12:09:57.613898+00');
INSERT INTO dokumen.document_templates (id, name, description, template_type, content, format, output_format, version, is_active, parent_template_id, variables, sample_data, letterhead_config, created_by, created_at, updated_by, updated_at) VALUES ('00000000-0000-0000-0000-000000000002', 'SK Penghapusan BMN', 'Surat Keputusan untuk persetujuan penghapusan Barang Milik Negara', 'sk_penghapusan_bmn', '<html>
<head>
    <meta charset="UTF-8">
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { text-align: center; margin-bottom: 30px; }
        .title { font-size: 18px; font-weight: bold; text-decoration: underline; }
        .content { margin-top: 20px; line-height: 1.6; }
        .signature { margin-top: 50px; text-align: right; }
    </style>
</head>
<body>
    <div class="header">
        <h2>KEJAKSAAN REPUBLIK INDONESIA</h2>
        <h3>{{satker_nama}}</h3>
    </div>

    <div class="content">
        <p class="title">SURAT KEPUTUSAN</p>
        <p class="title">TENTANG PERSETUJUAN PENGHAPUSAN BARANG MILIK NEGARA</p>

        <p>Menimbang:</p>
        <ol type="a">
            <li>Bahwa Barang Milik Negara yang tidak dapat digunakan lagi perlu dihapuskan dari daftar inventaris;</li>
            <li>Bahwa alasan penghapusan: {{alasan}};</li>
            <li>Bahwa berdasarkan pertimbangan sebagaimana dimaksud dalam huruf a dan b, perlu menetapkan Surat Keputusan tentang Persetujuan Penghapusan Barang Milik Negara.</li>
        </ol>

        <p>Mengingat:</p>
        <ol>
            <li>Peraturan Pemerintah Nomor 27 Tahun 2014 tentang Pengelolaan Barang Milik Negara/Daerah;</li>
            <li>Peraturan Menteri Keuangan tentang Pedoman Pengelolaan Barang Milik Negara;</li>
            <li>Peraturan Menteri Keuangan tentang Tata Cara Penghapusan Barang Milik Negara;</li>
        </ol>

        <p style="text-align: center; font-weight: bold;">MEMUTUSKAN:</p>

        <p>Menetapkan:</p>
        <p>PERTAMA: Menyetujui penghapusan Barang Milik Negara sebagaimana tercantum dalam lampiran Surat Keputusan ini.</p>
        <p>KEDUA: Penghapusan dilaksanakan sesuai dengan ketentuan peraturan perundang-undangan yang berlaku.</p>
        <p>KETIGA: Surat Keputusan ini mulai berlaku pada tanggal ditetapkan.</p>

        <p style="margin-top: 30px;">Ditetapkan di Jakarta</p>
        <p>pada tanggal {{approval_date}}</p>
    </div>

    <div class="signature">
        <p>Kepala {{satker_nama}}</p>
        <br><br><br>
        <p>_______________________</p>
    </div>
</body>
</html>', 'html', 'pdf', 1, true, NULL, '{"alasan": "string", "satker_nama": "string", "approval_date": "string", "document_type": "string"}', '{"alasan": "Rusak berat dan tidak ekonomis untuk diperbaiki", "satker_nama": "Kejaksaan Tinggi DKI Jakarta", "approval_date": "11 Februari 2026", "document_type": "SK Penghapusan BMN"}', '{"logo_url": "/assets/logo-kejaksaan.png", "header_text": "KEJAKSAAN REPUBLIK INDONESIA"}', '00000000-0000-0000-0000-000000000000', '2026-06-09 12:09:57.613898+00', NULL, '2026-06-09 12:09:57.613898+00');


--
-- Data for Name: document_versions; Type: TABLE DATA; Schema: dokumen; Owner: -
--



--
-- Data for Name: generated_documents; Type: TABLE DATA; Schema: dokumen; Owner: -
--



--
-- Data for Name: ocr_results; Type: TABLE DATA; Schema: dokumen; Owner: -
--



--
-- Data for Name: template_versions; Type: TABLE DATA; Schema: dokumen; Owner: -
--



--
-- Data for Name: in_app_notifications; Type: TABLE DATA; Schema: notifikasi; Owner: -
--



--
-- Data for Name: delivery_status; Type: TABLE DATA; Schema: notifikasi; Owner: -
--



--
-- Data for Name: email_templates; Type: TABLE DATA; Schema: notifikasi; Owner: -
--

INSERT INTO notifikasi.email_templates (id, name, subject, body_template, variables, enabled, created_at, updated_at) VALUES ('03f2d407-03f9-42fb-b019-0f801531dbca', 'workflow_submitted', 'Pengajuan Baru: {{entity_name}}', 'Yth. {{user_name}},

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
Kejaksaan Republik Indonesia', '{"deadline": "string", "user_name": "string", "action_url": "string", "entity_name": "string", "entity_type": "string", "requester_name": "string", "submission_date": "string"}', true, '2026-06-09 12:09:58.481547+00', '2026-06-09 12:09:58.481547+00');
INSERT INTO notifikasi.email_templates (id, name, subject, body_template, variables, enabled, created_at, updated_at) VALUES ('9fdc9ee8-9d80-4021-913f-76e63a9211dc', 'workflow_approved', 'Pengajuan Disetujui: {{entity_name}}', 'Yth. {{user_name}},

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
Kejaksaan Republik Indonesia', '{"user_name": "string", "entity_name": "string", "entity_type": "string", "document_url": "string", "approval_date": "string", "approver_name": "string"}', true, '2026-06-09 12:09:58.481547+00', '2026-06-09 12:09:58.481547+00');
INSERT INTO notifikasi.email_templates (id, name, subject, body_template, variables, enabled, created_at, updated_at) VALUES ('376ab712-3f37-4f42-8720-6cf8373c0cbe', 'workflow_rejected', 'Pengajuan Ditolak: {{entity_name}}', 'Yth. {{user_name}},

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
Kejaksaan Republik Indonesia', '{"user_name": "string", "entity_name": "string", "entity_type": "string", "rejector_name": "string", "rejection_date": "string", "rejection_reason": "string"}', true, '2026-06-09 12:09:58.481547+00', '2026-06-09 12:09:58.481547+00');
INSERT INTO notifikasi.email_templates (id, name, subject, body_template, variables, enabled, created_at, updated_at) VALUES ('40d83503-c666-40b4-9010-3d7387ed94aa', 'sla_breach', 'Peringatan: SLA Terlampaui - {{entity_name}}', 'Yth. {{user_name}},

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
Kejaksaan Republik Indonesia', '{"user_name": "string", "action_url": "string", "entity_name": "string", "entity_type": "string", "days_overdue": "number", "sla_deadline": "string", "current_state": "string"}', true, '2026-06-09 12:09:58.481547+00', '2026-06-09 12:09:58.481547+00');
INSERT INTO notifikasi.email_templates (id, name, subject, body_template, variables, enabled, created_at, updated_at) VALUES ('723dc452-4c82-46b5-ba48-2a359c3c96f0', 'permit_expiry_reminder', 'Pengingat: Izin Pemakaian BMN Akan Berakhir', 'Yth. {{user_name}},

Izin pemakaian BMN Anda akan berakhir dalam {{days_remaining}} hari:

Nomor Izin: {{permit_number}}
BMN: {{bmn_name}}
Tanggal Berakhir: {{expiry_date}}

Silakan perpanjang izin jika masih diperlukan:
{{action_url}}

Untuk pertanyaan, hubungi bagian perlengkapan satker Anda.

Terima kasih,
SIMPEL - Sistem Informasi Manajemen Perlengkapan
Kejaksaan Republik Indonesia', '{"bmn_name": "string", "user_name": "string", "action_url": "string", "expiry_date": "string", "permit_number": "string", "days_remaining": "number"}', true, '2026-06-09 12:09:58.481547+00', '2026-06-09 12:09:58.481547+00');


--
-- Data for Name: notification_queue; Type: TABLE DATA; Schema: notifikasi; Owner: -
--



--
-- Data for Name: sms_templates; Type: TABLE DATA; Schema: notifikasi; Owner: -
--

INSERT INTO notifikasi.sms_templates (id, name, body_template, variables, enabled, created_at, updated_at) VALUES ('ac6193b7-891d-4c84-a2f7-45a59e9792e2', 'workflow_submitted_sms', 'SIMPEL: Pengajuan baru {{entity_name}} memerlukan persetujuan Anda. Login ke sistem untuk meninjau.', '{"entity_name": "string"}', true, '2026-06-09 12:09:58.481547+00', '2026-06-09 12:09:58.481547+00');
INSERT INTO notifikasi.sms_templates (id, name, body_template, variables, enabled, created_at, updated_at) VALUES ('8401abff-626c-44fa-b8e7-a35e8e6cdb5d', 'workflow_approved_sms', 'SIMPEL: Pengajuan {{entity_name}} telah disetujui. Dokumen dapat diunduh di sistem.', '{"entity_name": "string"}', true, '2026-06-09 12:09:58.481547+00', '2026-06-09 12:09:58.481547+00');
INSERT INTO notifikasi.sms_templates (id, name, body_template, variables, enabled, created_at, updated_at) VALUES ('46cde086-e159-448e-a6ca-984c38757795', 'workflow_rejected_sms', 'SIMPEL: Pengajuan {{entity_name}} ditolak. Alasan: {{rejection_reason}}', '{"entity_name": "string", "rejection_reason": "string"}', true, '2026-06-09 12:09:58.481547+00', '2026-06-09 12:09:58.481547+00');
INSERT INTO notifikasi.sms_templates (id, name, body_template, variables, enabled, created_at, updated_at) VALUES ('2e6ff897-5736-4bb6-84cc-76dca13c081f', 'permit_expiry_reminder_sms', 'SIMPEL: Izin pemakaian BMN {{bmn_name}} ({{permit_number}}) akan berakhir dalam {{days_remaining}} hari. Perpanjang segera.', '{"bmn_name": "string", "permit_number": "string", "days_remaining": "number"}', true, '2026-06-09 12:09:58.481547+00', '2026-06-09 12:09:58.481547+00');


--
-- Data for Name: user_notification_preferences; Type: TABLE DATA; Schema: notifikasi; Owner: -
--



--
-- Data for Name: analisis_kebutuhan; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: audit_log; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: batch_operation_log; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: export_jobs; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: ms_aktivitas_bmn; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--

INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (1, 2000, 'DRAFT', 'Pengajuan baru dalam tahap penyusunan', 1, true, '2026-06-09 12:09:51.426871+00', '2026-06-09 12:09:51.426871+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (2, 2001, 'INPUT_BARANG', 'Pelaksana Satker menginput daftar kebutuhan barang', 2, true, '2026-06-09 12:09:51.426871+00', '2026-06-09 12:09:51.426871+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (3, 2002, 'SUBMIT_SATKER', 'Satker mengajukan ke Validator', 3, true, '2026-06-09 12:09:51.426871+00', '2026-06-09 12:09:51.426871+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (4, 2003, 'REVISI_SATKER', 'Dikembalikan ke Satker untuk revisi', 4, true, '2026-06-09 12:09:51.426871+00', '2026-06-09 12:09:51.426871+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (5, 2004, 'ANALISIS_KELAYAKAN', 'Validator Pusat melakukan analisis kelayakan', 5, true, '2026-06-09 12:09:51.426871+00', '2026-06-09 12:09:51.426871+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (6, 2005, 'PENYUSUNAN_PRIORITAS', 'Validator Pusat menyusun prioritas', 6, true, '2026-06-09 12:09:51.426871+00', '2026-06-09 12:09:51.426871+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (7, 2006, 'APPROVED', 'Pengajuan disetujui', 7, true, '2026-06-09 12:09:51.426871+00', '2026-06-09 12:09:51.426871+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (8, 2007, 'REJECTED', 'Pengajuan ditolak', 8, true, '2026-06-09 12:09:51.426871+00', '2026-06-09 12:09:51.426871+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (9, 2008, 'COMPLETED', 'Proses selesai', 9, true, '2026-06-09 12:09:51.426871+00', '2026-06-09 12:09:51.426871+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (10, 2009, 'CANCELLED', 'Pengajuan dibatalkan', 10, true, '2026-06-09 12:09:51.426871+00', '2026-06-09 12:09:51.426871+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (11, 3010, 'SUBMITTED_APPROVER_SATKER', 'Menunggu keputusan Approver Satker (setelah validasi internal)', 10, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (12, 3011, 'REVISI_OPERATOR', 'Dikembalikan ke Operator Satker untuk revisi (catatan wajib)', 11, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
-- Pemakaian BMN statuses (3000-3007). ms_aktivitas_bmn is the unified BMN
-- workflow-state master (izin_pemakaian_bmn.status_kode FKs to it); 3010/3011
-- were already present, the base statuses were missing.
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (13, 3000, 'DRAFT', 'Izin pemakaian baru dalam tahap penyusunan', 12, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (14, 3001, 'SUBMITTED', 'Diajukan ke Validator Satker', 13, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (15, 3002, 'APPROVED', 'Disetujui Approver Satker', 14, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (16, 3003, 'REJECTED', 'Ditolak', 15, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (17, 3004, 'ACTIVE', 'Izin pemakaian aktif', 16, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (18, 3005, 'EXPIRED', 'Izin pemakaian kadaluarsa', 17, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (19, 3006, 'REVOKED', 'Izin pemakaian dicabut', 18, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (20, 3007, 'CANCELLED', 'Izin pemakaian dibatalkan', 19, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
-- Penghapusan BMN statuses (4000-4011). penghapusan_bmn.status_kode is not
-- FK-constrained, but seeding keeps the state master complete for labels/joins.
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (21, 4000, 'DRAFT', 'Usulan penghapusan dalam tahap penyusunan', 20, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (22, 4001, 'SUBMIT_WILAYAH', 'Diajukan ke Validator Wilayah', 21, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (23, 4002, 'RETURNED_TO_OPERATOR', 'Dikembalikan ke Operator Satker', 22, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (24, 4003, 'SUBMIT_PUSAT', 'Diteruskan ke Validator Pusat', 23, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (25, 4004, 'VERIFIKASI_PUSAT', 'Diverifikasi Validator Pusat', 24, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (26, 4005, 'KONSEP_SK_GENERATED', 'Konsep SK telah digenerate', 25, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (27, 4006, 'SK_SIGNED', 'SK telah ditandatangani', 26, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (28, 4007, 'COMPLETED', 'Proses penghapusan selesai', 27, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (29, 4008, 'REJECTED', 'Usulan penghapusan ditolak', 28, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (30, 4010, 'KONSEP_SK_WILAYAH_GENERATED', 'Konsep SK Wilayah telah digenerate', 29, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (31, 4011, 'SK_SIGNED_WILAYAH', 'SK Wilayah telah ditandatangani', 30, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
-- Pakaian dinas workflow codes (1000-1012). FK target for
-- pengajuan_pakaian_dinas(.aktivitas_id), _satker(.aktivitas_id) and
-- _satker_aktivitas(.aktivitas_id). Mirrors AktivitasStatus
-- (pakaian_dinas/models/status.rs); kode/nama must stay in sync with the enum.
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (32, 1000, 'INPUT', 'Input', 31, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (33, 1001, 'SUBMIT_TO_VALIDATOR', 'Diajukan ke Validator', 32, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (34, 1003, 'REVISI_PELAKSANA', 'Revisi Pelaksana', 33, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (35, 1004, 'SUBMIT_TO_PUSAT', 'Diajukan ke Pusat', 34, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (36, 1005, 'REVISI_SATKER', 'Revisi Satker', 35, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (37, 1006, 'DITOLAK', 'Ditolak', 36, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (38, 1007, 'REVISI_WILAYAH', 'Revisi Wilayah', 37, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (39, 1008, 'SELESAI', 'Selesai', 38, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (40, 1009, 'START_KEJAGUNG', 'Mulai (Kejagung)', 39, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (41, 1010, 'SUBMIT_TO_PUSAT_FROM_WILAYAH', 'Diajukan ke Pusat dari Wilayah', 40, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (42, 1011, 'START_NON_KEJAGUNG', 'Mulai (Non-Kejagung)', 41, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');
INSERT INTO perlengkapan.ms_aktivitas_bmn (id, kode, nama, deskripsi, urutan, is_active, created_at, updated_at) VALUES (43, 1012, 'SUBMIT_TO_VALIDATOR_WILAYAH', 'Diajukan ke Validator Wilayah', 42, true, '2026-06-09 12:10:01.429501+00', '2026-06-09 12:10:01.429501+00');


--
-- Data for Name: izin_pemakaian_bmn; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: izin_pemakaian_bmn_aktivitas; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: mapping_kodefikasi; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: ms_jenis_pakaian_dinas; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--

INSERT INTO perlengkapan.ms_jenis_pakaian_dinas (id, nama, deskripsi, is_active, created_at, updated_at) VALUES ('726094b2-0c9f-44dc-97ca-9a061db433b3', 'PDH', 'Pakaian Dinas Harian', true, '2026-06-09 12:09:51.795601+00', '2026-06-09 12:09:51.795601+00');
INSERT INTO perlengkapan.ms_jenis_pakaian_dinas (id, nama, deskripsi, is_active, created_at, updated_at) VALUES ('912cd244-1071-4f43-b7dd-1376365b434b', 'PDL', 'Pakaian Dinas Lapangan', true, '2026-06-09 12:09:51.795601+00', '2026-06-09 12:09:51.795601+00');
INSERT INTO perlengkapan.ms_jenis_pakaian_dinas (id, nama, deskripsi, is_active, created_at, updated_at) VALUES ('93e4cc5c-ed24-4b59-9314-33624ced5312', 'Toga Jaksa', 'Toga untuk Jaksa', true, '2026-06-09 12:09:51.795601+00', '2026-06-09 12:09:51.795601+00');
INSERT INTO perlengkapan.ms_jenis_pakaian_dinas (id, nama, deskripsi, is_active, created_at, updated_at) VALUES ('f51bf0d2-d7d6-4a5d-93c4-8b43419409e1', 'Toga TU', 'Toga untuk Tenaga TU', true, '2026-06-09 12:09:51.795601+00', '2026-06-09 12:09:51.795601+00');
INSERT INTO perlengkapan.ms_jenis_pakaian_dinas (id, nama, deskripsi, is_active, created_at, updated_at) VALUES ('3d920e52-cd3b-4e20-bff9-3af21f7d90fd', 'Jas Almamater', 'Jas Almamater Kejaksaan', true, '2026-06-09 12:09:51.795601+00', '2026-06-09 12:09:51.795601+00');
INSERT INTO perlengkapan.ms_jenis_pakaian_dinas (id, nama, deskripsi, is_active, created_at, updated_at) VALUES ('c5f67a58-9779-42a9-8a2b-3f0d026a33db', 'Batik', 'Pakaian Batik Dinas', true, '2026-06-09 12:09:51.795601+00', '2026-06-09 12:09:51.795601+00');


--
-- Data for Name: ms_spesifikasi_pakaian_dinas; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: ms_spesifikasi_pakaian_dinas_foto; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: ms_subspesifikasi_pakaian_dinas; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: ms_ukuran; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--

INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('XS', 'BAJU', 1);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('S', 'BAJU', 2);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('M', 'BAJU', 3);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('L', 'BAJU', 4);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('XL', 'BAJU', 5);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('XXL', 'BAJU', 6);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('XXXL', 'BAJU', 7);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('4XL', 'BAJU', 8);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('27', 'CELANA', 1);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('28', 'CELANA', 2);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('29', 'CELANA', 3);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('30', 'CELANA', 4);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('31', 'CELANA', 5);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('32', 'CELANA', 6);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('33', 'CELANA', 7);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('34', 'CELANA', 8);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('35', 'CELANA', 9);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('36', 'CELANA', 10);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('37', 'CELANA', 11);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('38', 'CELANA', 12);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('39', 'CELANA', 13);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('40', 'CELANA', 14);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('42', 'CELANA', 15);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('44', 'CELANA', 16);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('36', 'SEPATU', 1);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('37', 'SEPATU', 2);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('38', 'SEPATU', 3);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('39', 'SEPATU', 4);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('40', 'SEPATU', 5);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('41', 'SEPATU', 6);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('42', 'SEPATU', 7);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('43', 'SEPATU', 8);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('44', 'SEPATU', 9);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('45', 'SEPATU', 10);
INSERT INTO perlengkapan.ms_ukuran (ukuran, "group", urutan) VALUES ('46', 'SEPATU', 11);


--
-- Data for Name: ms_workflow_status; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--

INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (2000, 'kebutuhan_bmn', 'Draft', 'Validator Pusat membuat pengajuan periode', 1, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (2001, 'kebutuhan_bmn', 'Input Barang', 'Operator Satker mengisi kebutuhan BMN', 2, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (2002, 'kebutuhan_bmn', 'Diajukan ke Validator Wilayah', 'Operator Satker mengirim ke Validator Wilayah', 3, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (2003, 'kebutuhan_bmn', 'Revisi Satker', 'Dikembalikan ke Operator Satker untuk perbaikan', 4, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (2004, 'kebutuhan_bmn', 'Diajukan ke Validator Pusat', 'Validator Wilayah mengirim ke Validator Pusat', 5, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (2005, 'kebutuhan_bmn', 'Analisis Kelayakan', 'Validator Pusat menganalisis kebutuhan', 6, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (2006, 'kebutuhan_bmn', 'Disetujui', 'Kebutuhan BMN disetujui Validator Pusat', 7, true, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (2007, 'kebutuhan_bmn', 'Ditolak', 'Kebutuhan BMN ditolak Validator Pusat', 8, true, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (2008, 'kebutuhan_bmn', 'Selesai', 'Proses kebutuhan BMN selesai', 9, true, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (2009, 'kebutuhan_bmn', 'Dibatalkan', 'Pengajuan dibatalkan', 10, true, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (3000, 'pemakaian_bmn', 'Draft', 'Operator Satker membuat konsep izin pemakaian', 1, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (3001, 'pemakaian_bmn', 'Diajukan', 'Draft diajukan untuk persetujuan', 2, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (3002, 'pemakaian_bmn', 'Disetujui', 'Izin pemakaian disetujui', 3, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (3003, 'pemakaian_bmn', 'Ditolak', 'Izin pemakaian ditolak', 4, true, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (3004, 'pemakaian_bmn', 'Aktif', 'Izin pemakaian aktif (surat sudah diupload)', 5, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (3005, 'pemakaian_bmn', 'Kadaluarsa', 'Izin pemakaian telah habis masa berlaku', 6, true, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (3006, 'pemakaian_bmn', 'Dicabut', 'Izin pemakaian dicabut', 7, true, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (3007, 'pemakaian_bmn', 'Dibatalkan', 'Pengajuan dibatalkan', 8, true, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (4000, 'penghapusan_bmn', 'Draft', 'Operator Satker membuat pengajuan SK Penghapusan', 1, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (4001, 'penghapusan_bmn', 'Diajukan ke Validator Wilayah', 'Operator Satker mengirim ke Validator Wilayah', 2, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (4002, 'penghapusan_bmn', 'Dikembalikan ke Operator', 'Validator Wilayah mengembalikan ke Operator untuk perbaikan', 3, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (4003, 'penghapusan_bmn', 'Diajukan ke Validator Pusat', 'Validator Wilayah mengirim ke Validator Pusat', 4, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (4004, 'penghapusan_bmn', 'Verifikasi Pusat', 'Validator Pusat memverifikasi pengajuan', 5, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (4005, 'penghapusan_bmn', 'Konsep SK Digenerate', 'Validator Pusat mengenerate konsep SK', 6, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (4006, 'penghapusan_bmn', 'SK Ditandatangani', 'SK sudah ditandatangani dan PDF diupload', 7, false, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (4007, 'penghapusan_bmn', 'Selesai', 'Proses SK Penghapusan selesai', 8, true, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (4008, 'penghapusan_bmn', 'Ditolak', 'Pengajuan SK Penghapusan ditolak', 9, true, '2026-06-09 12:09:55.981464+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (1007, 'pakaian_dinas', 'Revisi Wilayah', 'Dikembalikan ke Validator Wilayah oleh Validator Pusat untuk perbaikan', 7, false, '2026-06-09 12:09:56.587486+00');
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal, created_at) VALUES (2010, 'kebutuhan_bmn', 'Revisi Wilayah', 'Dikembalikan ke Validator Wilayah oleh Validator Pusat untuk perbaikan', 11, false, '2026-06-09 12:09:56.587486+00');


--
-- Data for Name: parallel_approvals; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: parallel_approval_votes; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: pegawai_pakaian_dinas; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: pemakaian_bmn_items; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: pengajuan_kebutuhan_bmn; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: pengajuan_bmn_referensi_diizinkan; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: pengajuan_kebutuhan_bmn_asset; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: pengajuan_kebutuhan_bmn_satker; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: pengajuan_kebutuhan_bmn_satker_aktivitas; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: pengajuan_kebutuhan_bmn_satker_barang; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: pengajuan_pakaian_dinas; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: pengajuan_pakaian_dinas_aktivitas; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: pengajuan_pakaian_dinas_pakaian; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: pengajuan_pakaian_dinas_satker; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: pengajuan_pakaian_dinas_satker_aktivitas; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: pengajuan_pakaian_dinas_satker_pegawai; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: pengajuan_pakaian_dinas_satker_pegawai_ukuran; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: pengajuan_pakaian_dinas_satker_terpilih; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: penghapusan_bmn; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: penghapusan_bmn_aktivitas; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: penghapusan_bmn_item; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: penghapusan_bmn_lampiran; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: roadmap_sarpras; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: riwayat_pemenuhan; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: workflow_definitions; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--

INSERT INTO perlengkapan.workflow_definitions (id, name, description, version, config, is_active, created_by, updated_by, created_at, updated_at) VALUES ('e5ae4fc3-b3b0-4782-b411-b5741eb7cc06', 'kebutuhan_bmn', 'Workflow for BMN requirements approval process', 1, '{"sla_minutes": {"APPROVED": 4320, "SUBMIT_SATKER": 2880, "ANALISIS_KELAYAKAN": 4320, "PENYUSUNAN_PRIORITAS": 1440}, "transitions": {"DRAFT": ["INPUT_BARANG", "CANCELLED"], "APPROVED": ["COMPLETED"], "REJECTED": [], "CANCELLED": [], "COMPLETED": ["CANCELLED"], "INPUT_BARANG": ["SUBMIT_SATKER", "DRAFT"], "REVISI_SATKER": ["INPUT_BARANG"], "SUBMIT_SATKER": ["ANALISIS_KELAYAKAN", "REVISI_SATKER", "REJECTED"], "ANALISIS_KELAYAKAN": ["PENYUSUNAN_PRIORITAS", "REVISI_SATKER", "REJECTED"], "PENYUSUNAN_PRIORITAS": ["APPROVED", "REJECTED"]}, "required_roles": {"APPROVED": "admin_pusat", "REJECTED": "admin_pusat", "INPUT_BARANG": "operator_satker", "REVISI_SATKER": "validator_pusat", "SUBMIT_SATKER": "operator_satker", "ANALISIS_KELAYAKAN": "validator_pusat", "PENYUSUNAN_PRIORITAS": "validator_pusat"}, "supports_parallel_approval": true}', true, NULL, NULL, '2026-06-09 12:09:52.656767+00', '2026-06-09 12:09:52.656767+00');
INSERT INTO perlengkapan.workflow_definitions (id, name, description, version, config, is_active, created_by, updated_by, created_at, updated_at) VALUES ('c08e7853-c2b8-4918-b85d-9fec3cbdf436', 'pemakaian_bmn', 'Workflow for BMN usage permit approval process', 1, '{"sla_minutes": {"APPROVED": 480, "SUBMITTED": 1440}, "transitions": {"DRAFT": ["SUBMITTED", "CANCELLED"], "ACTIVE": ["EXPIRED", "REVOKED"], "EXPIRED": [], "REVOKED": [], "APPROVED": ["ACTIVE"], "REJECTED": [], "CANCELLED": [], "SUBMITTED": ["APPROVED", "REJECTED"]}, "required_roles": {"REVOKED": "pimpinan_satker", "APPROVED": "pimpinan_satker", "REJECTED": "pimpinan_satker", "SUBMITTED": "pegawai"}, "supports_parallel_approval": false}', true, NULL, NULL, '2026-06-09 12:09:52.656767+00', '2026-06-09 12:09:52.656767+00');


--
-- Data for Name: workflow_delegations; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: workflow_instances; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: workflow_escalations; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Data for Name: workflow_transitions; Type: TABLE DATA; Schema: perlengkapan; Owner: -
--



--
-- Name: batch_operation_log_id_seq; Type: SEQUENCE SET; Schema: perlengkapan; Owner: -
--

SELECT pg_catalog.setval('perlengkapan.batch_operation_log_id_seq', 1, false);


--
-- Name: ms_aktivitas_bmn_id_seq; Type: SEQUENCE SET; Schema: perlengkapan; Owner: -
--

SELECT pg_catalog.setval('perlengkapan.ms_aktivitas_bmn_id_seq', 12, true);


--
-- PostgreSQL database dump complete
--

