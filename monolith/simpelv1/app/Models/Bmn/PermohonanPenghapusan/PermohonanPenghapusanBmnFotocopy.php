<?php

namespace App\Models\Bmn\PermohonanPenghapusan;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class PermohonanPenghapusanBmnFotocopy extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'permohonan_penghapusan_bmn_fotocopy';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'pengajuan_id',
        'nomor',
        'tanggal',
        'file',
    ];

    public static function getDetail($pengajuan_id)
    {
        $sql = 'SELECT a.* FROM permohonan_penghapusan_bmn_fotocopy a WHERE a.pengajuan_id = ? ';
        $result = DB::select($sql, [$pengajuan_id]);

        return $result;
    }

    public static function getMsPegawai($inst_satkerkd, $pengajuan_id)
    {
        $sql = 'SELECT a.peg_nip_baru as nip,a.nama,a.pangkat,a.jabatan
        FROM mv_curr_pegawai_all a
        LEFT JOIN pengajuan_pokja_pemilihan_pegawai b on a.peg_nip_baru = b.nip and b.pengajuan_id = ?
        WHERE b.nip is null and a.inst_satkerkd = ?
        ';
        $result = DB::select($sql, [$pengajuan_id, $inst_satkerkd]);

        return $result;
    }
}
