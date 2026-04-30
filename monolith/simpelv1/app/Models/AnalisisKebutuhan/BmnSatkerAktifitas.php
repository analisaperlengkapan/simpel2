<?php

namespace App\Models\AnalisisKebutuhan;

// use Illuminate\Contracts\Auth\MustVerifyEmail;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class BmnSatkerAktifitas extends Model
{
    use HasFactory, Blameable;
    protected $table = 'pengajuan_kebutuhan_bmn_satker_aktifitas';
    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'pengajuan_kebutuhan_bmn_satker_id',
        'ms_aktifitas_id',
        'komentar',
        'nip',
        'pangkat',
        'jabatan',
        'created_at',
        'updated_at',
        'role',
        'ms_satker_id',
        'nama',
        'created_by',
        'updated_by',
    ];

    static function getDetail($pengajuan_id)
    {
        $query = DB::table('pengajuan_kebutuhan_bmn_satker_aktifitas as a');
        $query->select('a.*', 'b.nama as nama_aktifitas');
        $query->join('ms_aktifitas_user as b', 'a.ms_aktifitas_id', '=', 'b.id');
        $query->where('a.pengajuan_kebutuhan_bmn_satker_id', $pengajuan_id);
        $query->orderBy('a.created_at', 'desc');
        return $query->get()->toArray();
    }
}
