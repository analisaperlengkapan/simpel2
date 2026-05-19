<?php

namespace App\Models\Pengadaan\PengadaanBarangJasa;

use App\Blameable;
use Carbon\Carbon;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class PengadaanBarangJasaSpk extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'pengadaan_barang_jasa_spk';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'no_spk',
        'no_permintaan',
        'tgl_permintaan',
        'no_ba',
        'tgl_ba',
        'tgl_mulai',
        'tgl_spk',
        'tgl_selesai',
        'nama_penyedia',
        'keterangan',
        'instruksi',
    ];

    public function getTglPermintaanAttribute()
    {
        return Carbon::parse($this->attributes['tgl_permintaan'])->translatedFormat('d F Y');
    }

    public function getTglBaAttribute()
    {
        return Carbon::parse($this->attributes['tgl_ba'])->translatedFormat('d F Y');
    }

    public function getTglMulaiAttribute()
    {
        return Carbon::parse($this->attributes['tgl_mulai'])->translatedFormat('d F Y');
    }

    public function getTglSpkAttribute()
    {
        return Carbon::parse($this->attributes['tgl_spk'])->translatedFormat('d F Y');
    }

    public function getTglSelesaiAttribute()
    {
        return Carbon::parse($this->attributes['tgl_selesai'])->translatedFormat('d F Y');
    }
}
