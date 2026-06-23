<?php

use App\Http\Controllers\AnalisisKebutuhan\Bmn\AnalisisKelayakanController;
use App\Http\Controllers\AnalisisKebutuhan\Bmn\PengajuanController as BmnPengajuanController;
use App\Http\Controllers\AnalisisKebutuhan\Bmn\PenyusunanPrioritasController;
use App\Http\Controllers\AnalisisKebutuhan\PakaianDinas\LaporanController;
use App\Http\Controllers\AnalisisKebutuhan\PakaianDinas\PengajuanController;
use App\Http\Controllers\AnalisisKebutuhan\PakaianDinas\UkuranPakaianPegawaiController;
use App\Http\Controllers\Asset\AlatBesarController;
use App\Http\Controllers\Asset\AngkutanController;
use App\Http\Controllers\Asset\BangunanAirController;
use App\Http\Controllers\Asset\GedungController;
use App\Http\Controllers\Asset\JalanJembatanController;
use App\Http\Controllers\Asset\JaringanController;
use App\Http\Controllers\Asset\KonstruksiController;
use App\Http\Controllers\Asset\LainController;
use App\Http\Controllers\Asset\NonTikController;
use App\Http\Controllers\Asset\QrCodeController;
use App\Http\Controllers\Asset\RenovasiController;
use App\Http\Controllers\Asset\RumahController;
use App\Http\Controllers\Asset\SenjataController;
use App\Http\Controllers\Asset\TanahController;
use App\Http\Controllers\Asset\TikController;
use App\Http\Controllers\Asset\WujudController;
use App\Http\Controllers\AssetTik\HakciptaController;
use App\Http\Controllers\AssetTik\LanggananController;
use App\Http\Controllers\AuthController;
use App\Http\Controllers\Bmn\Hibah\HibahController;
use App\Http\Controllers\Bmn\Hibah\HibahMonitorController;
use App\Http\Controllers\Bmn\IjinController;
use App\Http\Controllers\Bmn\IjinMonitorController;
use App\Http\Controllers\Bmn\Pemanfaatan\PemanfaatanMonitorController;
use App\Http\Controllers\Bmn\Pemanfaatan\PemanfaatanSkController;
use App\Http\Controllers\Bmn\Pencabutan\PencabutanController;
use App\Http\Controllers\Bmn\Penetapan\PenetapanMonitorController;
use App\Http\Controllers\Bmn\Penghapusan\PenghapusanMonitorController;
use App\Http\Controllers\Bmn\Penghapusan\PenghapusanSkController;
use App\Http\Controllers\Bmn\Pnbp\PnbpController;
use App\Http\Controllers\MainController;
use App\Http\Controllers\Pengadaan\DistribusiController;
use App\Http\Controllers\Pengadaan\PokjaPemilihanController;
use App\Http\Controllers\Pengadaan\RencanapengadaanlangsungController;
use App\Http\Controllers\Pengadaan\UserSpseController;
use App\Http\Controllers\Pengguna\AktifitasController;
use App\Http\Controllers\Pengguna\PenggunaController;
use App\Http\Controllers\Pengguna\ProfilController;
use App\Http\Controllers\Pengguna\ReviewController;
use App\Http\Controllers\Suport\BukuPanduanController;
use App\Http\Controllers\Suport\FaquserController;
use App\Http\Controllers\Suport\HelpdeskController;
use App\Http\Controllers\Suport\NotifikasiManualController;
use App\Models\Master\MsSatker;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Route;

/*
|--------------------------------------------------------------------------
| API Routes
|--------------------------------------------------------------------------
|
| Here is where you can register API routes for your application. These
| routes are loaded by the RouteServiceProvider and all of them will
| be assigned to the "api" middleware group. Make something great!
|
*/

