<?php

namespace App\Models\Pengadaan\PengadaanBarangJasa;

use App\Blameable;
use Carbon\Carbon;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class PengadaanBarangJasaNodis extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'pengadaan_barang_jasa_nodis';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'no_nodis',
        'tgl_nodis',
        'yth',
        'selaku',
        'dari',
        'sifat',
        'lampiran',
        'hal',
        'nama_pejabat',
        'nip_pejabat',
        'pangkat_pejabat',
        'jabatan_pejabat',
    ];

    public function getTglNodisAttribute()
    {
        return Carbon::parse($this->attributes['tgl_nodis'])->isoFormat('dddd D MMMM Y');
    }
}
