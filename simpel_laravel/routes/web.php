<?php

use App\Http\Controllers\AnalisisKebutuhan\Bmn\AnalisisKelayakanController;
use App\Http\Controllers\AnalisisKebutuhan\Bmn\DaftarKebutuhanBmnController;
use App\Http\Controllers\AnalisisKebutuhan\Bmn\PengajuanController as PengajuanBmn;
use App\Http\Controllers\AnalisisKebutuhan\Bmn\PenyusunanPrioritasController;
use App\Http\Controllers\AnalisisKebutuhan\PakaianDinas\LaporanController as PengajuanPakaianDinasLaporan;
use App\Http\Controllers\AnalisisKebutuhan\PakaianDinas\PengajuanController as PengajuanPakaianDinas;
use App\Http\Controllers\AnalisisKebutuhan\PakaianDinas\UkuranPakaianPegawaiController;
use App\Http\Controllers\Aset\AlatBesarController;
use App\Http\Controllers\Aset\AngkutanController;
use App\Http\Controllers\Aset\BangunanAirController;
use App\Http\Controllers\Aset\GedungController;
use App\Http\Controllers\Aset\JalanJembatanController;
use App\Http\Controllers\Aset\JaringanController;
use App\Http\Controllers\Aset\KonstruksiController;
use App\Http\Controllers\Aset\LainController;
use App\Http\Controllers\Aset\NonTikController;
use App\Http\Controllers\Aset\QrCodeController;
use App\Http\Controllers\Aset\RenovasiController;
use App\Http\Controllers\Aset\RumahController;
use App\Http\Controllers\Aset\SebaranController;
use App\Http\Controllers\Aset\SenjataController;
use App\Http\Controllers\Aset\TanahController;
use App\Http\Controllers\Aset\TikController;
use App\Http\Controllers\Aset\WujudController;
use App\Http\Controllers\AsetIntelektual\BmnTikController;
use App\Http\Controllers\AsetIntelektual\HakciptaController;
use App\Http\Controllers\AsetIntelektual\LanggananController;
use App\Http\Controllers\Bmn\Hibah\HibahController as Hibah;
use App\Http\Controllers\Bmn\Hibah\HibahMonitorController as HibahMonitor;
use App\Http\Controllers\Bmn\IjinController;
use App\Http\Controllers\Bmn\IjinPegawaiController;
use App\Http\Controllers\Bmn\IjinMonitorController;
use App\Http\Controllers\Bmn\Jasa\JasaController as Jasa;
use App\Http\Controllers\Bmn\Pencabutan\PencabutanController as Pencabutan;
use App\Http\Controllers\Bmn\Penghapusan\PengajuanPenghapusanBmnController;
use App\Http\Controllers\Bmn\Penghapusan\PenghapusanMonitorController as PenghapusanMonitor;
use App\Http\Controllers\Bmn\Penghapusan\PenghapusanSkController as PenghapusanSk;
use App\Http\Controllers\Bmn\Penghapusan\PermohonanPenghapusanBmnController;
use App\Http\Controllers\Bmn\Penghapusan\PermohonanPenghapusanBmnMonitorController;
use App\Http\Controllers\Bmn\Penghapusan\PermohonanPenghapusanBmnSkController;
use App\Http\Controllers\Bmn\Penghapusan\PersetujuanBmnMonitorController;
use App\Http\Controllers\Bmn\Perawatan\PerawatanController as Perawatan;
use App\Http\Controllers\Bmn\Perawatan\PerawatanMonitorController as PerawatanMonitor;
use App\Http\Controllers\Sistem\LogIntegrasiController;
use App\Http\Controllers\Dashboard\MainController;
use App\Http\Controllers\Master\IntegrasiDataController;
use App\Http\Controllers\Master\PakaianDinas\JenisPakaianDinasController;
use App\Http\Controllers\Master\PakaianDinas\SpesifikasiPakaianDinasController;
use App\Http\Controllers\Master\PakaianDinas\SubSpesifikasiPakaianDinasController;
use App\Http\Controllers\Master\PegawaiController;
use App\Http\Controllers\Master\SatkerController;
use App\Http\Controllers\Master\SssjController;
use App\Http\Controllers\Master\WilayahController;
use App\Http\Controllers\Monsakti\TransaksiAsetController;
use App\Http\Controllers\Pengadaan\PengadaanBarangJasaController;
use App\Http\Controllers\Pengadaan\RencanaPengadaanLangsungController;
use App\Http\Controllers\Pengadaan\RencanaPengadaanLangsungNonKontrakController;
use App\Http\Controllers\Pengaturan\BukuPanduanController;
use App\Http\Controllers\Pengaturan\MenuTema\MenuController;
use App\Http\Controllers\Pengaturan\MenuTema\TemaController;
use App\Http\Controllers\Pengaturan\PencadanganController;
use App\Http\Controllers\Pengguna\AktivitasController;
use App\Http\Controllers\Pengguna\LevelController;
use App\Http\Controllers\Pengguna\PenggunaController;
use App\Http\Controllers\Pengguna\ProfilController;
use App\Http\Controllers\Pengguna\ReviewController;
use App\Http\Controllers\Pengguna\SuperadminController;
use App\Http\Controllers\Referensi\BidangController;
use App\Http\Controllers\Referensi\GolonganController;
use App\Http\Controllers\Referensi\KelompokController;
use App\Http\Controllers\Referensi\SubkelompokController;
use App\Http\Controllers\Referensi\SubsubkelompokController;
use App\Http\Controllers\SumberDayaManusia\SdmPengadaanController;
use App\Http\Controllers\SumberDayaManusia\SdmPengelolaController;
use App\Http\Controllers\Bantuan\BantuanController as Bantuan;
use App\Http\Controllers\Bantuan\FaqController;
use App\Http\Controllers\Bantuan\FaqPenggunaController;
use App\Http\Controllers\Bantuan\HelpdeskController;
use App\Http\Controllers\Bantuan\KritikController;
use App\Http\Controllers\Bantuan\NotifikasiManualController;
use App\Http\Controllers\Bantuan\SurveyController;
use Illuminate\Support\Facades\Route;
use App\Http\Controllers\Bantuan\TopikController;
use App\Http\Controllers\Otentikasi\OtentikasiDuaFaktorController;
use App\Http\Controllers\Otentikasi\AuthController;

