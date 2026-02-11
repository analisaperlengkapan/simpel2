<?php

namespace App\Models\Master\PakaianDinas;

use App\Traits\LogTrait;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class JenisPakaianDinas extends Model
{
    use LogTrait;

    protected $table = 'ms_jenis_pakaian_dinas';
    const tableKet = "Referensi Jenis Pakaian Dinas";

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'nama'
    ];

    function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table);
        $query->select('*');
        if(!empty($select)) $query->select($select);
        if (!empty($search)) {
            //$searchVal = strtolower($search['search']['value']);
            $searchVal = $search['columns'];
            //if (isset($search['filterBy'])) {
            //    $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            //} else {
            //    $query->where(function (Builder $q) use ($searchVal) {
            //        $q->orWhere(DB::raw('lower(nama)'), "like", "%{$searchVal}%");
            //    });
            //}
            $query->where(function (Builder $q) use ($searchVal) {
                foreach($searchVal as $k => $v){
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    $tableName = $this->table;
                    if($value){
                        $dataType = DB::table('information_schema.columns')->select('data_type')->where('table_name', $tableName)->where('column_name', $columnName)->value('data_type');
                        if (in_array($dataType, ['integer', 'numeric', 'smallint', 'bigint'])) {
                            $q->where($columnName, '=', $value);
                        } elseif (in_array($dataType, ['timestamp without time zone', 'timestamp', 'date'])) {
                            if (strtotime($value)) {
                                $q->whereDate($columnName, '=', date('Y-m-d', strtotime($value)));
                            }
                        } else {
                            $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
                        }
                    }
                }
            });
        }
        $query->orderByDesc('created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }
}
