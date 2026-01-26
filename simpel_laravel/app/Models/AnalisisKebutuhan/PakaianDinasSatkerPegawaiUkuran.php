<?php

namespace App\Models\AnalisisKebutuhan;

// use Illuminate\Contracts\Auth\MustVerifyEmail;

use Illuminate\Database\Eloquent\Model;

class PakaianDinasSatkerPegawaiUkuran extends Model
{
    protected $table = 'pengajuan_pakaian_dinas_satker_pegawai_ukuran';
    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'pengajuan_pakaian_dinas_satker_id',
        'pengajuan_pakaian_dinas_satker_pegawai_id',
        'pengajuan_pakaian_dinas_pakaian_id',
        'ukuran',
    ];

    /**
     * The attributes that should be hidden for serialization.
     *
     * @var array<int, string>
     */

    /**
     * The attributes that should be cast.
     *
     * @var array<string, string>
     */
    // protected $casts = [
    //     'email_verified_at' => 'datetime',
    //     'password' => 'hashed',
    // ];
}