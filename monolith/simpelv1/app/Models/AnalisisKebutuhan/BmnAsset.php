<?php

namespace App\Models\AnalisisKebutuhan;

// use Illuminate\Contracts\Auth\MustVerifyEmail;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class BmnAsset extends Model
{
    use HasFactory, Blameable;
    protected $table = 'pengajuan_kebutuhan_bmn_asset';
    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'pengajuan_kebutuhan_bmn_id',
        'kode_barang',
        'nm_barang',
        'ms_jenis_asset_id',
        'keterangan',
    ];
}
