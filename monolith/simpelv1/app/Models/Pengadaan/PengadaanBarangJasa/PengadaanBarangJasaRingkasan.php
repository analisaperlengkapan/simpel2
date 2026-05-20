<?php

namespace App\Models\Pengadaan\PengadaanBarangJasa;

use App\Blameable;
use Carbon\Carbon;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class PengadaanBarangJasaRingkasan extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'pengadaan_barang_jasa_ringkasan_kontrak';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'no_dipa',
        'tgl_dipa',
        'cara_pembayaran',
        'alamat_penyedia',
        'nama_bank',
        'kantor_bank',
        'no_rek',
        'npwp',
        'sanksi',
    ];

    public function getTglDipaAttribute()
    {
        return Carbon::parse($this->attributes['tgl_dipa'])->translatedFormat('d F Y');
    }
}
