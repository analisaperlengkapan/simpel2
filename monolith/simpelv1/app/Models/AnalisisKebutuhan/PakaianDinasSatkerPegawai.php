<?php

namespace App\Models\AnalisisKebutuhan;

// use Illuminate\Contracts\Auth\MustVerifyEmail;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class PakaianDinasSatkerPegawai extends Model
{
    protected $table = 'pengajuan_pakaian_dinas_satker_pegawai';
    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'pengajuan_pakaian_dinas_satker_id',
        'nama',
        'with_hijab',
        'pangkat',
        'jabatan',
        'eselon',
        'jenis_kelamin',
        'jenis',
        'gol_kd'
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

    static function getDetail($pengajuanSatkerId)
    {
        $query = DB::table('pengajuan_pakaian_dinas_satker_pegawai')
            ->select(['pengajuan_pakaian_dinas_satker_pegawai.*', 'mv_curr_pegawai_all.nama'])
            ->join('mv_curr_pegawai_all', 'pengajuan_pakaian_dinas_satker_pegawai.nip', '=', 'mv_curr_pegawai_all.peg_nip_baru')
            ->join('pengajuan_pakaian_dinas_satker', 'pengajuan_pakaian_dinas_satker.id', '=', 'pengajuan_pakaian_dinas_satker_pegawai.pengajuan_pakaian_dinas_satker_id')
            ->where(['pengajuan_pakaian_dinas_satker.id' => $pengajuanSatkerId])
            ->orderBy('mv_curr_pegawai_all.nama', 'asc')->get();
        return $query;
    }

    static function getExistingPakaianDinas(array $where = [])
    {

        $query = DB::table('mv_curr_pegawai_all_mapped')
            ->select(['mv_curr_pegawai_all_mapped.*', 'mv_curr_pegawai_all_mapped.peg_nip_baru as id', 'mv_curr_pegawai_all_mapped.peg_nip_baru as nip', 'pegawai_pakaian_dinas.ukuran_baju', 'pegawai_pakaian_dinas.ukuran_celana', 'pegawai_pakaian_dinas.ukuran_sepatu', 'pegawai_pakaian_dinas.with_hijab', 'pegawai_pakaian_dinas.last_pengajuan_pakaian_dinas_satker_pegawai_id'])
            ->leftJoin('pegawai_pakaian_dinas', 'pegawai_pakaian_dinas.nip', '=', 'mv_curr_pegawai_all_mapped.peg_nip_baru')
            ->where($where)->orderBy('mv_curr_pegawai_all_mapped.nama')->get();
        return $query;
    }
}
