<?php

namespace App\Models\Sdm;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class Timakuntansibarangpegawai extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'sdm_timakuntansibarang_pegawai';

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
        'jabatan_tim',
        'nama',
    ];

    public static function getDetail1($pengajuan_id)
    {
        $query = DB::table('pengajuan_user_spse_sirup_pegawai as a');
        $query->select('a.*', 'b.nama');
        $query->join('mv_curr_pegawai_all as b', 'a.nip', '=', 'b.peg_nip_baru');
        $query->where('a.pengajuan_id', $pengajuan_id);
        $query->orderBy('b.nama', 'asc');

        return $query->get();
    }

    public static function getDetail($pengajuan_id)
    {
        $sql = 'SELECT a.*,c.nama
        FROM sdm_timakuntansibarang_pegawai a
        LEFT JOIN mv_curr_pegawai_all c on a.nip = c.peg_nip_baru
        WHERE a.pengajuan_id = ?
        ';
        $result = DB::select($sql, [$pengajuan_id]);

        return $result;
    }

    public static function getMsPegawai($inst_satkerkd)
    {
        $sql = 'SELECT a.peg_nip_baru as nip,a.nama,a.pangkat,a.jabatan
        FROM mv_curr_pegawai_all a
        WHERE a.inst_satkerkd = ? ';
        $result = DB::select($sql, [$inst_satkerkd]);

        return $result;
    }
}
