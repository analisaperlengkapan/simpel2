<?php

namespace App\Models\Pengadaan\PengadaanBarangJasa;

use App\Blameable;
use Carbon\Carbon;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class PengadaanBarangJasaBast extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'pengadaan_barang_jasa_bast';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'no_bast',
        'tgl_bast',
        'nama_pejabat',
        'nip_pejabat',
        'pangkat_pejabat',
        'jabatan_pejabat',
        'no_bapp',
        'tgl_bapp',
        'no_kepja',
        'tgl_kepja',
    ];

    public function getTglBastAttribute()
    {
        return Carbon::parse($this->attributes['tgl_bast'])->isoFormat('dddd D MMMM Y');
    }

    public function getTglBastNormalAttribute()
    {
        return Carbon::parse($this->attributes['tgl_bast'])->translatedFormat('d F Y');
    }

    public function getTglBappAttribute()
    {
        return Carbon::parse($this->attributes['tgl_bapp'])->isoFormat('dddd D MMMM Y');
    }

    public function getTglKepjaAttribute()
    {
        return Carbon::parse($this->attributes['tgl_kepja'])->translatedFormat('d F Y');
    }
}