/*
|--------------------------------------------------------------------------
| Web Routes
|--------------------------------------------------------------------------
|
| Here is where you can register web routes for your application. These
| routes are loaded by the RouteServiceProvider and all of them will
| be assigned to the "web" middleware group. Make something great!
|
*/

// Login / Logout
Route::get('/auth/login', [AuthController::class, 'index'])->name('login');
Route::post('/auth/login', [AuthController::class, 'login']);
Route::get('/auth/logout', [AuthController::class, 'logout']);

// Rute verifikasi OTP 2FA (boleh tanpa Auth guard, cukup session 2fa:user:id)
Route::get('/2fa', [OtentikasiDuaFaktorController::class,'showVerifyForm'])->name('2fa.index');
Route::post('/2fa', [OtentikasiDuaFaktorController::class,'verify'])->name('2fa.verify');

// Setup dan verifikasi 2FA (harus sudah login credential biasa)
Route::middleware(['auth'])->group(function () {
    // Tampilkan QR & form OTP
    Route::get('/2fa/setup', [OtentikasiDuaFaktorController::class, 'showSetupForm'])->name('2fa.setup');

    // Proses enable 2FA: simpan secret, logout, terus redirect ke /2fa untuk OTP
    Route::post('/2fa/setup', [OtentikasiDuaFaktorController::class, 'enable'])->name('2fa.setup.post');
});

