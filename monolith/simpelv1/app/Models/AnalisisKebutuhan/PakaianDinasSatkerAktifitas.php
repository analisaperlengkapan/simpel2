<?php

namespace App\Models\AnalisisKebutuhan;

// use Illuminate\Contracts\Auth\MustVerifyEmail;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class PakaianDinasSatkerAktifitas extends Model
{
    protected $table = 'pengajuan_pakaian_dinas_satker_aktifitas';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'pengajuan_pakaian_dinas_satker_id',
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

    public function getGridData($paging, $search = [])
    {
        $query = DB::table($this->table)->orderBy('created_at', 'desc');

        if (! empty($search)) {
            $searchVal = strtolower($search['search']['value']);
            if (isset($search['filterBy'])) {
                $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            } else {
                $query->where(function ($q) use ($searchVal) {
                    $q->orWhere(DB::raw('lower(nama)'), 'like', "%{$searchVal}%")
                        ->orWhere(DB::raw('lower(deskripsi)'), 'like', "%{$searchVal}%");
                });
            }
        }

        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }

    public static function getDetail($pengajuanId)
    {
        $query = DB::table('pengajuan_pakaian_dinas_satker_aktifitas')
            ->select(['pengajuan_pakaian_dinas_satker_aktifitas.*', 'ms_aktifitas.nama as nama_aktifitas'])
            ->join('ms_aktifitas', 'pengajuan_pakaian_dinas_satker_aktifitas.ms_aktifitas_id', '=', 'ms_aktifitas.id')
            ->where(['pengajuan_pakaian_dinas_satker_id' => $pengajuanId])
            ->orderBy('pengajuan_pakaian_dinas_satker_aktifitas.created_at', 'desc')->get()->toArray();

        return $query;
    }
}
