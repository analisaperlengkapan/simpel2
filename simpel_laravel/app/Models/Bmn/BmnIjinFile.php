<?php

namespace App\Models\Bmn;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class BmnIjinFile extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'bmn_ijin_pemakaian_satker_pegawai_file';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'pengajuan_pegawai_id',
        'jenis',
        'url'
    ];

    static function getDetail($pengajuan_id)
    {
        $query = DB::table('bmn_ijin_pemakaian_satker_pegawai as a');
        $query->select('a.*', 'b.nama');
        $query->join('mv_curr_pegawai_all as b', 'a.nip', '=', 'b.peg_nip_baru');
        $query->where('a.pengajuan_id', $pengajuan_id);
        $query->orderBy('b.nama', 'asc');
        return $query->get();
    }

    static function getMasterFile($kategori)
    {
        $query = DB::table('ms_pengajuan_file');
        $query->select('*');
        $query->where('kategori', $kategori);
        $query->orderBy('id', 'asc');
        return $query->get();
    }
}
