<?php

namespace App\Models\Bmn\PermohonanPenghapusan;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class PermohonanPenghapusanBmnFile extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'permohonan_penghapusan_bmn_file';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'pengajuan_id',
        'ms_penghapusan_file_id',
        'nomor',
        'tanggal',
        'file',
    ];

    static function getDetail($pengajuan_id)
    {
        $sql = "WITH tbl_file as(
			select pengajuan_asset_id,no_sk,tgl_sk, string_agg(concat(jenis,'---',url), '|#|' order by id) as filenya
                    from pengajuan_penghapusan_bmn_asset_file group by pengajuan_asset_id,no_sk,tgl_sk
                )
        SELECT a.*, b.filenya,b.no_sk,b.tgl_sk
        FROM pengajuan_penghapusan_bmn_asset a
        LEFT JOIN tbl_file b on a.id = b.pengajuan_asset_id
        WHERE a.pengajuan_id = ?
        ";
        $result = DB::select($sql, [$pengajuan_id]);
        return $result;
    }

    static function getMsPegawai($inst_satkerkd, $pengajuan_id)
    {
        $sql = "SELECT a.peg_nip_baru as nip,a.nama,a.pangkat,a.jabatan
        FROM mv_curr_pegawai_all a
        LEFT JOIN pengajuan_pokja_pemilihan_pegawai b on a.peg_nip_baru = b.nip and b.pengajuan_id = ?
        WHERE b.nip is null and a.inst_satkerkd = ?
        ";
        $result = DB::select($sql, [$pengajuan_id,$inst_satkerkd]);
        return $result;
    }
}
