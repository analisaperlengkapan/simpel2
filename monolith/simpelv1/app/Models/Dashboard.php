<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class Dashboard extends Model
{
    use HasFactory;

    public function getdata($type, $level = '', $wilayah = null, $satker = null)
    {
        // $where = $tahun ? "and date_part('year', a.tgl_perolehan) <=  {$tahun}" : '';
        $where = '';
        $roleSatker = session('userData.current_role.ms_satker_id_keu');
        if ($roleSatker != '') {
            $where .= " and a.id_satker_keu = '{$roleSatker}' ";
        }

        if ($level == 'K/L') {
            $where = 'and a.id_satker_keu in (
                select kdsatker_keu
                from ms_satker
                where kdsatker_keu is not null
            )';
        } elseif ($level == 'WILAYAH') {

            if ($wilayah) {
                $where = "and a.id_satker_keu in (
                    select kdsatker_keu
                    from ms_satker
                    where (inst_satkerinduk = '{$wilayah}' or inst_satkerkd = '{$wilayah}' )
                )";
            }

        } elseif ($level == 'SATKER') {
            if ($satker) {
                $where = "and a.id_satker_keu in (
                    select kdsatker_keu
                    from ms_satker
                    where ( inst_satkerinduk = '{$satker}' or inst_satkerkd = '{$satker}' )
                )";
            }
        }

        /*
        if ($kdsatker && $kdsatker != '00') {
            $where = " and b.inst_satkerkd = '{$kdsatker}'";
        }
        */

        switch ($type) {
            case 'statistik_pegawai':
                $sql = "
                    select sum(a.jml_laki) as total_laki, sum(a.jml_perempuan) as total_perempuan,
                    sum(a.jml_jaksa) as total_jaksa, sum(a.jml_tu) as total_tu, sum(a.total) as total
                    from vw_pegawai_dashboard a
                    where 1=1 {$where}
                ";
                // echo $sql;exit;
                $data = DB::select($sql);

                return $data;
                break;

            case 'statistik_grafik_psp_asset':
                $sql = "
                    select 'Sudah PSP' as tipe, count(a.kode_barang) as total
                    from siman.vw_all_asset_siman a
                    where 1=1 {$where} and a.no_psp is not null
                    union all
                    select 'Belum PSP' as tipe, count(a.kode_barang) as total
                    from siman.vw_all_asset_siman a
                    where 1=1 {$where} and a.no_psp is null
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_grafik_tahun_asset':
                $sql = "
                    select extract('year' from a.tanggal_perolehan) as tahun, count(a.kode_barang) as total
                    from siman.vw_all_asset_siman a
                    where 1=1 {$where} and extract('year' from a.tanggal_perolehan) between ( extract('year' from CURRENT_DATE) - 5 ) and extract('year' from CURRENT_DATE)
                    group by extract('year' from a.tanggal_perolehan)
                    order by extract('year' from a.tanggal_perolehan) asc
                ";
                $data = DB::select($sql);

                return $data;
                break;

            case 'statistik_subkelompok_aset_lainnya':
                $sql = "
                    select upper(a.sub_kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_tetap_lainnya_kl a
                    where 1=1 and sub_kelompok is not null {$where}
                    group by a.sub_kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kelompok_aset_lainnya':
                $sql = "
                    select upper(a.kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_tetap_lainnya_kl a
                    where 1=1 and kelompok is not null {$where}
                    group by a.kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kondisi_aset_lainnya':
                $sql = "
                    select upper(a.kondisi) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_tetap_lainnya_kl a
                    where 1=1 and kondisi is not null {$where}
                    group by a.kondisi
                ";
                $data = DB::select($sql);

                return $data;
                break;

            case 'statistik_subkelompok_renovasi':
                $sql = "
                    select upper(a.sub_kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_renovasi_kl a
                    where 1=1 and sub_kelompok is not null {$where}
                    group by a.sub_kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kelompok_renovasi':
                $sql = "
                    select upper(a.kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_renovasi_kl a
                    where 1=1 and kelompok is not null {$where}
                    group by a.kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kondisi_renovasi':
                $sql = "
                    select upper(a.kondisi) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_renovasi_kl a
                    where 1=1 and kondisi is not null {$where}
                    group by a.kondisi
                ";
                $data = DB::select($sql);

                return $data;
                break;

            case 'statistik_subkelompok_konstruksi':
                $sql = "
                    select upper(a.sub_kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_kdp_kl a
                    where 1=1 and sub_kelompok is not null {$where}
                    group by a.sub_kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kelompok_konstruksi':
                $sql = "
                    select upper(a.kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_kdp_kl a
                    where 1=1 and kelompok is not null {$where}
                    group by a.kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kondisi_konstruksi':
                $sql = "
                    select upper(a.kondisi) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_kdp_kl a
                    where 1=1 and kondisi is not null {$where}
                    group by a.kondisi
                ";
                $data = DB::select($sql);

                return $data;
                break;

            case 'statistik_subkelompok_jalan_jembatan':
                $sql = "
                    select upper(a.sub_kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_jalan_jembatan_kl a
                    where 1=1 and sub_kelompok is not null {$where}
                    group by a.sub_kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kelompok_jalan_jembatan':
                $sql = "
                    select upper(a.kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_jalan_jembatan_kl a
                    where 1=1 and kelompok is not null {$where}
                    group by a.kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kondisi_jalan_jembatan':
                $sql = "
                    select upper(a.kondisi) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_jalan_jembatan_kl a
                    where 1=1 and kondisi is not null {$where}
                    group by a.kondisi
                ";
                $data = DB::select($sql);

                return $data;
                break;

            case 'statistik_subkelompok_bangunan_air':
                $sql = "
                    select upper(a.sub_kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_bangunan_air_kl a
                    where 1=1 and sub_kelompok is not null {$where}
                    group by a.sub_kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kelompok_bangunan_air':
                $sql = "
                    select upper(a.kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_bangunan_air_kl a
                    where 1=1 and kelompok is not null {$where}
                    group by a.kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kondisi_bangunan_air':
                $sql = "
                    select upper(a.kondisi) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_bangunan_air_kl a
                    where 1=1 and kondisi is not null {$where}
                    group by a.kondisi
                ";
                $data = DB::select($sql);

                return $data;
                break;

            case 'statistik_subkelompok_rumah':
                $sql = "
                    select upper(a.sub_kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_rumah_negara_kl a
                    where 1=1 and sub_kelompok is not null {$where}
                    group by a.sub_kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kelompok_rumah':
                $sql = "
                    select upper(a.kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_rumah_negara_kl a
                    where 1=1 and kelompok is not null {$where}
                    group by a.kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kondisi_rumah':
                $sql = "
                    select upper(a.kondisi) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_rumah_negara_kl a
                    where 1=1 and kondisi is not null {$where}
                    group by a.kondisi
                ";
                $data = DB::select($sql);

                return $data;
                break;

            case 'statistik_subkelompok_instalasi_jaringan':
                $sql = "
                    select upper(a.sub_kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_instalasi_jaringan_kl a
                    where 1=1 and sub_kelompok is not null {$where}
                    group by a.sub_kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kelompok_instalasi_jaringan':
                $sql = "
                    select upper(a.kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_instalasi_jaringan_kl a
                    where 1=1 and kelompok is not null {$where}
                    group by a.kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kondisi_instalasi_jaringan':
                $sql = "
                    select upper(a.kondisi) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_instalasi_jaringan_kl a
                    where 1=1 and kondisi is not null {$where}
                    group by a.kondisi
                ";
                $data = DB::select($sql);

                return $data;
                break;

            case 'statistik_kelompok_tik':
                $sql = "
                    select upper(a.kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_tak_berwujud_kl a
                    where 1=1 and kelompok is not null {$where}
                    group by a.kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kondisi_tak_berwujud':
                $sql = "
                    select upper(a.kondisi) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_tak_berwujud_kl a
                    where 1=1 and kondisi is not null {$where}
                    group by a.kondisi
                ";
                $data = DB::select($sql);

                return $data;
                break;

            case 'statistik_subkelompok_tik':
                $sql = "
                    select upper(a.sub_kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_pm_tik_kl a
                    where 1=1 and sub_kelompok is not null {$where}
                    group by a.sub_kelompok;
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kelompok_tik':
                $sql = "
                    select upper(a.kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_pm_tik_kl a
                    where 1=1 and kelompok is not null {$where}
                    group by a.kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_penggunaan_tik':
                $sql = "
                    select upper(a.status_penggunaan) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_pm_tik_kl a
                    where 1=1 and status_penggunaan is not null {$where}
                    group by a.status_penggunaan
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kondisi_tik':
                $sql = "
                    select upper(a.kondisi) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_pm_tik_kl a
                    where 1=1 and kondisi is not null {$where}
                    group by a.kondisi
                ";
                $data = DB::select($sql);

                return $data;
                break;

            case 'statistik_subkelompok_nontik':
                $sql = "
                    select upper(a.sub_kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_pm_nontik_kl a
                    where 1=1 and sub_kelompok is not null {$where}
                    group by a.sub_kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kelompok_nontik':
                $sql = "
                    select upper(a.kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_pm_nontik_kl a
                    where 1=1 and kelompok is not null {$where}
                    group by a.kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kondisi_nontik':
                $sql = "
                    select upper(a.kondisi) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_pm_nontik_kl a
                    where 1=1 and kondisi is not null {$where}
                    group by a.kondisi
                ";
                $data = DB::select($sql);

                return $data;
                break;

            case 'statistik_subkelompok_kendaraan':
                $sql = "
                    select upper(a.sub_kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_alat_angkutan_kl a
                    where 1=1 and sub_kelompok is not null {$where}
                    group by a.sub_kelompok;
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kelompok_kendaraan':
                $sql = "
                    select upper(a.kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_alat_angkutan_kl a
                    where 1=1 and kelompok is not null {$where}
                    group by a.kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_penggunaan_kendaraan':
                $sql = "
                    select upper(a.status_penggunaan) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_alat_angkutan_kl a
                    where 1=1 and status_penggunaan is not null {$where}
                    group by a.status_penggunaan
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kondisi_kendaraan':
                $sql = "
                    select upper(a.kondisi) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_alat_angkutan_kl a
                    where 1=1 and kondisi is not null {$where}
                    group by a.kondisi
                ";
                $data = DB::select($sql);

                return $data;
                break;

            case 'statistik_subkelompok_alat_berat':
                $sql = "
                    select upper(a.sub_kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_alat_berat_kl a
                    where 1=1 and sub_kelompok is not null {$where}
                    group by a.sub_kelompok;
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kelompok_alat_berat':
                $sql = "
                    select upper(a.kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_alat_berat_kl a
                    where 1=1 and kelompok is not null {$where}
                    group by a.kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_penggunaan_alat_berat':
                $sql = "
                    select upper(a.status_penggunaan) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_alat_berat_kl a
                    where 1=1 and status_penggunaan is not null {$where}
                    group by a.status_penggunaan
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kondisi_alat_berat':
                $sql = "
                    select upper(a.kondisi) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_alat_berat_kl a
                    where 1=1 and kondisi is not null {$where}
                    group by a.kondisi
                ";
                $data = DB::select($sql);

                return $data;
                break;

            case 'statistik_subkelompok_gedung':
                $sql = "
                    select upper(a.sub_kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_gedung_bangunan_kl a
                    where 1=1 and sub_kelompok is not null {$where}
                    group by a.sub_kelompok;
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kelompok_gedung':
                $sql = "
                    select upper(a.kelompok) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_gedung_bangunan_kl a
                    where 1=1 and kelompok is not null {$where}
                    group by a.kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_penggunaan_gedung':
                $sql = "
                    select upper(a.status_penggunaan) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_gedung_bangunan_kl a
                    where 1=1 and status_penggunaan is not null {$where}
                    group by a.status_penggunaan
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kondisi_gedung':
                $sql = "
                    select upper(a.kondisi) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_gedung_bangunan_kl a
                    where 1=1 and kondisi is not null {$where}
                    group by a.kondisi
                ";
                $data = DB::select($sql);

                return $data;
                break;

            case 'statistik_subkelompok_tanah':
                $sql = "
                    select a.sub_kelompok as judul, sum(a.luas_tanah_seluruhnya::bigint) as total, sum(nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_tanah_kl a
                    where 1=1 {$where}
                    group by sub_kelompok
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kelompok_tanah':
                $sql = "
                    select a.kelompok as judul, sum(a.luas_tanah_seluruhnya::bigint) as total, sum(nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_tanah_kl a
                    where 1=1 {$where}
                    group by kelompok;
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_klasifikasi_tanah':
                $sql = "
                    select 'TANAH KOSONG' as judul, sum(a.luas_tanah_kosong) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_tanah_kl a
                    where a.luas_tanah_kosong <> 0 and a.luas_tanah_kosong > 0 and a.luas_tanah_kosong is not null {$where}
                    union all
                    select 'TANAH TERISI' as judul, sum(a.luas_tanah_untuk_bangunan) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_tanah_kl a
                    where a.luas_tanah_untuk_bangunan <> 0 and a.luas_tanah_untuk_bangunan > 0 and a.luas_tanah_untuk_bangunan is not null {$where}
                ";
                $data = DB::select($sql);

                return $data;
                break;
            case 'statistik_kondisi_tanah':
                $sql = "
                    select upper(a.kondisi) as judul, count(a.id) as total, sum(a.nilai_perolehan::bigint) as total_nilai_perolehan
                    from siman.siman_aset_tanah_kl a
                    where 1=1 and kondisi is not null {$where}
                    group by a.kondisi
                ";
                $data = DB::select($sql);

                return $data;
                break;

            case 'analisis_kebutuhan_bmn':
                $currentYear = (int) date('Y');
                $years = range($currentYear, $currentYear + 4);
                $unions = [];
                foreach ($years as $i => $year) {
                    // Threshold: 0 for year 1, 0.1 for year 2, 0.2 for year 3, 0.3 for year 4, 0.4 for year 5
                    $threshold = $i === 0 ? 0 : ($i * 0.1);
                    $keterangan = $i === 0
                        ? 'Aset dengan nilai buku nol atau habis masa pakai'
                        : ('Aset dengan nilai buku < '.($threshold * 100).'% nilai perolehan (perlu perencanaan)');
                    $whereNilaiBuku = $i === 0
                        ? 'a.nilai_buku <= 0 OR a.nilai_buku IS NULL'
                        : ($i === 4
                            ? 'a.nilai_buku > (a.nilai_perolehan * 0.'.($i - 1).") AND a.nilai_buku <= (a.nilai_perolehan * 0.$i)"
                            : 'a.nilai_buku > (a.nilai_perolehan * 0.'.($i - 1).") AND a.nilai_buku <= (a.nilai_perolehan * 0.$i)");
                    if ($i === 0) {
                        $whereNilaiBuku = 'a.nilai_buku <= 0 OR a.nilai_buku IS NULL';
                    } else {
                        $whereNilaiBuku = 'a.nilai_buku > (a.nilai_perolehan * 0.'.($i - 1).") AND a.nilai_buku <= (a.nilai_perolehan * 0.$i)";
                    }
                    $unions[] = "SELECT
                        '$year' as tahun_prediksi,
                        COUNT(CASE WHEN $whereNilaiBuku THEN 1 END) as jumlah_aset,
                        SUM(CASE WHEN $whereNilaiBuku THEN a.nilai_perolehan ELSE 0 END) as total_nilai_penggantian,
                        '$keterangan' as keterangan
                    FROM (
                        SELECT id::bigint, nilai_buku::bigint, nilai_perolehan::bigint, tipe_aset FROM (
                            SELECT id::bigint, nilai_buku::bigint, nilai_perolehan::bigint, 'gedung' as tipe_aset FROM siman.siman_aset_gedung_bangunan_kl WHERE 1=1 {$where}
                            UNION ALL
                            SELECT id::bigint, nilai_buku::bigint, nilai_perolehan::bigint, 'tik' as tipe_aset FROM siman.siman_aset_pm_tik_kl WHERE 1=1 {$where}
                            UNION ALL
                            SELECT id::bigint, nilai_buku::bigint, nilai_perolehan::bigint, 'non_tik' as tipe_aset FROM siman.siman_aset_pm_nontik_kl WHERE 1=1 {$where}
                            UNION ALL
                            SELECT id::bigint, nilai_buku::bigint, nilai_perolehan::bigint, 'kendaraan' as tipe_aset FROM siman.siman_aset_alat_angkutan_kl WHERE 1=1 {$where}
                            UNION ALL
                            SELECT id::bigint, nilai_buku::bigint, nilai_perolehan::bigint, 'alat_berat' as tipe_aset FROM siman.siman_aset_alat_berat_kl WHERE 1=1 {$where}
                        ) all_assets
                    ) a";
                }
                $sql = implode("\nUNION ALL\n", $unions);
                $sql = str_replace('and a.id_satker_keu', 'and id_satker_keu', $sql);
                $data = DB::select($sql);

                return $data;
                break;

            case 'statistik_bmn':
                $sql = "
                    select 'Alat Angkutan Bermotor' as kategori, 
                        count(a.id) as total, sum(a.nilai_perolehan::float8) as nilai_perolehan,
                        -1 as total_luas,
                        'asset/angkutan' as url, 'angkutan' as tipe 
                    from siman.siman_aset_alat_angkutan_kl a 
                    left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Alat Besar' as kategori, 
                        count(a.id) as total, sum(a.nilai_perolehan::float8) as nilai_perolehan,
                        -1 as total_luas,
                        'asset/alat_besar' as url, 'alat_besar' as tipe 
                    from siman.siman_aset_alat_berat_kl a 
                    left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Bangunan & Air' as kategori, 
                        count(a.id) as total, sum(a.nilai_perolehan::float8) as nilai_perolehan,
                        -1 as total_luas,
                        'asset/bangunan_air' as url, 'bangunan_air' as tipe 
                    from siman.siman_aset_bangunan_air_kl a 
                    left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Gedung Bangunan' as kategori, 
                        count(a.id) as total, sum(a.nilai_perolehan::float8) as nilai_perolehan,
                        sum(a.luas_bangunan::float8) as total_luas,
                        'asset/gedung' as url, 'gedung' as tipe 
                    from siman.siman_aset_gedung_bangunan_kl a 
                    left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Instalasi Jaringan' as kategori, 
                        count(a.id) as total, sum(a.nilai_perolehan::float8) as nilai_perolehan,
                        -1 as total_luas,
                        'asset/jaringan' as url, 'jaringan' as tipe 
                    from siman.siman_aset_instalasi_jaringan_kl a 
                    left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Jalan & Jembatan' as kategori, 
                        count(a.id) as total, sum(a.nilai_perolehan::float8) as nilai_perolehan,
                        -1 as total_luas,
                        'asset/jalan_jembatan' as url, 'jalan_jembatan' as tipe 
                    from siman.siman_aset_jalan_jembatan_kl a 
                    left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Konstruksi dalam Pengerjaan' as kategori, 
                        count(a.id) as total, sum(a.nilai_perolehan::float8) as nilai_perolehan,
                        -1 as total_luas,
                        'asset/konstruksi' as url, 'konstruksi' as tipe 
                    from siman.siman_aset_kdp_kl a 
                    left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Peralatan Mesin Non TIK' as kategori, 
                        count(a.id) as total, sum(a.nilai_perolehan::float8) as nilai_perolehan,
                        -1 as total_luas,
                        'asset/non_tik' as url, 'non_tik' as tipe 
                    from siman.siman_aset_pm_nontik_kl a 
                    left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Peralatan Mesin TIK' as kategori, 
                        count(a.id) as total, sum(a.nilai_perolehan::float8) as nilai_perolehan,
                        -1 as total_luas,
                        'asset/tik' as url, 'tik' as tipe 
                    from siman.siman_aset_pm_tik_kl a 
                    left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Rumah Negara' as kategori, 
                        count(a.id) as total, sum(a.nilai_perolehan::float8) as nilai_perolehan,
                        sum(a.luas_bangunan::float8) as total_luas,
                        'asset/rumah' as url, 'rumah' as tipe 
                    from siman.siman_aset_rumah_negara_kl a 
                    left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Aset Tak Berwujud' as kategori, 
                        count(a.id) as total, sum(a.nilai_perolehan::float8) as nilai_perolehan,
                        -1 as total_luas,
                        'asset/wujud' as url, 'wujud' as tipe 
                    from siman.siman_aset_tak_berwujud_kl a 
                    left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Tanah' as kategori, 
                        count(a.id) as total, sum(a.nilai_perolehan::float8) as nilai_perolehan,
                        sum(a.luas_tanah_seluruhnya::float8) as total_luas,
                        'asset/tanah' as url, 'tanah' as tipe 
                    from siman.siman_aset_tanah_kl a 
                    left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Aset Tetap Lainnya' as kategori, 
                        count(a.id) as total, sum(a.nilai_perolehan::float8) as nilai_perolehan,
                        -1 as total_luas,
                        'asset/lain' as url, 'lain' as tipe 
                    from siman.siman_aset_tetap_lainnya_kl a 
                    left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Aset Tetap Renovasi' as kategori, 
                        count(a.id) as total, sum(a.nilai_perolehan::float8) as nilai_perolehan,
                        -1 as total_luas,
                        'asset/renovasi' as url, 'renovasi' as tipe 
                    from siman.siman_aset_renovasi_kl a 
                    left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                ";
                /*
                $sql = "
                    select 'Alat Angkutan Bermotor' as kategori, count(a.id) as total,'asset/angkutan' as url, 'angkutan' as tipe from siman.siman_aset_alat_angkutan_kl a left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Alat Besar' as kategori, count(a.id) as total,'asset/alat_besar' as url, 'alat_besar' as tipe from siman.siman_aset_alat_berat_kl a left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Bangunan & Air' as kategori, count(a.id) as total,'asset/bangunan_air' as url, 'bangunan_air' as tipe from siman.siman_aset_bangunan_air_kl a left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Gedung Bangunan' as kategori, count(a.id) as total,'asset/gedung' as url, 'gedung' as tipe from siman.siman_aset_gedung_bangunan_kl a left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Instalasi Jaringan' as kategori, count(a.id) as total,'asset/jaringan' as url, 'jaringan' as tipe from siman.siman_aset_instalasi_jaringan_kl a left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Jalan & Jembatan' as kategori, count(a.id) as total,'asset/jalan_jembatan' as url, 'jalan_jembatan' as tipe from siman.siman_aset_jalan_jembatan_kl a left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Konstruksi dalam Pengerjaan' as kategori, count(a.id) as total,'asset/konstruksi' as url, 'konstruksi' as tipe from siman.siman_aset_kdp_kl a left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Peralatan Mesin Non TIK' as kategori, count(a.id) as total,'asset/non_tik' as url, 'non_tik' as tipe from siman.siman_aset_pm_nontik_kl a left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Peralatan Mesin TIK' as kategori, count(a.id) as tota,'asset/tik' as url, 'tik' as tipe from siman.siman_aset_pm_tik_kl a left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Rumah Negara' as kategori, count(a.id) as total,'asset/rumah' as url, 'rumah' as tipe from siman.siman_aset_rumah_negara_kl a left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Aset Tak Berwujud' as kategori, count(a.id) as total,'asset/wujud' as url, 'wujud' as tipe from siman.siman_aset_tak_berwujud_kl a left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Tanah' as kategori, count(a.id) as total,'asset/tanah' as url, 'tanah' as tipe from siman.siman_aset_tanah_kl a left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Aset Tetap Lainnya' as kategori, count(a.id) as total,'asset/lain' as url, 'lain' as tipe from siman.siman_aset_tetap_lainnya_kl a left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                    union all
                    select 'Aset Tetap Renovasi' as kategori, count(a.id) as total,'asset/renovasi' as url, 'renovasi' as tipe from siman.siman_aset_renovasi_kl a left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1 {$where}
                ";
                */
                // echo $sql;exit;
                $data = DB::select($sql);

                return $data;
                break;
        }

    }

    public function getTahunPerolehan()
    {
        $tables = [
            'asset_alat_angkutan_bermotor',
            'asset_alat_besar',
            'asset_alat_persenjataan',
            'asset_bangunan_air',
            'asset_gedung_bangunan',
            'asset_instalasi_jaringan',
            'asset_jalan_jembatan',
            'asset_konstruksi_dalam_pengerjaan',
            'asset_peralatan_mesin_non_tik',
            'asset_peralatan_mesin_tik',
            'asset_rumah_negara',
            'asset_tak_berwujud',
            'asset_tanah',
            'asset_tetap_lainnya',
            'asset_tetap_renovasi',
        ];

        /*
        $tables = [
            'siman.siman_aset_alat_angkutan_kl',
            'siman.siman_aset_alat_berat_kl',
            'siman.siman_aset_bangunan_air_kl',
            'siman.siman_aset_gedung_bangunan_kl',
            'siman.siman_aset_instalasi_jaringan_kl',
            'siman.siman_aset_jalan_jembatan_kl',
            'siman.siman_aset_kdp_kl',
            'siman.siman_aset_pm_nontik_kl',
            'siman.siman_aset_pm_tik_kl',
            'siman.siman_aset_rumah_negara_kl',
            'siman.siman_aset_tak_berwujud_kl',
            'siman.siman_aset_tanah_kl',
            'siman.siman_aset_tetap_lainnya_kl',
            'siman.siman_aset_renovasi_kl',
            'siman.siman_aset_rusak_berat_kl',
            'siman.siman_aset_persediaan_kl',
        ];
        */

        $distinctYears = [];

        foreach ($tables as $table) {
            $distinctYears = array_merge(
                $distinctYears,
                DB::table($table)
                    ->distinct()
                    ->select(DB::raw('EXTRACT(YEAR FROM tgl_perolehan) as tahun_perolehan'))
                    ->get()
                    ->pluck('tahun_perolehan')
                    ->toArray()
            );
        }

        // Remove duplicates and sort the years
        $distinctYears = array_unique($distinctYears);
        sort($distinctYears);

        return $distinctYears;
    }
}

/*
select 'Alat Angkutan Bermotor' as kategori, count(a.id) as total,'asset/angkutan' as url from asset_alat_angkutan_bermotor a left join ms_satker as b on a.id_satker = b.kdsatker_keu where 1=1 {$where}
union all
select 'Alat Besar' as kategori, count(a.id) as total,'asset/alat_besar' as url from asset_alat_besar a left join ms_satker as b on a.id_satker = b.kdsatker_keu where 1=1 {$where}
union all
select 'Alat Persenjataan' as kategori, count(a.id) as total,'asset/senjata' as url from asset_alat_persenjataan a left join ms_satker as b on a.id_satker = b.kdsatker_keu where 1=1 {$where}
union all
select 'Bangunan & Air' as kategori, count(a.id) as total,'asset/bangunan_air' as url from asset_bangunan_air a left join ms_satker as b on a.id_satker = b.kdsatker_keu where 1=1 {$where}
union all
select 'Gedung Bangunan' as kategori, count(a.id) as total,'asset/gedung' as url from asset_gedung_bangunan a left join ms_satker as b on a.id_satker = b.kdsatker_keu where 1=1 {$where}
union all
select 'Instalasi Jaringan' as kategori, count(a.id) as total,'asset/jaringan' as url from asset_instalasi_jaringan a left join ms_satker as b on a.id_satker = b.kdsatker_keu where 1=1 {$where}
union all
select 'Jalan & Jembatan' as kategori, count(a.id) as total,'asset/jalan_jembatan' as url from asset_jalan_jembatan a left join ms_satker as b on a.id_satker = b.kdsatker_keu where 1=1 {$where}
union all
select 'Konstruksi dalam Pengerjaan' as kategori, count(a.id) as total,'asset/konstruksi' as url from asset_konstruksi_dalam_pengerjaan a left join ms_satker as b on a.id_satker = b.kdsatker_keu where 1=1 {$where}
union all
select 'Peralatan Mesin Non TIK' as kategori, count(a.id) as total,'asset/non_tik' as url from asset_peralatan_mesin_non_tik a left join ms_satker as b on a.id_satker = b.kdsatker_keu where 1=1 {$where}
union all
select 'Peralatan Mesin TIK' as kategori, count(a.id) as tota,'asset/tik' as url from asset_peralatan_mesin_tik a left join ms_satker as b on a.id_satker = b.kdsatker_keu where 1=1 {$where}
union all
select 'Rumah Negara' as kategori, count(a.id) as total,'asset/rumah' as url from asset_rumah_negara a left join ms_satker as b on a.id_satker = b.kdsatker_keu where 1=1 {$where}
union all
select 'Aset Tak Berwujud' as kategori, count(a.id) as total,'asset/wujud' as url from asset_tak_berwujud a left join ms_satker as b on a.id_satker = b.kdsatker_keu where 1=1 {$where}
union all
select 'Tanah' as kategori, count(a.id) as total,'asset/tanah' as url from asset_tanah a left join ms_satker as b on a.id_satker = b.kdsatker_keu where 1=1 {$where}
union all
select 'Aset Tetap Lainnya' as kategori, count(a.id) as total,'asset/lain' as url from asset_tetap_lainnya a left join ms_satker as b on a.id_satker = b.kdsatker_keu where 1=1 {$where}
union all
select 'Aset Tetap Renovasi' as kategori, count(a.id) as total,'asset/renovasi' as url from asset_tetap_renovasi a left join ms_satker as b on a.id_satker = b.kdsatker_keu where 1=1 {$where}

union all
                    select 'Alat Rusak Berat' as kategori, count(a.id) as total,'asset/senjata' as url from siman.siman_aset_rusak_berat_kl a left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1{$where}
                    union all
                    select 'Aset Persediaan' as kategori, count(a.id) as total,'asset/senjata' as url from siman.siman_aset_persediaan_kl a left join ms_satker as b on a.id_satker_keu = b.kdsatker_keu where 1=1{$where}
*/
