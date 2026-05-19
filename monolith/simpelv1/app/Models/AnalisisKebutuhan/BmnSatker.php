<?php

namespace App\Models\AnalisisKebutuhan;

// use Illuminate\Contracts\Auth\MustVerifyEmail;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class BmnSatker extends Model
{
    use Blameable, HasFactory;

    protected $table = 'pengajuan_kebutuhan_bmn_satker';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'pengajuan_kebutuhan_bmn_id',
        'id_satker_keu',
        'id_kejati',
        'id_kejari',
        'id_cabjari',
        'created_by',
        'updated_by',
        'ms_aktifitas_id',
        'created_at',
        'updated_at',
        'ms_satker_id',
        'ms_satker_pusat_id',
        'prioritas',
    ];
}
