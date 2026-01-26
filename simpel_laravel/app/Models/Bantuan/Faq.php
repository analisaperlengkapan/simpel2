<?php

namespace App\Models\Bantuan;

use App\Blameable;
use App\Traits\LogTrait;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class Faq extends Model
{
    use HasFactory;
    use Blameable;
    use LogTrait;
    protected $table = 'support_faq';
    const tableKet = 'manajemen FAQ';
    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'pertanyaan',
        'jawaban',
        'platform',
        'status'
    ];

    function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table)->select('*');
        if(!empty($select)) $query->select($select);
        if (!empty($search)) {
            $searchVal = $search['columns'];
            //if (isset($search['filterBy'])) {
            //    $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            // } else {
            //    $query->where(function (Builder $q) use ($searchVal) {
            //        $q->orWhere(DB::raw("lower(pertanyaan)"), 'like', "%{$searchVal}%")
            //            ->orWhere(DB::raw("lower(jawaban)"), 'like', "%{$searchVal}%");
            //             if($searchVal && is_numeric($searchVal))
            //            $q->orWhere('kategori', "=", $searchVal);
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
