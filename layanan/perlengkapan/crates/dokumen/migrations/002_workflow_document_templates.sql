-- Migration: Add workflow document templates
-- Description: Insert templates for SK Kebutuhan BMN and SK Penghapusan BMN
-- Requirements: REQ-D001, REQ-D002, REQ-W011
-- Date: 2026-02-11

-- Insert SK Kebutuhan BMN template
INSERT INTO dokumen.document_templates (
    id,
    name,
    description,
    template_type,
    content,
    format,
    output_format,
    variables,
    sample_data,
    letterhead_config,
    created_by,
    version
) VALUES (
    '00000000-0000-0000-0000-000000000001'::UUID,
    'SK Kebutuhan BMN',
    'Surat Keputusan untuk persetujuan kebutuhan Barang Milik Negara',
    'sk_kebutuhan_bmn',
    '<html>
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
</html>',
    'html',
    'pdf',
    '{"satker_nama": "string", "tahun_anggaran": "integer", "approval_date": "string", "document_type": "string"}'::JSONB,
    '{"satker_nama": "Kejaksaan Tinggi DKI Jakarta", "tahun_anggaran": 2026, "approval_date": "11 Februari 2026", "document_type": "SK Kebutuhan BMN"}'::JSONB,
    '{"logo_url": "/assets/logo-kejaksaan.png", "header_text": "KEJAKSAAN REPUBLIK INDONESIA"}'::JSONB,
    '00000000-0000-0000-0000-000000000000'::UUID,
    1
) ON CONFLICT (name, version) DO NOTHING;

-- Insert SK Penghapusan BMN template
INSERT INTO dokumen.document_templates (
    id,
    name,
    description,
    template_type,
    content,
    format,
    output_format,
    variables,
    sample_data,
    letterhead_config,
    created_by,
    version
) VALUES (
    '00000000-0000-0000-0000-000000000002'::UUID,
    'SK Penghapusan BMN',
    'Surat Keputusan untuk persetujuan penghapusan Barang Milik Negara',
    'sk_penghapusan_bmn',
    '<html>
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
</html>',
    'html',
    'pdf',
    '{"satker_nama": "string", "alasan": "string", "approval_date": "string", "document_type": "string"}'::JSONB,
    '{"satker_nama": "Kejaksaan Tinggi DKI Jakarta", "alasan": "Rusak berat dan tidak ekonomis untuk diperbaiki", "approval_date": "11 Februari 2026", "document_type": "SK Penghapusan BMN"}'::JSONB,
    '{"logo_url": "/assets/logo-kejaksaan.png", "header_text": "KEJAKSAAN REPUBLIK INDONESIA"}'::JSONB,
    '00000000-0000-0000-0000-000000000000'::UUID,
    1
) ON CONFLICT (name, version) DO NOTHING;

-- Add comments
COMMENT ON TABLE dokumen.document_templates IS 'Document templates for automated generation - includes workflow templates';
