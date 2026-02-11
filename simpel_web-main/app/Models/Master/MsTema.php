<?php

namespace App\Models\Master;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class MsTema extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'ms_setting_qr';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */

    function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table);
        $query->select('*');
        $query->where("level","1");
        if (!empty($search)) {
            $searchVal = strtolower($search['search']['value']);
            if (isset($search['filterBy'])) {
                $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            } else {
                $query->where(function (Builder $q) use ($searchVal) {
                    $q->orWhere(DB::raw('lower(name)'), "like", "%{$searchVal}%");
                });
            }
        }
        $query->orderBy('urutan','asc');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }
}
