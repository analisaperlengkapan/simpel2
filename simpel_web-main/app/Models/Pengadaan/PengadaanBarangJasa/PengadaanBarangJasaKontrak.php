<?php

namespace App\Models\Pengadaan\PengadaanBarangJasa;

use App\Blameable;
use Carbon\Carbon;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class PengadaanBarangJasaKontrak extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'pengadaan_barang_jasa_kontrak';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'no_kontrak',
        'tgl_kontrak',
    ];

    public function getTglKontrakAttribute()
    {
        return Carbon::parse($this->attributes['tgl_kontrak'])->isoFormat('dddd D MMMM Y');
    }
}
