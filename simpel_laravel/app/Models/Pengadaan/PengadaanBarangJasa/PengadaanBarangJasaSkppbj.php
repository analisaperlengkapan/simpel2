<?php

namespace App\Models\Pengadaan\PengadaanBarangJasa;

use App\Blameable;
use App\Traits\LogTrait;
use Carbon\Carbon;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class PengadaanBarangJasaSkppbj extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'pengadaan_barang_jasa_skppbj';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'tgl_skppbj',
        'nip_penandatangan',
        'nama_penandatangan',
        'pangkat_penandatangan',
        'penyedia',
        'jabatan_penandatangan',
        'alamat',
    ];

    public function getTglSkppbjAttribute()
    {
        return Carbon::parse($this->attributes['tgl_skppbj'])->translatedFormat('d F Y');
    }
}
