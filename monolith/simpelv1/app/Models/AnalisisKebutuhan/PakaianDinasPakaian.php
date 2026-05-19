<?php

namespace App\Models\AnalisisKebutuhan;

// use Illuminate\Contracts\Auth\MustVerifyEmail;

use Illuminate\Database\Eloquent\Model;

class PakaianDinasPakaian extends Model
{
    protected $table = 'pengajuan_pakaian_dinas_pakaian';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'pengajuan_pakaian_dinas_id',
        'jenis_pakaian_id',
        'jenis_pakaian_nama',
        'spesifikasi_id',
        'spesifikasi_nama',
        'spesifikasi_ukuran_group',
        'subspesifikasi_id',
        'subspesifikasi_nama',
        'subspesifikasi_gender',

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
