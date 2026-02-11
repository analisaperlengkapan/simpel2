<?php

namespace App\Models\Master\PakaianDinas;

use App\Blameable;
use App\Traits\LogTrait;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class SubSpesifikasiPakaianDinas extends Model
{
    use HasFactory, Blameable, LogTrait;

    protected $table = 'ms_subspesifikasi_pakaian_dinas';
    const tableKet = 'Referensi Subspesifikasi Pakaian Dinas';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'id_spesifikasi',
        'gender',
        'nama'
    ];

    function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table . ' as a');
        $query->leftJoin('ms_spesifikasi_pakaian_dinas as b', 'a.id_spesifikasi', '=', 'b.id');
        $query->leftJoin('ms_jenis_pakaian_dinas as c', 'c.id', '=', 'b.ms_jenis_pakaian_dinas_id');
        $query->select('a.*', 'b.nama as nm_spesifikasi', 'c.nama as jenis_pakaian', 'c.id as ms_jenis_pakaian_id');
        if (!empty($search)) {
            $searchVal = strtolower($search['search']['value']);
            if (isset($search['filterBy'])) {
                $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            } else {
                $query->where(function (Builder $q) use ($searchVal) {
                    $q->orWhere(DB::raw('lower(a.nama)'), "like", "%{$searchVal}%")
                        ->orWhere(DB::raw('lower(b.nama)'), "like", "%{$searchVal}%")
                        ->orWhere(DB::raw('lower(c.nama)'), "like", "%{$searchVal}%");
                });
            }
        }
        $query->orderByDesc('a.created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }

    function getComplete($ids)
    {
        $query = DB::table($this->table . ' as a');
        $query->leftJoin('ms_spesifikasi_pakaian_dinas as b', 'a.id_spesifikasi', '=', 'b.id');
        $query->leftJoin('ms_jenis_pakaian_dinas as c', 'c.id', '=', 'b.ms_jenis_pakaian_dinas_id');
        $query->select('a.*', 'b.nama as nm_spesifikasi', 'b.ms_ukuran_group', 'c.nama as jenis_pakaian', 'c.id as ms_jenis_pakaian_id')
            ->whereIn('a.id', $ids);
        return $query->get();
    }
}