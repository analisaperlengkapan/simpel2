<?php

namespace App\Models\Pengadaan\PokjaPemilihan;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class PokjaPemilihanPegawaiFile extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'pengajuan_pokja_pemilihan_pegawai_file';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'pengajuan_pegawai_id',
        'jenis',
        'url',
    ];

    public static function getDetail($pengajuan_id, $jenis = '')
    {
        $query = DB::table('pengajuan_pokja_pemilihan_pegawai_file as a');
        $query->select('a.*');
        $query->join('pengajuan_pokja_pemilihan_pegawai as b', 'a.pengajuan_pegawai_id', '=', 'b.id');
        $query->where('b.pengajuan_id', $pengajuan_id);
        if ($jenis) {
            $query->where('a.jenis', $jenis);
        }

        return $query->get();
    }

    public static function getMasterFile($kategori)
    {
        $query = DB::table('ms_pengajuan_file');
        $query->select('*');
        $query->where('kategori', $kategori);
        $query->orderBy('id', 'asc');

        return $query->get();
    }
}
