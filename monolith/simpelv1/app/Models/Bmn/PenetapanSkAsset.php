<?php

namespace App\Models\Bmn;

use Illuminate\Database\Eloquent\Model;

class PenetapanSkAsset extends Model
{
    protected $table = 'bmn_penetapan_asset';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'asset_nama',
        'asset_kode',
        'barang_nama',
        'barang_kode',
        'bmn_penetapan_id',
        'vw_aset_psp_id',
    ];
}
