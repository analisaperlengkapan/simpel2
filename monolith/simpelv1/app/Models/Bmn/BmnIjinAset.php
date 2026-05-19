<?php

namespace App\Models\Bmn;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class BmnIjinAset extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'bmn_ijin_pemakaian_satker_pegawai_aset';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'pengajuan_id',
        'kode_barang',
        'nama_barang',
        'keterangan',
        'nup',
        'nip',
    ];

    public static function getAset($pengajuan_id)
    {
        $sql = 'select * from vw_asset_bmn';
        $result = DB::select($sql);

        return $result;
    }
}
