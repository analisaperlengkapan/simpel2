<?php

namespace App\Models\Master;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class MsIntegrasiData extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'ms_integrasi_data';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'host',
        'username',
        'password',
        'nama_aplikasi',
    ];

    public function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table);
        $query->select('*');
        if (! empty($search)) {
            $searchVal = strtolower($search['search']['value']);
            if (isset($search['filterBy'])) {
                $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            } else {
                $query->where(function (Builder $q) use ($searchVal) {
                    $q->orWhere(DB::raw('lower(host)'), 'like', "%{$searchVal}%");
                    $q->orWhere(DB::raw('lower(username)'), 'like', "%{$searchVal}%");
                    $q->orWhere(DB::raw('lower(nama_aplikasi)'), 'like', "%{$searchVal}%");
                });
            }
        }
        $query->orderByDesc('created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }
}
