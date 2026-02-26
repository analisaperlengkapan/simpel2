<?php

namespace App\Models\Suport;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class UserBantuanAktifitas extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'suport_helpdesk_aktifitas';

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
        $query = DB::table('suport_helpdesk_aktifitas as a');
        $query->select('a.*', 'b.name as nama_user','c.name as role');
        $query->join('users as b', 'a.nama', '=', 'b.id');
		$query->join('ms_role as c', 'a.role', '=', 'c.id');
        $query->where('a.pengajuan_id', $pengajuan_id);
        $query->orderBy('a.created_at', 'desc');
        return $query->get()->toArray();
    }
}
