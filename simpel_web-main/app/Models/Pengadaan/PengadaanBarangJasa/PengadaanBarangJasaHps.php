<?php

namespace App\Models\Pengadaan\PengadaanBarangJasa;

use App\Blameable;
use App\Traits\LogTrait;
use Carbon\Carbon;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class PengadaanBarangJasaHps extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'pengadaan_barang_jasa_hps';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'no_hps',
        'tgl_hps',
        'nip_penandatangan',
        'nama_penandatangan',
        'pangkat_penandatangan',
        'barang',
        'keterangan',
    ];
}
