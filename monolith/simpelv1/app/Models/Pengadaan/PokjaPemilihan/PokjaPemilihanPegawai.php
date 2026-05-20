<?php

namespace App\Models\Pengadaan\PokjaPemilihan;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class PokjaPemilihanPegawai extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'pengajuan_pokja_pemilihan_pegawai';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'pengajuan_id',
        'nip',
        'pangkat',
        'jabatan',
    ];

    public static function getDetail($pengajuan_id)
    {
        $sql = "WITH tbl_file as(
			select pengajuan_pegawai_id, string_agg(concat(jenis,'---',url), '|#|' order by id) as filenya
                    from pengajuan_pokja_pemilihan_pegawai_file group by pengajuan_pegawai_id
                )
        SELECT a.*, b.filenya, c.nama
        FROM pengajuan_pokja_pemilihan_pegawai a
        LEFT JOIN tbl_file b on a.id = b.pengajuan_pegawai_id
        LEFT JOIN mv_curr_pegawai_all c on a.nip = c.peg_nip_baru
        WHERE a.pengajuan_id = ?
        ";
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
