<?php

namespace App\Models\Asset;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class LaporMasalah extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'asset_lapor_masalah';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'kategori',
        'id_asset',
        'permasalahan',
        'tindak_lanjut',
        'id_status',
    ];

    public function getDataGrid($paging, $search, $kategori, $id_asset)
    {
        $query = DB::table($this->table.' as a');
        $query->leftJoin('ms_status_lapor_masalah as b', 'a.id_status', '=', 'b.id');
        $query->select('a.*', 'b.status');
        $query->where('a.kategori', '=', $kategori);
        $query->where('a.id_asset', '=', $id_asset);
        if (! empty($search)) {
            $searchVal = strtolower($search['search']['value']);
            if (isset($search['filterBy'])) {
                $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            } else {
                $query->where(function (Builder $q) use ($searchVal) {
                    $q->orWhere(DB::raw('lower(b.status)'), 'like', "%{$searchVal}%");
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
        $query = DB::table('asset_lapor_masalah as a');
        $query->leftJoin('ms_status_lapor_masalah as b', 'a.id_status', '=', 'b.id');
        $query->select('a.*', 'b.status');
        $query->where('a.id', '=', $id);

        return $query->first();
    }
}
