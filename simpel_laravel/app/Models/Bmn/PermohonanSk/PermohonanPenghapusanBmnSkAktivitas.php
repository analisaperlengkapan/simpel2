<?php

namespace App\Models\Bmn\PermohonanSk;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class PermohonanPenghapusanBmnSkAktivitas extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'permohonan_penghapusan_bmn_sk_aktivitas';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'pengajuan_id',
        'ms_aktifitas_id',
        'komentar',
        'nip',
        'pangkat',
        'jabatan',
        'role',
        'ms_satker_id',
        'nama',
    ];

    static function getDetail($pengajuan_id)
    {
        $query = DB::table('permohonan_penghapusan_bmn_sk_aktifitas as a');
        $query->select('a.*',
        DB::raw("CASE WHEN a.ms_aktifitas_id = 3006 THEN 'Surat Keputusan Terbit'
        ELSE b.nama END as nama_aktifitas"));
        $query->join('ms_aktifitas_user as b', 'a.ms_aktifitas_id', '=', 'b.id');
        $query->where('a.pengajuan_id', $pengajuan_id);
        $query->orderBy('a.created_at', 'desc');
        return $query->get()->toArray();
    }
}
