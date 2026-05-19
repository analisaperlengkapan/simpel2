<?php

namespace App\Models\Master;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class MsTema extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'ms_setting_qr';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    public function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table);
        $query->select('*');
        $query->where('level', '1');
        if (! empty($search)) {
            $searchVal = strtolower($search['search']['value']);
            if (isset($search['filterBy'])) {
                $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            } else {
                $query->where(function (Builder $q) use ($searchVal) {
                    $q->orWhere(DB::raw('lower(name)'), 'like', "%{$searchVal}%");
                });
            }
        }
        $query->orderBy('urutan', 'asc');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }
}