// Semua route setelah Login + 2FA berhasil wajib verified
Route::middleware(['auth', '2fa'])->group(function () {
    Route::post('/auth/changePassword', [AuthController::class, 'changePassword']);
    Route::post('/auth/changeUser', [AuthController::class, 'changeUser']);
    Route::post('/auth/resetPassword', [AuthController::class, 'resetPassword']);
    Route::get('/auth/changeRole/{roleId}', [AuthController::class, 'changeRole']);

    route::get('/pengguna/pengguna/gridData', [PenggunaController::class, 'gridData']);
    route::resource('pengguna/pengguna', PenggunaController::class);
    route::post('pengguna/profil/change-pp', [ProfilController::class, 'changePP']);
    route::resource('pengguna/profil', ProfilController::class);
    route::get('/profil', [ProfilController::class, 'index'])->name('profil');

    route::get('/pengguna/superadmin/gridData', [SuperadminController::class, 'gridData']);
    route::resource('/pengguna/superadmin', SuperadminController::class);

    route::get('/pengguna/level/gridData', [LevelController::class, 'gridData']);
    route::resource('/pengguna/level', LevelController::class);

    route::get('/pengguna/aktivitas/gridData', [AktivitasController::class, 'gridData']);
    route::resource('/pengguna/aktivitas', AktivitasController::class);

    route::get('/pengguna/review/gridData', [ReviewController::class, 'gridData']);
    route::resource('/pengguna/review', ReviewController::class);

    Route::get('/dashboard', [MainController::class, 'index']);
    Route::post('/dashboard', [MainController::class, 'dashboard']);
    Route::get('/getUserChangers', [MainController::class, 'getUserChangers']);
    Route::get('/dashboard/gridDataPegawaiDashboard', [MainController::class, 'gridDataPegawaiDashboard']);
    Route::post('/dashboarddetail', [MainController::class, 'dashboardDetail']);

    // Disable 2FA
    Route::post('/2fa/disable', [OtentikasiDuaFaktorController::class, 'disable'])->name('2fa.disable');
    // Disable 2FA oleh superadmin untuk user lain
    Route::post('/pengguna/nonaktifkan-2fa/{id}', [OtentikasiDuaFaktorController::class, 'disableBySuperadmin'])->middleware('auth')->name('2fa.disableBySuperadmin');

    Route::get('/searchPegawai/{nip}', [MainController::class, 'searchPegawai']);
    Route::get('/getNotif', [MainController::class, 'getNotif']);
    Route::get('/', [MainController::class, 'index']);

    Route::post('/asset/tanah/gridData', [TanahController::class, 'gridData']);
    Route::get('/asset/tanah/cetakLabel/{id}', [TanahController::class, 'cetakLabel']);
    Route::post('/asset/tanah/cetakExcel', [TanahController::class, 'cetakExcel']);
    Route::post('/asset/tanah/cetakPdf', [TanahController::class, 'cetakPdf']);
    Route::resource('/asset/tanah', TanahController::class);

    Route::post('/mapsdetail', [TanahController::class, 'mapsDetail']);
    Route::post('/mapssimpan', [TanahController::class, 'mapsSimpan']);

    Route::get('/asset/angkutan/gridData', [AngkutanController::class, 'gridData']);
    Route::get('/asset/angkutan/cetakLabel/{id}', [AngkutanController::class, 'cetakLabel']);
    Route::resource('/asset/angkutan', AngkutanController::class);
    Route::post('/asset/angkutan/cetakExcel', [AngkutanController::class, 'cetakExcel']);
    Route::post('/asset/angkutan/cetakPdf', [AngkutanController::class, 'cetakPdf']);

    Route::get('/asset/non_tik/gridData', [NonTikController::class, 'gridData']);
    Route::get('/asset/non_tik/cetakLabel/{id}', [NonTikController::class, 'cetakLabel']);
    Route::resource('/asset/non_tik', NonTikController::class);
    Route::post('/asset/non_tik/cetakExcel', [NonTikController::class, 'cetakExcel']);
    Route::post('/asset/non_tik/cetakPdf', [NonTikController::class, 'cetakPdf']);

    Route::get('/asset/tik/gridData', [TikController::class, 'gridData']);
    Route::get('/asset/tik/cetakLabel/{id}', [TikController::class, 'cetakLabel']);
    Route::resource('/asset/tik', TikController::class);
    Route::post('/asset/tik/cetakExcel', [TikController::class, 'cetakExcel']);
    Route::post('/asset/tik/cetakPdf', [TikController::class, 'cetakPdf']);

    Route::get('/master/satker/gridData', [SatkerController::class, 'gridData']);
    Route::resource('/master/satker', SatkerController::class);

    Route::get('/bmn/ijin/gridData', [IjinController::class, 'gridData']);
    Route::get('/bmn/ijinpegawai/gridData', [IjinPegawaiController::class, 'gridData']);
    Route::get('/bmn/ijinmonitor/gridData', [IjinMonitorController::class, 'gridData']);

    Route::get('/bmn/ijin/gridDataPegawai/{pengajuan_id}', [IjinController::class, 'gridDataPegawai']);
    Route::get('/bmn/ijinpegawai/gridDataPegawai/{pengajuan_id}', [IjinPegawaiController::class, 'gridDataPegawai']);
    Route::get('/bmn/ijinmonitor/gridDataPegawai/{pengajuan_id}', [IjinController::class, 'gridDataPegawai']);
    Route::get('/bmn/ijin/gridDataMsPegawai/{pengajuan_id}', [IjinController::class, 'gridDataMsPegawai']);
    Route::get('/bmn/ijinpegawai/gridDataMsPegawai/{pengajuan_id}', [IjinPegawaiController::class, 'gridDataMsPegawai']);
    Route::get('/bmn/ijinmonitor/gridDataMsPegawai/{pengajuan_id}', [IjinController::class, 'gridDataMsPegawai']);
    Route::get('/bmn/ijin/gridDataAset/{pengajuan_id}', [IjinController::class, 'gridDataAset']);
    Route::get('/bmn/ijinpegawai/gridDataAset/{pengajuan_id}', [IjinPegawaiController::class, 'gridDataAset']);
    Route::get('/bmn/ijinmonitor/gridDataAset/{pengajuan_id}', [IjinController::class, 'gridDataAset']);
    Route::post('/bmn/ijin/savePengajuan', [IjinController::class, 'savePengajuan']);
    Route::post('/bmn/ijinpegawai/savePengajuan', [IjinPegawaiController::class, 'savePengajuan']);
    Route::post('/bmn/ijinmonitor/savePengajuan', [IjinController::class, 'savePengajuan']);
    Route::post('/bmn/ijin/savePegawai', [IjinController::class, 'savePegawai']);
    Route::post('/bmn/ijinpegawai/savePegawai', [IjinPegawaiController::class, 'savePegawai']);
    Route::post('/bmn/ijinmonitor/savePegawai', [IjinController::class, 'savePegawai']);
    Route::post('/bmn/ijin/saveUserPegawai', [IjinController::class, 'saveUserPegawai']);
    Route::post('/bmn/ijinmonitor/saveUserPegawai', [IjinController::class, 'saveUserPegawai']);
    Route::delete('/bmn/ijin/deletePegawai/{id}', [IjinController::class, 'deletePegawai']);
    Route::delete('/bmn/ijinpegawai/deletePegawai/{id}', [IjinPegawaiController::class, 'deletePegawai']);
    Route::delete('/bmn/ijinmonitor/deletePegawai/{id}', [IjinController::class, 'deletePegawai']);
    Route::resource('/bmn/ijin', IjinController::class);
    Route::resource('/bmn/ijinpegawai', IjinPegawaiController::class);
    Route::resource('/bmn/ijinmonitor', IjinMonitorController::class);
    Route::get('/bmn/ijin/cetak/{id}', [IjinController::class, 'cetak']);
    Route::get('/bmn/ijinmonitor/cetak/{id}', [IjinController::class, 'cetak']);

    Route::get('/master/wilayah/gridData', [WilayahController::class, 'gridData']);
    Route::resource('/master/wilayah', WilayahController::class);

    Route::post('/master/pegawai/gridData', [PegawaiController::class, 'gridData']);
    Route::post('/master/pegawai/cetakExcel', [PegawaiController::class, 'cetakExcel']);
    Route::post('/master/pegawai/cetakPdf', [PegawaiController::class, 'cetakPdf']);
    Route::resource('/master/pegawai', PegawaiController::class);

    Route::get('/master/sssj/gridData', [SssjController::class, 'gridData']);
    Route::resource('/master/sssj', SssjController::class);

    Route::get('/asset/alat_besar/gridData', [AlatBesarController::class, 'gridData']);
    Route::get('/asset/alat_besar/cetakLabel/{id}', [AlatBesarController::class, 'cetakLabel']);
    Route::resource('/asset/alat_besar', AlatBesarController::class);
    Route::post('/asset/alat_besar/cetakExcel', [AlatBesarController::class, 'cetakExcel']);
    Route::post('/asset/alat_besar/cetakPdf', [AlatBesarController::class, 'cetakPdf']);

    Route::get('/asset/senjata/gridData', [SenjataController::class, 'gridData']);
    Route::get('/asset/senjata/cetakLabel/{id}', [SenjataController::class, 'cetakLabel']);
    Route::resource('/asset/senjata', SenjataController::class);
    Route::post('/asset/senjata/cetakExcel', [SenjataController::class, 'cetakExcel']);
    Route::post('/asset/senjata/cetakPdf', [SenjataController::class, 'cetakPdf']);

    Route::get('/asset/gedung/gridData', [GedungController::class, 'gridData']);
    Route::get('/asset/gedung/cetakLabel/{id}', [GedungController::class, 'cetakLabel']);
    Route::resource('/asset/gedung', GedungController::class);
    Route::post('/asset/gedung/cetakExcel', [GedungController::class, 'cetakExcel']);
    Route::post('/asset/gedung/cetakPdf', [GedungController::class, 'cetakPdf']);

    Route::get('/asset/rumah/gridData', [RumahController::class, 'gridData']);
    Route::get('/asset/rumah/cetakLabel/{id}', [RumahController::class, 'cetakLabel']);
    Route::resource('/asset/rumah', RumahController::class);
    Route::post('/asset/rumah/cetakExcel', [RumahController::class, 'cetakExcel']);
    Route::post('/asset/rumah/cetakPdf', [RumahController::class, 'cetakPdf']);

    Route::get('/asset/jalan_jembatan/gridData', [JalanJembatanController::class, 'gridData']);
    Route::get('/asset/jalan_jembatan/cetakLabel/{id}', [JalanJembatanController::class, 'cetakLabel']);
    Route::resource('/asset/jalan_jembatan', JalanJembatanController::class);
    Route::post('/asset/jalan_jembatan/cetakExcel', [JalanJembatanController::class, 'cetakExcel']);
    Route::post('/asset/jalan_jembatan/cetakPdf', [JalanJembatanController::class, 'cetakPdf']);

    Route::get('/asset/bangunan_air/gridData', [BangunanAirController::class, 'gridData']);
    Route::get('/asset/bangunan_air/cetakLabel/{id}', [BangunanAirController::class, 'cetakLabel']);
    Route::resource('/asset/bangunan_air', BangunanAirController::class);
    Route::post('/asset/bangunan_air/cetakExcel', [BangunanAirController::class, 'cetakExcel']);
    Route::post('/asset/bangunan_air/cetakPdf', [BangunanAirController::class, 'cetakPdf']);

    Route::get('/asset/jaringan/gridData', [JaringanController::class, 'gridData']);
    Route::get('/asset/jaringan/cetakLabel/{id}', [JaringanController::class, 'cetakLabel']);
    Route::resource('/asset/jaringan', JaringanController::class);
    Route::post('/asset/jaringan/cetakExcel', [JaringanController::class, 'cetakExcel']);
    Route::post('/asset/jaringan/cetakPdf', [JaringanController::class, 'cetakPdf']);

    Route::get('/asset/konstruksi/gridData', [KonstruksiController::class, 'gridData']);
    Route::get('/asset/konstruksi/cetakLabel/{id}', [KonstruksiController::class, 'cetakLabel']);
    Route::resource('/asset/konstruksi', KonstruksiController::class);
    Route::post('/asset/konstruksi/cetakExcel', [KonstruksiController::class, 'cetakExcel']);
    Route::post('/asset/konstruksi/cetakPdf', [KonstruksiController::class, 'cetakPdf']);

    Route::get('/asset/lain/gridData', [LainController::class, 'gridData']);
    Route::get('/asset/lain/cetakLabel/{id}', [LainController::class, 'cetakLabel']);
    Route::resource('/asset/lain', LainController::class);
    Route::post('/asset/lain/cetakExcel', [LainController::class, 'cetakExcel']);
    Route::post('/asset/lain/cetakPdf', [LainController::class, 'cetakPdf']);

    Route::get('/asset/renovasi/gridData', [RenovasiController::class, 'gridData']);
    Route::get('/asset/renovasi/cetakLabel/{id}', [RenovasiController::class, 'cetakLabel']);
    Route::resource('/asset/renovasi', RenovasiController::class);
    Route::post('/asset/renovasi/cetakExcel', [RenovasiController::class, 'cetakExcel']);
    Route::post('/asset/renovasi/cetakPdf', [RenovasiController::class, 'cetakPdf']);

    Route::get('/asset/wujud/gridData', [WujudController::class, 'gridData']);
    Route::get('/asset/wujud/cetakLabel/{id}', [WujudController::class, 'cetakLabel']);
    Route::resource('/asset/wujud', WujudController::class);
    Route::post('/asset/wujud/cetakExcel', [WujudController::class, 'cetakExcel']);
    Route::post('/asset/wujud/cetakPdf', [WujudController::class, 'cetakPdf']);

    Route::resource('/asset/sebaran', SebaranController::class);
    Route::post('/asset/sebaran/getsatkerkoordinat', [SebaranController::class, 'getSatkerKoordinat']);
    Route::post('/asset/sebaran/detailsatker', [SebaranController::class, 'detailSatker']);

    Route::get('/asset/qr-code/gridData', [QrCodeController::class, 'gridData']);
    Route::post('/asset/qr-code/cetakLabel', [QrCodeController::class, 'cetakLabel']);
    Route::resource('/asset/qr-code', QrCodeController::class);

    Route::get('/bmn/pemanfaatan/pemanfaatansk/gridData', [PemanfaatanSk::class, 'gridData']);
    Route::post('/bmn/pemanfaatan/pemanfaatansk/uploadSk', [PemanfaatanSk::class, 'uploadSk']);
    Route::resource('/bmn/pemanfaatan/pemanfaatansk', PemanfaatanSk::class);

    Route::get('/bmn/pemanfaatan/pemanfaatanmonitor/gridData', [PemanfaatanMonitor::class, 'gridData']);
    Route::resource('/bmn/pemanfaatan/pemanfaatanmonitor', PemanfaatanMonitor::class);

    // Route::get('/bmn/penghapusan/penghapusansk/gridData', [PenghapusanSk::class, 'gridData']);
    // Route::resource('/bmn/penghapusan/penghapusansk', PenghapusanSk::class);

    Route::delete('/bmn/penghapusan/penghapusansk/deleteFilelain/{id}', [PermohonanPenghapusanBmnController::class, 'deleteFilelain']);
    Route::get('/bmn/penghapusan/penghapusansk/downloadZip/{pengajuan_id}', [PermohonanPenghapusanBmnController::class, 'downloadZip']);
    Route::get('/bmn/penghapusan/penghapusansk/gridDataFotoCopy/{pengajuan_id}', [PermohonanPenghapusanBmnSkController::class, 'gridDataFotoCopy']);
    Route::get('/bmn/penghapusan/penghapusansk/gridDataFoto/{pengajuan_id}', [PermohonanPenghapusanBmnSkController::class, 'gridDataFoto']);
    Route::get('/bmn/penghapusan/penghapusansk/gridData', [PermohonanPenghapusanBmnSkController::class, 'gridData']);
    Route::get('/bmn/penghapusan/penghapusansk/gridDataPermohonan', [PermohonanPenghapusanBmnSkController::class, 'gridDataPermohonan']);
    Route::post('/bmn/penghapusan/penghapusansk/saveFotocopy', [PermohonanPenghapusanBmnSkController::class, 'saveFotocopy']);
    Route::post('/bmn/penghapusan/penghapusansk/saveFoto', [PermohonanPenghapusanBmnSkController::class, 'saveFoto']);
    Route::delete('/bmn/penghapusan/penghapusansk/deleteFotocopy/{id}', [PermohonanPenghapusanBmnSkController::class, 'deleteFotocopy']);
    Route::delete('/bmn/penghapusan/penghapusansk/deleteFoto/{id}', [PermohonanPenghapusanBmnSkController::class, 'deleteFoto']);
    Route::post('/bmn/penghapusan/penghapusansk/savePengajuan', [PermohonanPenghapusanBmnSkController::class, 'savePengajuan']);
    Route::resource('/bmn/penghapusan/penghapusansk', PermohonanPenghapusanBmnSkController::class);

    Route::get('/bmn/penghapusan/pemindahtanganan/downloadZip/{pengajuan_id}', [PermohonanPenghapusanBmnController::class, 'downloadZip']);
    Route::get('/bmn/penghapusan/pemindahtanganan/gridDataFotoCopy/{pengajuan_id}', [PermohonanPenghapusanBmnController::class, 'gridDataFotoCopy']);
    Route::get('/bmn/penghapusan/pemindahtanganan/gridDataFoto/{pengajuan_id}', [PermohonanPenghapusanBmnController::class, 'gridDataFoto']);
    Route::get('/bmn/penghapusan/pemindahtanganan/gridData', [PermohonanPenghapusanBmnController::class, 'gridData']);
    Route::post('/bmn/penghapusan/pemindahtanganan/saveFotocopy', [PermohonanPenghapusanBmnController::class, 'saveFotocopy']);
    Route::post('/bmn/penghapusan/pemindahtanganan/saveFoto', [PermohonanPenghapusanBmnController::class, 'saveFoto']);
    Route::delete('/bmn/penghapusan/pemindahtanganan/deleteFotocopy/{id}', [PermohonanPenghapusanBmnController::class, 'deleteFotocopy']);
    Route::delete('/bmn/penghapusan/pemindahtanganan/deleteFoto/{id}', [PermohonanPenghapusanBmnController::class, 'deleteFoto']);
    Route::post('/bmn/penghapusan/pemindahtanganan/savePengajuan', [PermohonanPenghapusanBmnController::class, 'savePengajuan']);
    Route::resource('/bmn/penghapusan/pemindahtanganan', PermohonanPenghapusanBmnController::class);

    Route::get('/bmn/penghapusan/pemusnahan/downloadZip/{pengajuan_id}', [PermohonanPenghapusanBmnController::class, 'downloadZip']);
    Route::get('/bmn/penghapusan/pemusnahan/gridDataFotoCopy/{pengajuan_id}', [PermohonanPenghapusanBmnController::class, 'gridDataFotoCopy']);
    Route::get('/bmn/penghapusan/pemusnahan/gridDataFoto/{pengajuan_id}', [PermohonanPenghapusanBmnController::class, 'gridDataFoto']);
    Route::get('/bmn/penghapusan/pemusnahan/gridData', [PermohonanPenghapusanBmnController::class, 'gridData']);
    Route::post('/bmn/penghapusan/pemusnahan/saveFotocopy', [PermohonanPenghapusanBmnController::class, 'saveFotocopy']);
    Route::post('/bmn/penghapusan/pemusnahan/saveFoto', [PermohonanPenghapusanBmnController::class, 'saveFoto']);
    Route::delete('/bmn/penghapusan/pemusnahan/deleteFotocopy/{id}', [PermohonanPenghapusanBmnController::class, 'deleteFotocopy']);
    Route::delete('/bmn/penghapusan/pemusnahan/deleteFoto/{id}', [PermohonanPenghapusanBmnController::class, 'deleteFoto']);
    Route::post('/bmn/penghapusan/pemusnahan/savePengajuan', [PermohonanPenghapusanBmnController::class, 'savePengajuan']);
    Route::resource('/bmn/penghapusan/pemusnahan', PermohonanPenghapusanBmnController::class);

    Route::get('/bmn/penghapusan/pemusnahan/downloadZip/{pengajuan_id}', [PermohonanPenghapusanBmnController::class, 'downloadZip']);
    Route::get('/bmn/penghapusan/sebablain/gridDataFotoCopy/{pengajuan_id}', [PermohonanPenghapusanBmnController::class, 'gridDataFotoCopy']);
    Route::get('/bmn/penghapusan/sebablain/gridDataFoto/{pengajuan_id}', [PermohonanPenghapusanBmnController::class, 'gridDataFoto']);
    Route::get('/bmn/penghapusan/sebablain/gridData', [PermohonanPenghapusanBmnController::class, 'gridData']);
    Route::post('/bmn/penghapusan/sebablain/saveFotocopy', [PermohonanPenghapusanBmnController::class, 'saveFotocopy']);
    Route::post('/bmn/penghapusan/sebablain/saveFoto', [PermohonanPenghapusanBmnController::class, 'saveFoto']);
    Route::delete('/bmn/penghapusan/sebablain/deleteFotocopy/{id}', [PermohonanPenghapusanBmnController::class, 'deleteFotocopy']);
    Route::delete('/bmn/penghapusan/sebablain/deleteFoto/{id}', [PermohonanPenghapusanBmnController::class, 'deleteFoto']);
    Route::post('/bmn/penghapusan/sebablain/savePengajuan', [PermohonanPenghapusanBmnController::class, 'savePengajuan']);
    Route::resource('/bmn/penghapusan/sebablain', PermohonanPenghapusanBmnController::class);

    Route::delete('/bmn/penghapusan/penghapusanmonitor/deleteFilelain/{id}', [PermohonanPenghapusanBmnMonitorController::class, 'deleteFilelain']);
    Route::get('/bmn/penghapusan/penghapusanmonitor/gridDataFilelain/{pengajuan_id}', [PermohonanPenghapusanBmnMonitorController::class, 'gridDataFilelain']);
    Route::get('/bmn/penghapusan/penghapusanmonitor/gridDataFotoCopy/{pengajuan_id}', [PermohonanPenghapusanBmnMonitorController::class, 'gridDataFotoCopy']);
    Route::get('/bmn/penghapusan/penghapusanmonitor/gridDataFoto/{pengajuan_id}', [PermohonanPenghapusanBmnMonitorController::class, 'gridDataFoto']);
    Route::get('/bmn/penghapusan/penghapusanmonitor/gridData', [PermohonanPenghapusanBmnMonitorController::class, 'gridData']);
    Route::get('/bmn/penghapusan/penghapusanmonitor/gridDataPermohonan', [PermohonanPenghapusanBmnMonitorController::class, 'gridDataPermohonan']);
    Route::post('/bmn/penghapusan/penghapusanmonitor/savePengajuan', [PermohonanPenghapusanBmnMonitorController::class, 'savePengajuan']);
    Route::resource('/bmn/penghapusan/penghapusanmonitor', PermohonanPenghapusanBmnMonitorController::class);
    Route::post('/bmn/penghapusan/penghapusanmonitor/saveLain', [PermohonanPenghapusanBmnMonitorController::class, 'saveLain']);

    Route::get('/bmn/penghapusan/persetujuanmonitor/gridData', [PersetujuanBmnMonitorController::class, 'gridData']);
    Route::delete('/bmn/penghapusan/persetujuanmonitor/deleteFilelain/{id}', [PersetujuanBmnMonitorController::class, 'deleteFilelain']);
    Route::get('/bmn/penghapusan/persetujuanmonitor/gridDataFilelain/{pengajuan_id}', [PersetujuanBmnMonitorController::class, 'gridDataFilelain']);
    Route::get('/bmn/penghapusan/persetujuanmonitor/gridDataFotoCopy/{pengajuan_id}', [PersetujuanBmnMonitorController::class, 'gridDataFotoCopy']);
    Route::get('/bmn/penghapusan/persetujuanmonitor/gridDataFoto/{pengajuan_id}', [PersetujuanBmnMonitorController::class, 'gridDataFoto']);
    Route::delete('/bmn/penghapusan/persetujuanmonitor/deleteFilelain/{id}', [PersetujuanBmnMonitorController::class, 'deleteFilelain']);
    Route::post('/bmn/penghapusan/persetujuanmonitor/saveLain', [PersetujuanBmnMonitorController::class, 'saveLain']);
    Route::post('/bmn/penghapusan/persetujuanmonitor/savePengajuan', [PersetujuanBmnMonitorController::class, 'savePengajuan']);
    Route::resource('/bmn/penghapusan/persetujuanmonitor', PersetujuanBmnMonitorController::class);

    // Route::get('/bmn/penghapusan/penghapusanmonitor/gridData', [PenghapusanMonitor::class, 'gridData']);
    // Route::resource('/bmn/penghapusan/penghapusanmonitor', PenghapusanMonitor::class);

    Route::get('/bmn/pencabutan/pencabutan/gridData', [Pencabutan::class, 'gridData']);
    Route::resource('/bmn/pencabutan/pencabutan', Pencabutan::class);

    Route::post('/sdm/sdmpengadaan/gridData', [SdmPengadaanController::class, 'gridData']);
    Route::resource('/sdm/sdmpengadaan', SdmPengadaanController::class);

    Route::post('/sdm/sdmpengelola/gridData', [SdmPengelolaController::class, 'gridData']);
    Route::resource('/sdm/sdmpengelola', SdmPengelolaController::class);

    Route::post('/asset-tik/bmn-tik/gridData', [BmnTikController::class, 'gridData']);
    Route::resource('/asset-tik/bmn-tik', BmnTikController::class);

    Route::post('/asset-tik/hakcipta/gridData', [HakciptaController::class, 'gridData']);
    Route::resource('/asset-tik/hakcipta', HakciptaController::class);

    Route::post('/asset-tik/langganan/gridData', [LanggananController::class, 'gridData']);
    Route::resource('/asset-tik/langganan', LanggananController::class);

    Route::get('/support/helpdesk/gridData', [HelpdeskController::class, 'gridData']);
    Route::post('/support/helpdesk/saveKomentar', [HelpdeskController::class, 'saveKomentar']);
    Route::resource('/support/helpdesk', HelpdeskController::class);

    Route::get('/support/kritik/gridData', [KritikController::class, 'gridData']);
    Route::resource('/support/kritik', KritikController::class);

    Route::get('/support/faq/gridData', [FaqController::class, 'gridData']);
    Route::resource('/support/faq', FaqController::class);

    Route::get('/support/buku-panduan', [\App\Http\Controllers\Bantuan\BukuPanduanController::class, 'index']);
    Route::get('/support/bantuan/gridData', [Bantuan::class, 'gridData']);
    Route::resource('/support/bantuan', Bantuan::class);

    Route::resource('/support/faquser', FaqPenggunaController::class);

    Route::get('/support/survey/gridData', [SurveyController::class, 'gridData']);
    Route::post('/support/survey/saveKomentar', [SurveyController::class, 'saveKomentar']);
    Route::resource('/support/survey', SurveyController::class);

    Route::get('/support/notifikasi-manual/gridData', [NotifikasiManualController::class, 'gridData']);
    Route::resource('/support/notifikasi-manual', NotifikasiManualController::class);

    Route::get('/bmn/hibah/hibah/gridData', [Hibah::class, 'gridData']);
    Route::resource('/bmn/hibah/hibah', Hibah::class);

    Route::get('/bmn/hibah/hibahmonitor/gridData', [HibahMonitor::class, 'gridData']);
    Route::resource('/bmn/hibah/hibahmonitor', HibahMonitor::class);

    Route::get('/bmn/pnbp/pnbp/gridData', [Pnbp::class, 'gridData']);
    Route::resource('/bmn/pnbp/pnbp', Pnbp::class);

    Route::get('/bmn/jasa/jasa/gridData', [Jasa::class, 'gridData']);
    Route::resource('/bmn/jasa/jasa', Jasa::class);

    Route::get('/bmn/perawatan/perawatan/gridData', [Perawatan::class, 'gridData']);
    Route::resource('/bmn/perawatan/perawatan', Perawatan::class);

    Route::get('/bmn/perawatan/perawatanmonitor/gridData', [PerawatanMonitor::class, 'gridData']);
    Route::resource('/bmn/perawatan/perawatanmonitor', PerawatanMonitor::class);

    //

    Route::get('/analisis-kebutuhan/pakaian-dinas/pengajuan/gridData', [PengajuanPakaianDinas::class, 'gridData']);
    Route::get('/analisis-kebutuhan/pakaian-dinas/pengajuan/gridDataSatker', [PengajuanPakaianDinas::class, 'gridDataSatker']);
    Route::get('/analisis-kebutuhan/pakaian-dinas/pengajuan/{pengajuanId}/list-satker', [PengajuanPakaianDinas::class, 'listSatker']);
    Route::post('/analisis-kebutuhan/pakaian-dinas/pengajuan/savePengajuan', [PengajuanPakaianDinas::class, 'savePengajuan']);
    Route::post('/analisis-kebutuhan/pakaian-dinas/pengajuan/validatorAction', [PengajuanPakaianDinas::class, 'validatorAction']);
    Route::resource('/analisis-kebutuhan/pakaian-dinas/pengajuan', PengajuanPakaianDinas::class);
    Route::get('/analisis-kebutuhan/pakaian-dinas/laporan/getPengadaanByTahun/{tahun}', [PengajuanPakaianDinasLaporan::class, 'getPengadaanByTahun']);
    Route::get('/analisis-kebutuhan/pakaian-dinas/laporan/cetak', [PengajuanPakaianDinasLaporan::class, 'cetak']);
    Route::resource('/analisis-kebutuhan/pakaian-dinas/laporan', PengajuanPakaianDinasLaporan::class);
    Route::resource('/analisis-kebutuhan/pakaian-dinas/ukuran-pakaian-pegawai', UkuranPakaianPegawaiController::class);

    Route::get('/analisis-kebutuhan/bmn/pengajuan/gridData', [PengajuanBmn::class, 'gridData']);
    Route::get('/analisis-kebutuhan/bmn/pengajuan/gridDataSatker', [PengajuanBmn::class, 'gridDataSatker']);
    Route::get('/analisis-kebutuhan/bmn/pengajuan/gridDataBarang/{id}', [PengajuanBmn::class, 'gridDataBarang']);
    Route::post('/analisis-kebutuhan/bmn/pengajuan/saveBarang', [PengajuanBmn::class, 'saveBarang']);
    Route::post('/analisis-kebutuhan/bmn/pengajuan/savePengajuan', [PengajuanBmn::class, 'savePengajuan']);
    Route::get('/analisis-kebutuhan/bmn/pengajuan/{pengajuanId}/list-satker', [PengajuanBmn::class, 'listSatker']);
    Route::delete('/analisis-kebutuhan/bmn/pengajuan/deleteBarang/{id}', [PengajuanBmn::class, 'deleteBarang']);
    Route::resource('/analisis-kebutuhan/bmn/pengajuan', PengajuanBmn::class);

    Route::get('/analisis-kebutuhan/bmn/daftar-kebutuhan-bmn/gridData', [DaftarKebutuhanBmnController::class, 'gridData']);
    Route::get('/analisis-kebutuhan/bmn/daftar-kebutuhan-bmn/gridDataSatker', [DaftarKebutuhanBmnController::class, 'gridDataSatker']);
    Route::get('/analisis-kebutuhan/bmn/daftar-kebutuhan-bmn/gridDataBarang/{id}', [DaftarKebutuhanBmnController::class, 'gridDataBarang']);
    Route::resource('/analisis-kebutuhan/bmn/daftar-kebutuhan-bmn', DaftarKebutuhanBmnController::class);
    Route::post('/analisis-kebutuhan/bmn/daftar-kebutuhan-bmn/saveBarang', [DaftarKebutuhanBmnController::class, 'saveBarang']);
    Route::post('/analisis-kebutuhan/bmn/daftar-kebutuhan-bmn/savePengajuan', [DaftarKebutuhanBmnController::class, 'savePengajuan']);
    Route::delete('/analisis-kebutuhan/bmn/daftar-kebutuhan-bmn/deleteBarang/{id}', [DaftarKebutuhanBmnController::class, 'deleteBarang']);

    Route::get('/analisis-kebutuhan/bmn/analisis-kelayakan/gridDataBarang/{id}', [AnalisisKelayakanController::class, 'gridDataBarang']);
    Route::get('/analisis-kebutuhan/bmn/analisis-kelayakan/gridData', [AnalisisKelayakanController::class, 'gridData']);
    Route::get('/analisis-kebutuhan/bmn/analisis-kelayakan/gridDataSatker', [AnalisisKelayakanController::class, 'gridDataSatker']);
    Route::get('/analisis-kebutuhan/bmn/analisis-kelayakan/gridDataSatkerAset', [AnalisisKelayakanController::class, 'gridDataSatkerAset']);
    Route::resource('/analisis-kebutuhan/bmn/analisis-kelayakan', AnalisisKelayakanController::class);

    Route::get('/analisis-kebutuhan/bmn/penyusunan-prioritas/gridDataBarang/{id}', [PenyusunanPrioritasController::class, 'gridDataBarang']);
    Route::get('/analisis-kebutuhan/bmn/penyusunan-prioritas/gridData', [PenyusunanPrioritasController::class, 'gridData']);
    Route::get('/analisis-kebutuhan/bmn/penyusunan-prioritas/gridDataSatker', [PenyusunanPrioritasController::class, 'gridDataSatker']);
    Route::get('/analisis-kebutuhan/bmn/penyusunan-prioritas/gridDataSatkerAset', [PenyusunanPrioritasController::class, 'gridDataSatkerAset']);
    Route::post('/analisis-kebutuhan/bmn/penyusunan-prioritas/cetakExcel', [PenyusunanPrioritasController::class, 'cetakExcel']);
    Route::resource('/analisis-kebutuhan/bmn/penyusunan-prioritas', PenyusunanPrioritasController::class);
    Route::post('/analisis-kebutuhan/bmn/penyusunan-prioritas/savePrioritas', [PenyusunanPrioritasController::class, 'savePrioritas']);

    Route::get('/analisis-kebutuhan/bmn/cetak-dokumen/gridDataBarang/{id}', [PenyusunanPrioritasController::class, 'gridDataBarang']);
    Route::get('/analisis-kebutuhan/bmn/cetak-dokumen/gridData', [PenyusunanPrioritasController::class, 'gridData']);
    Route::get('/analisis-kebutuhan/bmn/cetak-dokumen/gridDataSatker', [PenyusunanPrioritasController::class, 'gridDataSatker']);
    Route::get('/analisis-kebutuhan/bmn/cetak-dokumen/gridDataSatkerAset', [PenyusunanPrioritasController::class, 'gridDataSatkerAset']);
    Route::post('/analisis-kebutuhan/bmn/cetak-dokumen/cetakExcel', [PenyusunanPrioritasController::class, 'cetakExcel']);
    Route::resource('/analisis-kebutuhan/bmn/cetak-dokumen', PenyusunanPrioritasController::class);
    Route::post('/analisis-kebutuhan/bmn/cetak-dokumen/savePrioritas', [PenyusunanPrioritasController::class, 'savePrioritas']);
    Route::get('/analisis-kebutuhan/bmn/cetak-dokumen/cetak/{id}', [PenyusunanPrioritasController::class, 'cetak']);

    //END ANALISIS KEBUTUHAN

    Route::get('/pengadaan/rencanapengadaanlangsung/gridData', [RencanaPengadaanLangsungController::class, 'gridData']);
    Route::resource('/pengadaan/rencanapengadaanlangsung', RencanaPengadaanLangsungController::class);

    Route::get('/pengadaan/pengadaanlangsung-nonkontrak/gridData', [RencanaPengadaanLangsungNonKontrakController::class, 'gridData']);
    Route::resource('/pengadaan/pengadaanlangsung-nonkontrak', RencanaPengadaanLangsungNonKontrakController::class);

    Route::post('/pengadaan/barang-jasa/gridDataAnggaran', [PengadaanBarangJasaController::class, 'gridDataAnggaran']);
    Route::get('/pengadaan/barang-jasa/gridData', [PengadaanBarangJasaController::class, 'gridData']);
    Route::post('/pengadaan/barang-jasa/saveHps', [PengadaanBarangJasaController::class, 'saveHps']);
    Route::post('/pengadaan/barang-jasa/saveSkppbj', [PengadaanBarangJasaController::class, 'saveSkppbj']);
    Route::post('/pengadaan/barang-jasa/saveSpk', [PengadaanBarangJasaController::class, 'saveSpk']);
    Route::post('/pengadaan/barang-jasa/saveRingkasan', [PengadaanBarangJasaController::class, 'saveRingkasan']);
    Route::post('/pengadaan/barang-jasa/saveKontrak', [PengadaanBarangJasaController::class, 'saveKontrak']);
    Route::post('/pengadaan/barang-jasa/saveBast', [PengadaanBarangJasaController::class, 'saveBast']);
    Route::post('/pengadaan/barang-jasa/saveBapp', [PengadaanBarangJasaController::class, 'saveBapp']);
    Route::post('/pengadaan/barang-jasa/saveNodis', [PengadaanBarangJasaController::class, 'saveNodis']);
    Route::get('/pengadaan/barang-jasa/cetakHps/{id}', [PengadaanBarangJasaController::class, 'cetakHps']);
    Route::get('/pengadaan/barang-jasa/cetakSkppbj/{id}', [PengadaanBarangJasaController::class, 'cetakSkppbj']);
    Route::get('/pengadaan/barang-jasa/cetakSpk/{id}', [PengadaanBarangJasaController::class, 'cetakSpk']);
    Route::get('/pengadaan/barang-jasa/cetakRingkasan/{id}', [PengadaanBarangJasaController::class, 'cetakRingkasan']);
    Route::get('/pengadaan/barang-jasa/cetakKontrak/{id}', [PengadaanBarangJasaController::class, 'cetakKontrak']);
    Route::get('/pengadaan/barang-jasa/cetakBast/{id}', [PengadaanBarangJasaController::class, 'cetakBast']);
    Route::get('/pengadaan/barang-jasa/cetakBapp/{id}', [PengadaanBarangJasaController::class, 'cetakBapp']);
    Route::get('/pengadaan/barang-jasa/cetakNodis/{id}', [PengadaanBarangJasaController::class, 'cetakNodis']);
    Route::resource('/pengadaan/barang-jasa', PengadaanBarangJasaController::class);

    Route::get('/master/pakaian-dinas/jenis-pakaian-dinas/gridData', [JenisPakaianDinasController::class, 'gridData']);
    Route::resource('/master/pakaian-dinas/jenis-pakaian-dinas', JenisPakaianDinasController::class);

    Route::get('/master/pakaian-dinas/spesifikasi-pakaian-dinas/gridData', [SpesifikasiPakaianDinasController::class, 'gridData']);
    Route::resource('/master/pakaian-dinas/spesifikasi-pakaian-dinas', SpesifikasiPakaianDinasController::class);

    Route::get('/master/pakaian-dinas/subspesifikasi-pakaian-dinas/gridData', [SubSpesifikasiPakaianDinasController::class, 'gridData']);
    Route::resource('/master/pakaian-dinas/subspesifikasi-pakaian-dinas', SubSpesifikasiPakaianDinasController::class);

    Route::get('/pengaturan/menu/gridData', [MenuController::class, 'gridData']);
    Route::resource('/pengaturan/menu', MenuController::class);

    Route::resource('/pengaturan/tema', TemaController::class);

    Route::get('/pengaturan/pencadangan/gridData', [PencadanganController::class, 'gridData']);
    Route::post('/pengaturan/pencadangan/restore', [PencadanganController::class, 'restore']);
    Route::resource('/pengaturan/pencadangan', PencadanganController::class);

    Route::get('/pengaturan/buku-panduan/gridData', [BukuPanduanController::class, 'gridData']);
    Route::resource('/pengaturan/buku-panduan', BukuPanduanController::class);

    Route::get('/pengaturan/integrasi-data/gridData', [IntegrasiDataController::class, 'gridData']);
    Route::resource('/pengaturan/integrasi-data', IntegrasiDataController::class);

    Route::get('/referensi/golongan/gridData', [GolonganController::class, 'gridData']);
    Route::resource('/referensi/golongan', GolonganController::class);

    Route::get('/referensi/bidang/gridData', [BidangController::class, 'gridData']);
    Route::resource('/referensi/bidang', BidangController::class);

    Route::get('/referensi/kelompok/gridData', [KelompokController::class, 'gridData']);
    Route::resource('/referensi/kelompok', KelompokController::class);

    Route::get('/referensi/subkelompok/gridData', [SubkelompokController::class, 'gridData']);
    Route::resource('/referensi/subkelompok', SubkelompokController::class);

    Route::get('/referensi/subsubkelompok/gridData', [SubsubkelompokController::class, 'gridData']);
    Route::resource('/referensi/subsubkelompok', SubsubkelompokController::class);

    Route::get('/log-integrasi/gridData', [LogIntegrasiController::class, 'gridData']);
    Route::resource('/log-integrasi', LogIntegrasiController::class);

    Route::get('/monsakti/transaksi-aset/gridData', [TransaksiAsetController::class, 'gridData']);

    Route::get('/support/topik/gridData', [TopikController::class, 'gridData']);
    Route::resource('/support/topik', TopikController::class);

    Route::get('/analisis-kebutuhan/pakaian-dinas/laporan/cetak', [PengajuanPakaianDinasLaporan::class, 'cetak']);

});
