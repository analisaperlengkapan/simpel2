<?php

namespace App\Models;

// use Illuminate\Contracts\Auth\MustVerifyEmail;

use App\Traits\LogTrait;
use Illuminate\Database\Eloquent\Model;

class PegawaiPakaianDinas extends Model
{
    use LogTrait;
    protected $table = 'pegawai_pakaian_dinas';
    const tableKet = "Ukuran Pakaian Pegawai";
    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'nip',
        'ukuran_baju',
        'ukuran_celana',
        'ukuran_sepatu',
        'with_hijab',
        'pangkat',
        'jabatan',
        'status',
        'last_pengajuan_pakaian_dinas_satker_pegawai_id',
    ];
}
