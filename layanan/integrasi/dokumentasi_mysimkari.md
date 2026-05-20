API MySIMKARI

1. Pertama tarik dulu data satker

Endpoint: https://mysimkari.kejaksaan.go.id/api/anbut/get-satker
Auth Type: Bearer Token
Sample data tarikan:
{
    "status": "200",
    "message": "Berhasil",
    "data": [
        {
            "id": "36b85f1e-e304-4c42-b7da-7f03f0c431d3",
            "parent_id": null,
            "nama_satker": "KEJAKSAAN AGUNG",
            "tipe_satker": "Kejaksaan Agung",
            "alamat_satker": "Jl. Sultan Hasanuddin  Nomor 1, Kebayoran Baru, Jakarta Selatan",
            "kode_satker": "00",
            "telp_satker": "Telp. (021) 7236510",
            "website_satker": "www.kejaksaan.go.id",
            "city": "Jakarta",
            "long": "106.797822",
            "lat": "-6.240841",
            "provinsi": "DKI JAKARTA",
            "wilayah": "II",
            "kategori_satker": "A",
            "pulau": "Jawa"
        },
        {
            "id": "9517d6c8-814e-41c6-893c-daa693cac03b",
            "parent_id": "36b85f1e-e304-4c42-b7da-7f03f0c431d3",
            "nama_satker": "KEJAKSAAN TINGGI ACEH",
            "tipe_satker": "Kejaksaan Tinggi",
            "alamat_satker": "Jl. Dr. Mohd. Hasan, Batoh, Kota Banda Aceh 23245",
            "kode_satker": "01",
            "telp_satker": "(0651) 22240 Fax. (0651) 23245",
            "website_satker": "https://kejati-aceh.kejaksaan.go.id/",
            "city": "Banda Aceh",
            "long": "95.329354",
            "lat": "5.523631",
            "provinsi": "DI. ACEH",
            "wilayah": "I",
            "kategori_satker": "B",
            "pulau": "Sumatera"
        },
        {
            "id": "587bc3aa-4177-4257-8307-01cf1edcad1b",
            "parent_id": "9517d6c8-814e-41c6-893c-daa693cac03b",
            "nama_satker": "KEJAKSAAN NEGERI BANDA ACEH",
            "tipe_satker": "Kejaksaan Negeri",
            "alamat_satker": "JALAN. CUT MUTIA NO 21 BANDA ACEH",
            "kode_satker": "01.01",
            "telp_satker": "(0651) 22241",
            "website_satker": "kejari-bandaaceh.go.id",
            "city": "Banda Aceh",
            "long": "95.317246",
            "lat": "5.556329",
            "provinsi": "DI. ACEH",
            "wilayah": "I",
            "kategori_satker": "A",
            "pulau": "Sumatera"
        },

2. Kemudian tarik data pegawai dari setiap satker
Endpoint: https://mysimkari.kejaksaan.go.id/api/anbut/pegawai-satker/[id dari get-satker]
Auth Type: Bearer Token
Sample data tarikan:
{
    "status": "200",
    "message": "Berhasil",
    "data": [
        {
            "nama": "EFFENDI MARUAPEY, S.H., M.H.",
            "nip": "11807",
            "no_hp": "",
            "email_dinas": "",
            "bidang": "PIDMIL",
            "foto": "",
            "jk": "L",
            "agama": "Islam",
            "nrp": "11807/P",
            "jabatan": "Direktur Penindakan pada Jaksa Agung Muda Bidang Pidana Militer Kejaksaan Agung",
            "golpang": "Muda Pati / (IV/c)",
            "jenis_jabatan_terakhir": "STRUKTURAL TU",
            "jabat_tmt": "2024-04-23",
            "eselon": "II/a",
            "nama_satker": "KEJAKSAAN AGUNG",
            "GOL_KD": "IV/c"
        },
        {
            "nama": "ASKARI, S.H.",
            "nip": "11950003260368",
            "no_hp": "085318881995",
            "email_dinas": "askari1195@kejaksaan.go.id",
            "bidang": "PIDMIL",
            "foto": "",
            "jk": "L",
            "agama": null,
            "nrp": "11950003260368",
            "jabatan": "Kepala Subdirektorat Koordinasi Penuntutan pada Direktorat Penuntutan Jaksa Agung Muda Bidang Pidana Militer Kejaksaan Agung",
            "golpang": "Nindya Wira TU / (IV/b)",
            "jenis_jabatan_terakhir": "STRUKTURAL TU",
            "jabat_tmt": "2025-02-07",
            "eselon": "III/a",
            "nama_satker": "KEJAKSAAN AGUNG",
            "GOL_KD": "IV/b"
        },
        {
            "nama": "HAIRUL ARIFIN, S.P. S.H.",
            "nip": "11990003480770",
            "no_hp": "081237177137",
            "email_dinas": "hairul.arifin@kejaksaan.go.id",
            "bidang": "PIDMIL",
            "foto": "QaM74MYAZMOfvAahArGFftjalv5deAOkKuning.jpg",
            "jk": "L",
            "agama": "Islam",
            "nrp": "11990003480770",
            "jabatan": "Kepala Subdirektorat Koordinasi Penindakan pada Direktorat Penindakan Jaksa Agung Muda Bidang Pidana Militer Kejaksaan Agung",
            "golpang": "IV/a / (Adi Wira)",
            "jenis_jabatan_terakhir": "STRUKTURAL TU",
            "jabat_tmt": "2025-04-24",
            "eselon": "III/a",
            "nama_satker": "KEJAKSAAN AGUNG",
            "GOL_KD": "IV/a"
        },
