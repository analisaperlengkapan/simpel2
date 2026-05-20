<?php

namespace App\Models\Bmn;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class BmnIjinPegawai extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'bmn_ijin_pemakaian_satker_pegawai';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        // 'pengajuan_id',
        'nip',
        'pangkat',
        'jabatan',
        'username',
        'password',
    ];

    public static function getDetail1($pengajuan_id)
    {
        $query = DB::table('bmn_ijin_pemakaian_satker_pegawai as a');
        $query->select('a.*', 'b.nama');
        $query->join('mv_curr_pegawai_all as b', 'a.nip', '=', 'b.peg_nip_baru');
        $query->where('a.pengajuan_id', $pengajuan_id);
        $query->orderBy('b.nama', 'asc');

        return $query->get();
    }

    public static function getDetail($pengajuan_id)
    {
        $sql = "WITH tbl_file as(
			select pengajuan_pegawai_id, string_agg(concat(jenis,'---',url), '|#|' order by id) as filenya
                    from pengajuan_user_spse_sirup_pegawai_file group by pengajuan_pegawai_id
                )
        SELECT a.*, b.filenya, c.nama, concat(d.kode_barang,'-',d.nama_barang,'-',d.keterangan) as nm_aset
        FROM bmn_ijin_pemakaian_satker_pegawai a
        LEFT JOIN tbl_file b on a.id = b.pengajuan_pegawai_id
        LEFT JOIN mv_curr_pegawai_all c on a.nip = c.peg_nip_baru
		Left Join bmn_ijin_pemakaian_satker_pegawai_aset d on a.nip=d.nip and a.pengajuan_id=d.pengajuan_id
        WHERE a.pengajuan_id = ?
        ";
        $result = DB::select($sql, [$pengajuan_id]);

        return $result;
    }

    public static function getDetailMonitor($pengajuan_id)
    {
        $sql = "WITH tbl_file as(
			select pengajuan_pegawai_id, string_agg(concat(jenis,'---',url), '|#|' order by id) as filenya
                    from pengajuan_user_spse_sirup_pegawai_file group by pengajuan_pegawai_id
                )
        SELECT a.*, b.filenya, c.nama, concat(d.kode_barang,'-',d.nama_barang,'-',d.keterangan) as nm_aset
        FROM bmn_ijin_pemakaian_satker_pegawai a
        LEFT JOIN tbl_file b on a.id = b.pengajuan_pegawai_id
        LEFT JOIN mv_curr_pegawai_all c on a.nip = c.peg_nip_baru
		Left Join bmn_ijin_pemakaian_satker_pegawai_aset d on a.nip=d.nip and a.pengajuan_id=d.pengajuan_id
        WHERE a.pengajuan_id = ?
        ";
        $result = DB::select($sql, [$pengajuan_id]);

        return $result;
    }

    public static function getMsPegawai($inst_satkerkd, $pengajuan_id)
    {
        $sql = 'SELECT a.peg_nip_baru as nip,a.nama,a.pangkat,a.jabatan
        FROM mv_curr_pegawai_all a
        LEFT JOIN bmn_ijin_pemakaian_satker_pegawai b on a.peg_nip_baru = b.nip and b.pengajuan_id = ?
        WHERE b.nip is null and a.inst_satkerkd = ?
        ';
        $result = DB::select($sql, [$pengajuan_id, $inst_satkerkd]);

        return $result;
    }
}
