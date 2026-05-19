<?php

namespace App\Models\AnalisisKebutuhan;

// use Illuminate\Contracts\Auth\MustVerifyEmail;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class BmnSatkerBarang extends Model
{
    use Blameable, HasFactory;

    protected $table = 'pengajuan_kebutuhan_bmn_satker_barang';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'nama',
        'jumlah',
        'alasan',
        'file_pendukung',
        'skor',
        'created_at',
        'updated_at',
        'pengajuan_kebutuhan_bmn_satker_id',
        'created_by',
        'updated_by',
        'keterangan',
        'jml_setuju',
        'prioritas',
        'kode_barang',
    ];
}
