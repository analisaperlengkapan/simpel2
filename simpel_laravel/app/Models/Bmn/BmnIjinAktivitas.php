<?php

namespace App\Models\Bmn;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class BmnIjinAktivitas extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'bmn_ijin_pemakaian_aktivitas';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'pengajuan_id',
        'ms_aktivitas_id',
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
        $query = DB::table('bmn_ijin_pemakaian_aktivitas as a');
        $query->select('a.*', 'b.nama as nama_aktivitas');
        $query->join('ms_aktivitas_user as b', 'a.ms_aktivitas_id', '=', 'b.id');
        $query->where('a.pengajuan_id', $pengajuan_id);
        $query->orderBy('a.created_at', 'desc');
        return $query->get()->toArray();
    }
}
