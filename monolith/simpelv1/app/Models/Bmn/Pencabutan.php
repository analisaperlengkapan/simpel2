<?php

namespace App\Models\Bmn;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class Pencabutan extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'bmn_pencabutan';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'id_satker',
        'kdsatker_keu',
        'nm_satker',
        'jenis_sk',
        'no_surat',
        'tgl_surat',
        'dikeluarkan_di',
        'file_sk',
    ];

    public function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table.' as a');
        $query->leftJoin('ms_satker as b', 'a.kdsatker_keu', '=', 'b.kdsatker_keu');
        $query->select('a.*', 'b.inst_nama');
        if (! empty($search)) {
            $searchVal = strtolower($search['search']['value']);
            if (isset($search['filterBy'])) {
                $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            } else {
                $query->where(function (Builder $q) use ($searchVal) {
                    $q->orWhere(DB::raw('lower(a.kdsatker_keu)'), 'like', "%{$searchVal}%")
                        ->orWhere(DB::raw('lower(b.inst_nama)'), 'like', "%{$searchVal}%")
                        ->orWhere(DB::raw('lower(no_surat)'), 'like', "%{$searchVal}%")
                        ->orWhere(DB::raw('lower(dikeluarkan_di)'), 'like', "%{$searchVal}%");
                });
            }
        }
        $query->orderByDesc('created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }

    public static function findOne($id)
    {
        $query = DB::table('asset_tik_hakcipta as a');
        $query->leftJoin('ms_satker as b', 'a.kdsatker_keu', '=', 'b.kdsatker_keu');
        $query->select('a.*', 'b.inst_nama');
        $query->where('a.id', '=', $id);

        return $query->first();
    }
}