Route::middleware('auth:sanctum')->get('/user', function (Request $request) {
    return $request->user();
});
Route::post('login', [AuthController::class, 'loginApi']);
Route::middleware(['auth:api', 'token2session'])->group(function () {
    Route::get('logout', [AuthController::class, 'logoutApi']);
    Route::get('user', [AuthController::class, 'getAuthenticatedUser']);
    Route::post('change-password', [AuthController::class, 'changePassword']);
    Route::get('change-role/{roleId}', [AuthController::class, 'changeRoleMobile']);
    Route::get('/pengguna/gridData', [PenggunaController::class, 'gridData']);
    Route::get('/pengguna/aktifitas', [AktifitasController::class, 'gridData']);
    Route::post('pengguna/review', [ReviewController::class, 'store']);
    Route::post('pengguna/change-pp', [ProfilController::class, 'changePP']);

    Route::get('/analisis-kebutuhan/pakaian-dinas/pengajuan/gridDataSatker', [PengajuanController::class, 'gridDataSatker']);
    Route::get('/analisis-kebutuhan/pakaian-dinas/pengajuan', [PengajuanController::class, 'gridData']);
    Route::get('/analisis-kebutuhan/bmn/analisis-kelayakan', [AnalisisKelayakanController::class, 'gridData']);
    Route::get('/analisis-kebutuhan/bmn/analisis-kelayakan/{id}', [PenyusunanPrioritasController::class, 'gridDataSatker']);
    Route::get('/analisis-kebutuhan/bmn/analisis-kelayakan/detail/{id}', [AnalisisKelayakanController::class, 'edit']);
    Route::get('/analisis-kebutuhan/bmn/pengajuan', [BmnPengajuanController::class, 'gridData']);

    Route::get('/analisis-kebutuhan/pakaian-dinas/ukuran-pakaian-pegawai', [UkuranPakaianPegawaiController::class, 'getData']);
    Route::post('/analisis-kebutuhan/pakaian-dinas/ukuran-pakaian-pegawai', [UkuranPakaianPegawaiController::class, 'store']);

    Route::get('/bmn/pencabutan/pencabutan', [PencabutanController::class, 'gridData']);
    Route::get('/bmn/pemanfaatan/pemanfaatansk', [PemanfaatanSkController::class, 'gridData']);
    Route::get('/bmn/pemanfaatan/pemanfaatanmonitor', [PemanfaatanMonitorController::class, 'gridData']);

    Route::get('/bmn/penghapusan/penghapusansk', [PenghapusanSkController::class, 'gridData']);
    Route::get('/bmn/penghapusan/penghapusanmonitor', [PenghapusanMonitorController::class, 'gridData']);

    Route::get('/bmn/hibah/hibah', [HibahController::class, 'gridData']);
    Route::get('/bmn/hibah/hibahmonitor', [HibahMonitorController::class, 'gridData']);

    Route::get('/bmn/pnbp/pnbp', [PnbpController::class, 'gridData']);

    Route::get('/suport/faq/{platform}', [FaquserController::class, 'indexData']);
    Route::get('/suport/faq', [FaquserController::class, 'indexData']);

    Route::get('/pengadaan/user-sirup', [UserSpseController::class, 'gridData']);
    Route::get('/pengadaan/user-sirup/{id}', [UserSpseController::class, 'show']);
    Route::get('/pengadaan/user-spse', [UserSpseController::class, 'gridData']);
    Route::get('/pengadaan/user-spse/{id}', [UserSpseController::class, 'show']);
    Route::get('/pengadaan/pokja-pemilihan', [PokjaPemilihanController::class, 'gridData']);
    Route::get('/pengadaan/pokja-pemilihan/{id}', [PokjaPemilihanController::class, 'show']);
    Route::get('/pengadaan/rencanapengadaanlangsung', [RencanapengadaanlangsungController::class, 'gridData']);
    Route::get('/pengadaan/distribusi/{jenis}', [DistribusiController::class, 'gridData']);

    Route::get('/pengelolaan-bmn/penetapan-status-penggunaan', [PenetapanMonitorController::class, 'gridData']);
    Route::get('/pengelolaan-bmn/pemakaian-bmn', [PenetapanMonitorController::class, 'gridData']);

    Route::get('/asset-tik/daftar-aset', [TikController::class, 'gridData']);
    Route::get('/asset-tik/hak-cipta', [HakciptaController::class, 'gridData']);
    Route::get('/asset-tik/langganan', [LanggananController::class, 'gridData']);

    Route::get('/asset/tanah', [TanahController::class, 'gridData']);
    Route::get('/asset/angkutan', [AngkutanController::class, 'gridData']);
    Route::get('/asset/non-tik', [NonTikController::class, 'gridData']);
    Route::get('/asset/khusus-tik', [TikController::class, 'gridData']);
    Route::get('/asset/alat-besar', [AlatBesarController::class, 'gridData']);
    Route::get('/asset/alat-persenjataan', [SenjataController::class, 'gridData']);
    Route::get('/asset/gedung-bangunan', [GedungController::class, 'gridData']);
    Route::get('/asset/rumah-negara', [RumahController::class, 'gridData']);
    Route::get('/asset/jalan-jembatan', [JalanJembatanController::class, 'gridData']);
    Route::get('/asset/bangunan-air', [BangunanAirController::class, 'gridData']);
    Route::get('/asset/jaringan', [JaringanController::class, 'gridData']);
    Route::get('/asset/kontruksi', [KonstruksiController::class, 'gridData']);
    Route::get('/asset/asset-lain', [LainController::class, 'gridData']);
    Route::get('/asset/asset-renovasi', [RenovasiController::class, 'gridData']);
    Route::get('/asset/asset-tak-berwujud', [WujudController::class, 'gridData']);
    Route::post('/asset/read-qr', [QrCodeController::class, 'readQr']);

    Route::get('/bmn/ijin', [IjinController::class, 'gridData']);
    Route::get('/bmn/ijin/{id}', [IjinController::class, 'getData']);
    Route::get('/bmn/ijin-monitoring', [IjinMonitorController::class, 'gridData']);
    Route::get('/bmn/ijin-monitoring/{id}', [IjinMonitorController::class, 'gridDataPegawai']);

    Route::get('notifikasi-manual/get-by-role/{roleId}', [NotifikasiManualController::class, 'getByRole']);
    Route::get('notifikasi-manual/create', [NotifikasiManualController::class, 'createData']);
    Route::post('notifikasi-manual/create', [NotifikasiManualController::class, 'store']);
    Route::get('notifikasi-manual/show/{id}', [NotifikasiManualController::class, 'showData']);
    Route::get('notifikasi-manual', [NotifikasiManualController::class, 'gridData']);

    Route::get('notif', [MainController::class, 'getNotif']);
    Route::put('unreadnotif', [MainController::class, 'updateNotifID']);
    Route::get('guide-book/{platform}', [BukuPanduanController::class, 'getData']);

    Route::get('/bantuan/tiket-topik', [HelpdeskController::class, 'getTopik']);
    Route::get('/bantuan/tiket', [HelpdeskController::class, 'gridData']);
    Route::post('/bantuan/tiket', [HelpdeskController::class, 'store']);

    Route::get('/laporan/pakaian-dinas/tahun/{tahun}', [LaporanController::class, 'getPengadaanByTahun']);
    Route::get('/laporan/pakaian-dinas/cetak', [LaporanController::class, 'cetak']);
    Route::get('/laporan/pakaian-dinas', [LaporanController::class, 'mobile']);

    Route::get('/master/satker', function () {
        $model = MsSatker::class;
        $id_wilayah = request('id_wilayah', null);
        if ($id_wilayah == '00') {
            $data = $model::where('is_pusat', '=', 1);
        } elseif (empty($id_wilayah)) {
            $data = $model::where('is_pusat', '=', 0);
        } else {
            $data = $model::where('inst_satkerkd', 'like', "{$id_wilayah}%");
        }
        $data = $data->orderBy('inst_satkerkd')->get();

        return response()->json($data);
    });

});
