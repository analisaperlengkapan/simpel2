<?php

namespace App\Http\Controllers;

use App\Helpers\MyHelper;
use App\Models\Dashboard;
use App\Models\Master;
use App\Models\MsSatker;
use App\Models\Notifikasi;
use App\Models\Pengguna\Pengguna;
use Illuminate\Http\Request;
use Illuminate\Support\Arr;

class MainController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    public function index(Request $request)
    {
        // $create  = User::create(['name' => 'test', 'email' => 'test', 'password' => 'password', 'nip' => '1231231']);
        // $lastId = $create->value('id');
        // dd($lastId);
        // $data = $request->session()->all();

        // echo "<pre>";
        // print_r(session('userData.current_role'));exit;

        // $satkers = Master::getWilayahDashboard();

        // echo "<pre>";
        // print_r($satkers);exit;

        $dashboard = new Dashboard();
        $dataAset = $dashboard->getdata('statistik_bmn');
        $grafik_tahun_asset = $dashboard->getdata('statistik_grafik_tahun_asset');
        $grafik_psp_asset = $dashboard->getdata('statistik_grafik_psp_asset');
        
        // Get kondisi data for each asset type
        $kondisi_aset_lainnya = $dashboard->getdata('statistik_kondisi_aset_lainnya');
        $kondisi_renovasi = $dashboard->getdata('statistik_kondisi_renovasi');
        $kondisi_konstruksi = $dashboard->getdata('statistik_kondisi_konstruksi');
        $kondisi_jalan_jembatan = $dashboard->getdata('statistik_kondisi_jalan_jembatan');
        $kondisi_bangunan_air = $dashboard->getdata('statistik_kondisi_bangunan_air');
        $kondisi_rumah = $dashboard->getdata('statistik_kondisi_rumah');
        $kondisi_instalasi_jaringan = $dashboard->getdata('statistik_kondisi_instalasi_jaringan');
        $kondisi_tak_berwujud = $dashboard->getdata('statistik_kondisi_tak_berwujud');
        $kondisi_tik = $dashboard->getdata('statistik_kondisi_tik');
        $kondisi_nontik = $dashboard->getdata('statistik_kondisi_nontik');
        $kondisi_kendaraan = $dashboard->getdata('statistik_kondisi_kendaraan');
        $kondisi_alat_berat = $dashboard->getdata('statistik_kondisi_alat_berat');
        $kondisi_gedung = $dashboard->getdata('statistik_kondisi_gedung');
        $kondisi_tanah = $dashboard->getdata('statistik_kondisi_tanah');
        $analisis_kebutuhan_bmn = $dashboard->getdata('analisis_kebutuhan_bmn');

        $tahun = [];
        $totalAsset = [];
        $psp = [];
        $totalpsp = [];

        //$tahun_perolehan = (array) $dashboard->getTahunPerolehan();
        if (!empty($grafik_tahun_asset)) {
            $tahun = Arr::pluck($grafik_tahun_asset, 'tahun');
            $totalAsset = Arr::pluck($grafik_tahun_asset, 'total');
        }

        if (!empty($grafik_psp_asset)) {
            $psp = Arr::pluck($grafik_psp_asset, 'tipe');
            $totalpsp = Arr::pluck($grafik_psp_asset, 'total');
        }

        $jenis = ['K/L', 'WILAYAH', 'SATKER'];
        $wilayah = Master::getWilayahDashboard();
        $satker = Master::getSatkerDashboard();

        $data = [
            'statistik' => $dataAset,

            'data_tahun_asset' => $grafik_tahun_asset,
            'tahun' => $tahun,
            'totalAsset' => $totalAsset,

            'data_psp_asset' => $grafik_psp_asset,
            'psp' => $psp,
            'totalpsp' => $totalpsp,

            // Kondisi data for charts
            'kondisi_aset_lainnya' => $kondisi_aset_lainnya,
            'kondisi_renovasi' => $kondisi_renovasi,
            'kondisi_konstruksi' => $kondisi_konstruksi,
            'kondisi_jalan_jembatan' => $kondisi_jalan_jembatan,
            'kondisi_bangunan_air' => $kondisi_bangunan_air,
            'kondisi_rumah' => $kondisi_rumah,
            'kondisi_instalasi_jaringan' => $kondisi_instalasi_jaringan,
            'kondisi_tak_berwujud' => $kondisi_tak_berwujud,
            'kondisi_tik' => $kondisi_tik,
            'kondisi_nontik' => $kondisi_nontik,
            'kondisi_kendaraan' => $kondisi_kendaraan,
            'kondisi_alat_berat' => $kondisi_alat_berat,
            'kondisi_gedung' => $kondisi_gedung,
            'kondisi_tanah' => $kondisi_tanah,
            'analisis_kebutuhan_bmn' => $analisis_kebutuhan_bmn,

            //'yearOptions' => MyHelper::generateSelectOptions(['data' => $tahun_perolehan, 'selected'=>'']),

            'jenisOptions' => MyHelper::generateSelectOptions([
                'data' => $jenis,
                'text' => 'jenis',
                'value' => null,
                'selected' => '',
            ]),
            'wilayahOptions' => MyHelper::generateSelectOptions([
                'data' => $wilayah,
                'text' => 'inst_nama',
                'value' => 'inst_satkerkd',
                'selected' => '',
            ]),
            'satkerOptions' => MyHelper::generateSelectOptions([
                'data' => $satker,
                'text' => 'inst_nama',
                'value' => 'inst_satkerkd',
                'selected' => '',
            ]),
        ];

        // echo "<pre>";
        // print_r($data);exit;

        return view('dashboard.dashboard', $data);
    }


    /**
     * Show the form for creating a new resource.
     */
    public function create()
    {
        //
    }

    /**
     * Store a newly created resource in storage.
     */
    public function dashboardDetail(Request $request)
    {
        $tipe = $request->input('tipe');
        $level = $request->input('level_laporan');
        $wilayah = $request->input('wilayah');
        $satker = $request->input('satker');

        $dashboard = new Dashboard();

        switch ($tipe) {
            case "lain":
                $template = 'dashboard.dashboardAsetlainnya';
                $kondisi_aset_lainnya = $dashboard->getdata('statistik_kondisi_aset_lainnya', $level, $wilayah, $satker);
                $kelompok_aset_lainnya = $dashboard->getdata('statistik_kelompok_aset_lainnya', $level, $wilayah, $satker);
                $subkelompok_aset_lainnya = $dashboard->getdata('statistik_subkelompok_aset_lainnya', $level, $wilayah, $satker);

                $data = [
                    'kondisi_aset_lainnya' => $kondisi_aset_lainnya,
                    'kelompok_aset_lainnya' => $kelompok_aset_lainnya,
                    'subkelompok_aset_lainnya' => $subkelompok_aset_lainnya,
                ];
                break;
            case "renovasi":
                $template = 'dashboard.dashboardRenovasi';
                $kondisi_renovasi = $dashboard->getdata('statistik_kondisi_renovasi', $level, $wilayah, $satker);
                $kelompok_renovasi = $dashboard->getdata('statistik_kelompok_renovasi', $level, $wilayah, $satker);
                $subkelompok_renovasi = $dashboard->getdata('statistik_subkelompok_renovasi', $level, $wilayah, $satker);

                $data = [
                    'kondisi_renovasi' => $kondisi_renovasi,
                    'kelompok_renovasi' => $kelompok_renovasi,
                    'subkelompok_renovasi' => $subkelompok_renovasi,
                ];
                break;
            case "konstruksi":
                $template = 'dashboard.dashboardKonstruksi';
                $kondisi_konstruksi = $dashboard->getdata('statistik_kondisi_konstruksi', $level, $wilayah, $satker);
                $kelompok_konstruksi = $dashboard->getdata('statistik_kelompok_konstruksi', $level, $wilayah, $satker);
                $subkelompok_konstruksi = $dashboard->getdata('statistik_subkelompok_konstruksi', $level, $wilayah, $satker);

                $data = [
                    'kondisi_konstruksi' => $kondisi_konstruksi,
                    'kelompok_konstruksi' => $kelompok_konstruksi,
                    'subkelompok_konstruksi' => $subkelompok_konstruksi,
                ];
                break;
            case "konstruksi":
                $template = 'dashboard.dashboardKonstruksi';
                $kondisi_konstruksi = $dashboard->getdata('statistik_kondisi_konstruksi', $level, $wilayah, $satker);
                $kelompok_konstruksi = $dashboard->getdata('statistik_kelompok_konstruksi', $level, $wilayah, $satker);
                $subkelompok_konstruksi = $dashboard->getdata('statistik_subkelompok_konstruksi', $level, $wilayah, $satker);

                $data = [
                    'kondisi_konstruksi' => $kondisi_konstruksi,
                    'kelompok_konstruksi' => $kelompok_konstruksi,
                    'subkelompok_konstruksi' => $subkelompok_konstruksi,
                ];
                break;
            case "jalan_jembatan":
                $template = 'dashboard.dashboardJalanjembatan';
                $kondisi_jalan_jembatan = $dashboard->getdata('statistik_kondisi_jalan_jembatan', $level, $wilayah, $satker);
                $kelompok_jalan_jembatan = $dashboard->getdata('statistik_kelompok_jalan_jembatan', $level, $wilayah, $satker);
                $subkelompok_jalan_jembatan = $dashboard->getdata('statistik_subkelompok_jalan_jembatan', $level, $wilayah, $satker);

                $data = [
                    'kondisi_jalan_jembatan' => $kondisi_jalan_jembatan,
                    'kelompok_jalan_jembatan' => $kelompok_jalan_jembatan,
                    'subkelompok_jalan_jembatan' => $subkelompok_jalan_jembatan,
                ];
                break;
            case "bangunan_air":
                $template = 'dashboard.dashboardBangunanair';
                $kondisi_bangunan_air = $dashboard->getdata('statistik_kondisi_bangunan_air', $level, $wilayah, $satker);
                $kelompok_bangunan_air = $dashboard->getdata('statistik_kelompok_bangunan_air', $level, $wilayah, $satker);
                $subkelompok_bangunan_air = $dashboard->getdata('statistik_subkelompok_bangunan_air', $level, $wilayah, $satker);

                $data = [
                    'kondisi_bangunan_air' => $kondisi_bangunan_air,
                    'kelompok_bangunan_air' => $kelompok_bangunan_air,
                    'subkelompok_bangunan_air' => $subkelompok_bangunan_air,
                ];
                break;
            case "bangunan_air":
                $template = 'dashboard.dashboardBangunanair';
                $kondisi_bangunan_air = $dashboard->getdata('statistik_kondisi_bangunan_air', $level, $wilayah, $satker);
                $kelompok_bangunan_air = $dashboard->getdata('statistik_kelompok_bangunan_air', $level, $wilayah, $satker);
                $subkelompok_bangunan_air = $dashboard->getdata('statistik_subkelompok_bangunan_air', $level, $wilayah, $satker);

                $data = [
                    'kondisi_bangunan_air' => $kondisi_bangunan_air,
                    'kelompok_bangunan_air' => $kelompok_bangunan_air,
                    'subkelompok_bangunan_air' => $subkelompok_bangunan_air,
                ];
                break;
            case "rumah":
                $template = 'dashboard.dashboardRumah';
                $kondisi_rumah = $dashboard->getdata('statistik_kondisi_rumah', $level, $wilayah, $satker);
                $kelompok_rumah = $dashboard->getdata('statistik_kelompok_rumah', $level, $wilayah, $satker);
                $subkelompok_rumah = $dashboard->getdata('statistik_subkelompok_rumah', $level, $wilayah, $satker);

                $data = [
                    'kondisi_rumah' => $kondisi_rumah,
                    'kelompok_rumah' => $kelompok_rumah,
                    'subkelompok_rumah' => $subkelompok_rumah,
                ];
                break;
            case "jaringan":
                $template = 'dashboard.dashboardInstalasijaringan';
                $kondisi_instalasi_jaringan = $dashboard->getdata('statistik_kondisi_instalasi_jaringan', $level, $wilayah, $satker);
                $kelompok_instalasi_jaringan = $dashboard->getdata('statistik_kelompok_instalasi_jaringan', $level, $wilayah, $satker);
                $subkelompok_instalasi_jaringan = $dashboard->getdata('statistik_subkelompok_instalasi_jaringan', $level, $wilayah, $satker);

                $data = [
                    'kondisi_instalasi_jaringan' => $kondisi_instalasi_jaringan,
                    'kelompok_instalasi_jaringan' => $kelompok_instalasi_jaringan,
                    'subkelompok_instalasi_jaringan' => $subkelompok_instalasi_jaringan,
                ];
                break;
            case "jaringan":
                $template = 'dashboard.dashboardInstalasijaringan';
                $kondisi_instalasi_jaringan = $dashboard->getdata('statistik_kondisi_instalasi_jaringan', $level, $wilayah, $satker);
                $kelompok_instalasi_jaringan = $dashboard->getdata('statistik_kelompok_instalasi_jaringan', $level, $wilayah, $satker);
                $subkelompok_instalasi_jaringan = $dashboard->getdata('statistik_subkelompok_instalasi_jaringan', $level, $wilayah, $satker);

                $data = [
                    'kondisi_instalasi_jaringan' => $kondisi_instalasi_jaringan,
                    'kelompok_instalasi_jaringan' => $kelompok_instalasi_jaringan,
                    'subkelompok_instalasi_jaringan' => $subkelompok_instalasi_jaringan,
                ];
                break;
            case "wujud":
                $template = 'dashboard.dashboardTakwujud';
                $kondisi_tak_berwujud = $dashboard->getdata('statistik_kondisi_tak_berwujud', $level, $wilayah, $satker);
                $kelompok_tak_berwujud = $dashboard->getdata('statistik_kelompok_tik', $level, $wilayah, $satker);

                $data = [
                    'kondisi_tak_berwujud' => $kondisi_tak_berwujud,
                    'kelompok_tak_berwujud' => $kelompok_tak_berwujud,
                ];
                break;
            case "tik":
                $template = 'dashboard.dashboardTik';
                $kondisi_tik = $dashboard->getdata('statistik_kondisi_tik', $level, $wilayah, $satker);
                $penggunaan_tik = $dashboard->getdata('statistik_penggunaan_tik', $level, $wilayah, $satker);
                $kelompok_tik = $dashboard->getdata('statistik_kelompok_tik', $level, $wilayah, $satker);
                $subkelompok_tik = $dashboard->getdata('statistik_subkelompok_tik', $level, $wilayah, $satker);

                $data = [
                    'kondisi_tik' => $kondisi_tik,
                    'penggunaan_tik' => $penggunaan_tik,
                    'kelompok_tik' => $kelompok_tik,
                    'subkelompok_tik' => $subkelompok_tik,
                ];
                break;
            case "non_tik":
                $template = 'dashboard.dashboardNontik';
                $kondisi_nontik = $dashboard->getdata('statistik_kondisi_nontik', $level, $wilayah, $satker);
                $kelompok_nontik = $dashboard->getdata('statistik_kelompok_nontik', $level, $wilayah, $satker);
                $subkelompok_nontik = $dashboard->getdata('statistik_subkelompok_nontik', $level, $wilayah, $satker);

                $data = [
                    'kondisi_nontik' => $kondisi_nontik,
                    'kelompok_nontik' => $kelompok_nontik,
                    'subkelompok_nontik' => $subkelompok_nontik,
                ];
                break;
            case "angkutan":
                $template = 'dashboard.dashboardAngkutan';
                $kondisi_kendaraan = $dashboard->getdata('statistik_kondisi_kendaraan', $level, $wilayah, $satker);
                $penggunaan_kendaraan = $dashboard->getdata('statistik_penggunaan_kendaraan', $level, $wilayah, $satker);
                $kelompok_kendaraan = $dashboard->getdata('statistik_kelompok_kendaraan', $level, $wilayah, $satker);
                $subkelompok_kendaraan = $dashboard->getdata('statistik_subkelompok_kendaraan', $level, $wilayah, $satker);

                $data = [
                    'kondisi_kendaraan' => $kondisi_kendaraan,
                    'penggunaan_kendaraan' => $penggunaan_kendaraan,
                    'kelompok_kendaraan' => $kelompok_kendaraan,
                    'subkelompok_kendaraan' => $subkelompok_kendaraan,
                ];
                break;
            case "angkutan":
                $template = 'dashboard.dashboardAngkutan';
                $kondisi_kendaraan = $dashboard->getdata('statistik_kondisi_kendaraan', $level, $wilayah, $satker);
                $penggunaan_kendaraan = $dashboard->getdata('statistik_penggunaan_kendaraan', $level, $wilayah, $satker);
                $kelompok_kendaraan = $dashboard->getdata('statistik_kelompok_kendaraan', $level, $wilayah, $satker);
                $subkelompok_kendaraan = $dashboard->getdata('statistik_subkelompok_kendaraan', $level, $wilayah, $satker);

                $data = [
                    'kondisi_kendaraan' => $kondisi_kendaraan,
                    'penggunaan_kendaraan' => $penggunaan_kendaraan,
                    'kelompok_kendaraan' => $kelompok_kendaraan,
                    'subkelompok_kendaraan' => $subkelompok_kendaraan,
                ];
                break;
            case "alat_besar":
                $template = 'dashboard.dashboardAlatbesar';
                $kondisi_alat_berat = $dashboard->getdata('statistik_kondisi_alat_berat', $level, $wilayah, $satker);
                $penggunaan_alat_berat = $dashboard->getdata('statistik_penggunaan_alat_berat', $level, $wilayah, $satker);
                $kelompok_alat_berat = $dashboard->getdata('statistik_kelompok_alat_berat', $level, $wilayah, $satker);
                $subkelompok_alat_berat = $dashboard->getdata('statistik_subkelompok_alat_berat', $level, $wilayah, $satker);

                $data = [
                    'kondisi_alat_berat' => $kondisi_alat_berat,
                    'penggunaan_alat_berat' => $penggunaan_alat_berat,
                    'kelompok_alat_berat' => $kelompok_alat_berat,
                    'subkelompok_alat_berat' => $subkelompok_alat_berat,
                ];
                break;
            case "gedung":
                $template = 'dashboard.dashboardGedung';
                $kondisi_gedung = $dashboard->getdata('statistik_kondisi_gedung', $level, $wilayah, $satker);
                $penggunaan_gedung = $dashboard->getdata('statistik_penggunaan_gedung', $level, $wilayah, $satker);
                $kelompok_gedung = $dashboard->getdata('statistik_kelompok_gedung', $level, $wilayah, $satker);
                $subkelompok_gedung = $dashboard->getdata('statistik_subkelompok_gedung', $level, $wilayah, $satker);

                $data = [
                    'kondisi_gedung' => $kondisi_gedung,
                    'penggunaan_gedung' => $penggunaan_gedung,
                    'kelompok_gedung' => $kelompok_gedung,
                    'subkelompok_gedung' => $subkelompok_gedung,
                ];
                break;
            case "gedung":
                $template = 'dashboard.dashboardGedung';
                $kondisi_gedung = $dashboard->getdata('statistik_kondisi_gedung', $level, $wilayah, $satker);
                $penggunaan_gedung = $dashboard->getdata('statistik_penggunaan_gedung', $level, $wilayah, $satker);
                $kelompok_gedung = $dashboard->getdata('statistik_kelompok_gedung', $level, $wilayah, $satker);
                $subkelompok_gedung = $dashboard->getdata('statistik_subkelompok_gedung', $level, $wilayah, $satker);

                $data = [
                    'kondisi_gedung' => $kondisi_gedung,
                    'penggunaan_gedung' => $penggunaan_gedung,
                    'kelompok_gedung' => $kelompok_gedung,
                    'subkelompok_gedung' => $subkelompok_gedung,
                ];
                break;
            case "tanah":
                $template = 'dashboard.dashboardTanah';
                $klasifikasi_tanah = $dashboard->getdata('statistik_klasifikasi_tanah', $level, $wilayah, $satker);
                $kelompok_tanah = $dashboard->getdata('statistik_kelompok_tanah', $level, $wilayah, $satker);
                $sub_kelompok_tanah = $dashboard->getdata('statistik_subkelompok_tanah', $level, $wilayah, $satker);

                $data = [
                    'klasifikasi_tanah' => $klasifikasi_tanah,
                    'kelompok_tanah' => $kelompok_tanah,
                    'sub_kelompok_tanah' => $sub_kelompok_tanah,
                ];
                break;
            case "tanah":
                $template = 'dashboard.dashboardTanah';
                $klasifikasi_tanah = $dashboard->getdata('statistik_klasifikasi_tanah', $level, $wilayah, $satker);
                $kelompok_tanah = $dashboard->getdata('statistik_kelompok_tanah', $level, $wilayah, $satker);
                $sub_kelompok_tanah = $dashboard->getdata('statistik_subkelompok_tanah', $level, $wilayah, $satker);

                $data = [
                    'klasifikasi_tanah' => $klasifikasi_tanah,
                    'kelompok_tanah' => $kelompok_tanah,
                    'sub_kelompok_tanah' => $sub_kelompok_tanah,
                ];
                break;
        }

        return view($template, $data);
    }

    public function dashboard(Request $request)
    {
        $level = $request->input('level_laporan');
        $wilayah = $request->input('wilayah');
        $satker = $request->input('satker');

        $dashboard = new Dashboard();
        $dataAset = $dashboard->getdata('statistik_bmn', $level, $wilayah, $satker);
        $grafik_tahun_asset = $dashboard->getdata('statistik_grafik_tahun_asset', $level, $wilayah, $satker);
        $grafik_psp_asset = $dashboard->getdata('statistik_grafik_psp_asset', $level, $wilayah, $satker);
        
        // Get kondisi data for each asset type with filters
        $kondisi_aset_lainnya = $dashboard->getdata('statistik_kondisi_aset_lainnya', $level, $wilayah, $satker);
        $kondisi_renovasi = $dashboard->getdata('statistik_kondisi_renovasi', $level, $wilayah, $satker);
        $kondisi_konstruksi = $dashboard->getdata('statistik_kondisi_konstruksi', $level, $wilayah, $satker);
        $kondisi_jalan_jembatan = $dashboard->getdata('statistik_kondisi_jalan_jembatan', $level, $wilayah, $satker);
        $kondisi_bangunan_air = $dashboard->getdata('statistik_kondisi_bangunan_air', $level, $wilayah, $satker);
        $kondisi_rumah = $dashboard->getdata('statistik_kondisi_rumah', $level, $wilayah, $satker);
        $kondisi_instalasi_jaringan = $dashboard->getdata('statistik_kondisi_instalasi_jaringan', $level, $wilayah, $satker);
        $kondisi_tak_berwujud = $dashboard->getdata('statistik_kondisi_tak_berwujud', $level, $wilayah, $satker);
        $kondisi_tik = $dashboard->getdata('statistik_kondisi_tik', $level, $wilayah, $satker);
        $kondisi_nontik = $dashboard->getdata('statistik_kondisi_nontik', $level, $wilayah, $satker);
        $kondisi_kendaraan = $dashboard->getdata('statistik_kondisi_kendaraan', $level, $wilayah, $satker);
        $kondisi_alat_berat = $dashboard->getdata('statistik_kondisi_alat_berat', $level, $wilayah, $satker);
        $kondisi_gedung = $dashboard->getdata('statistik_kondisi_gedung', $level, $wilayah, $satker);

        $tahun = [];
        $totalAsset = [];
        $psp = [];
        $totalpsp = [];

        //$tahun_perolehan = (array) $dashboard->getTahunPerolehan();
        if (!empty($grafik_tahun_asset)) {
            $tahun = Arr::pluck($grafik_tahun_asset, 'tahun');
            $totalAsset = Arr::pluck($grafik_tahun_asset, 'total');
        }
        if (!empty($grafik_psp_asset)) {
            $psp = Arr::pluck($grafik_psp_asset, 'tipe');
            $totalpsp = Arr::pluck($grafik_psp_asset, 'total');
        }
        //$satkers = Master::getSatkersKeu();

        $jenis = ['K/L', 'WILAYAH', 'SATKER'];
        $wilayahs = Master::getWilayahDashboard();
        $satkers = Master::getSatkerDashboard();

        $data = [
            'statistik' => $dataAset,

            'data_tahun_asset' => $grafik_tahun_asset,
            'tahun' => $tahun,
            'totalAsset' => $totalAsset,

            'data_psp_asset' => $grafik_psp_asset,
            'psp' => $psp,
            'totalpsp' => $totalpsp,

            // Kondisi data for charts
            'kondisi_aset_lainnya' => $kondisi_aset_lainnya,
            'kondisi_renovasi' => $kondisi_renovasi,
            'kondisi_konstruksi' => $kondisi_konstruksi,
            'kondisi_jalan_jembatan' => $kondisi_jalan_jembatan,
            'kondisi_bangunan_air' => $kondisi_bangunan_air,
            'kondisi_rumah' => $kondisi_rumah,
            'kondisi_instalasi_jaringan' => $kondisi_instalasi_jaringan,
            'kondisi_tak_berwujud' => $kondisi_tak_berwujud,
            'kondisi_tik' => $kondisi_tik,
            'kondisi_nontik' => $kondisi_nontik,
            'kondisi_kendaraan' => $kondisi_kendaraan,
            'kondisi_alat_berat' => $kondisi_alat_berat,
            'kondisi_gedung' => $kondisi_gedung,

            'jenisOptions' => MyHelper::generateSelectOptions([
                'data' => $jenis,
                'text' => 'jenis',
                'value' => null,
                'selected' => $level ?? '',
            ]),
            'wilayahOptions' => MyHelper::generateSelectOptions([
                'data' => $wilayahs,
                'text' => 'inst_nama',
                'value' => 'inst_satkerkd',
                'selected' => $wilayah ?? '',
            ]),
            'satkerOptions' => MyHelper::generateSelectOptions([
                'data' => $satkers,
                'text' => 'inst_nama',
                'value' => 'inst_satkerkd',
                'selected' => $satker ?? '',
            ]),

        ];

        return view('dashboard.dashboard', $data);
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        //
    }

    /**
     * Show the form for editing the specified resource.
     */
    public function edit(string $id)
    {
        //
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, string $id)
    {
        //
    }

    /**
     * Remove the specified resource from storage.
     */
    public function destroy(string $id)
    {
        //
    }

    public function error($code, $msg)
    {
        return view('error', ['errCode' => $code, 'errMsg' => $msg]);
    }

    public function searchPegawai(string $nip)
    {
        $pegawai = Master::getPegawaiByNip($nip);
        if ($pegawai)
            return $this->resSuccess('ok', null, $pegawai);
        return $this->resError('Pegawai Tidak ditemukan');
    }

    public function getNotif()
    {
        $notifs = Notifikasi::getNotifs();
        return response()->json($notifs, 200);
    }

    public function updateNotifID(Request $request)
    {
        $requestData = $request->json()->all();
        $id = $requestData['id'];

        //echo $id;exit;
        $notif = Notifikasi::find($id);

        if (!$notif) {
            return response()->json(['message' => 'Notifikasi not found'], 404);
        }

        // Update the 'is_read' attribute
        $notif->update([
            'is_read' => 1,
        ]);

        return response()->json(['message' => 'Notifikasi updated successfully'], 200);
    }

    public function getGuideBook()
    {
        $path = config('constants.buku_panduan_path');
        return response()->json(asset($path));
    }

    public function gridDataPegawaiDashboard(Request $request)
    {
        $user = new Master();
        $pagingParams = $request->only(['start', 'length']);
        $searchParams = $request->only(['columns']);

        $data = $user->gridDataPegawaiDashboard($pagingParams, $searchParams);
        return response()->json([
            'data' => $data['data'],
            'recordsTotal' => $data['total'],
            'recordsFiltered' => $data['total'],
        ]);
    }

    public function getUserChangers()
    {
        $search = request()->input('q') ?? null;

        $data = Pengguna::getUserChanger(trim(strtolower($search)));
        return response()->json([
            'data' => $data
        ]);
    }
}
