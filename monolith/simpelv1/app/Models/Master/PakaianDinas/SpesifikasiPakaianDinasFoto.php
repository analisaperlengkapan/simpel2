<?php

namespace App\Models\Master\PakaianDinas;

use Illuminate\Database\Eloquent\Model;

class SpesifikasiPakaianDinasFoto extends Model
{
    protected $table = 'ms_spesifikasi_pakaian_dinas_foto';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'ms_spesifikasi_pakaian_dinas_id',
        'path',
        'filename',
    ];
}
