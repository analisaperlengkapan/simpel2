<?php

namespace App\Models\Bmn\PengajuanPenghapusanBmn;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class PengajuanPenghapusanBmnAssetFile extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'pengajuan_penghapusan_bmn_asset_file';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'pengajuan_asset_id',
        'jenis',
        'url',
        'no_sk',
        'tgl_sk',
    ];

    static function getDetail($pengajuan_id, $jenis="")
    {
        $query = DB::table('pengajuan_penghapusan_bmn_asset_file as a');
        $query->select('a.*');
        $query->join('pengajuan_penghapusan_bmn_asset as b', 'a.pengajuan_asset_id', '=', 'b.id');
        $query->where('b.pengajuan_id', $pengajuan_id);
        return $query->get();
    }
}
