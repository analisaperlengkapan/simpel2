<?php

namespace App\Models\Bmn;

// use Illuminate\Contracts\Auth\MustVerifyEmail;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class PenetapanSkAktifitas extends Model
{
    protected $table = 'bmn_penetapan_aktifitas';
    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'bmn_penetapan_id',
        'ms_aktifitas_id',
        'komentar',
        'nip',
        'nama',
        'pangkat',
        'jabatan',
        'role',
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

    function getGridData($paging, $search = [])
    {
        $query = DB::table($this->table)->orderBy('created_at', 'desc');

        if (!empty($search)) {
            $searchVal = strtolower($search['search']['value']);
            if (isset($search['filterBy'])) {
                $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            } else {
                $query->where(function ($q) use ($searchVal) {
                    $q->orWhere(DB::raw('lower(nama)'), "like", "%{$searchVal}%")
                        ->orWhere(DB::raw("lower(deskripsi)"), 'like', "%{$searchVal}%");
                });
            }
        }

        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }

    static function getDetail($pengajuanId)
    {
        $query = DB::table('bmn_penetapan_aktifitas as a')
            ->select(['a.*', 'b.nama as nama_aktifitas'])
            ->join('ms_aktifitas as b', 'a.ms_aktifitas_id', '=', 'b.id')
            ->where(['a.bmn_penetapan_id' => $pengajuanId])
            ->orderBy('a.created_at', 'desc')->get()->toArray();
        return $query;
    }
}