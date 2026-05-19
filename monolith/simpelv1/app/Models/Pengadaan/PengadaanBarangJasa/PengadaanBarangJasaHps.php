<?php

namespace App\Models\Pengadaan\PengadaanBarangJasa;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class PengadaanBarangJasaHps extends Model
{
    use Blameable;
    use HasFactory;

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
