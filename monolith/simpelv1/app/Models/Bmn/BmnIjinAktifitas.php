<?php

namespace App\Models\Bmn;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class BmnIjinAktifitas extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'bmn_ijin_pemakaian_aktifitas';

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

    public static function getDetail($pengajuan_id)
    {
        $query = DB::table('bmn_ijin_pemakaian_aktifitas as a');
        $query->select('a.*', 'b.nama as nama_aktifitas');
        $query->join('ms_aktifitas_user as b', 'a.ms_aktifitas_id', '=', 'b.id');
        $query->where('a.pengajuan_id', $pengajuan_id);
        $query->orderBy('a.created_at', 'desc');

        return $query->get()->toArray();
    }
}
