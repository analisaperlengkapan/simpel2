<?php

namespace App\Models\Monsakti;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class TransaksiAset extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'monsakti_trx_aset';

    public function getDataGrid($paging, $search = [], $select = [])
    {
        $query = DB::table($this->table.' as a');
        if (! empty($select)) {
            $query->select($select);
        }
        if (! empty($search)) {
            $searchVal = $search['columns'];
            $query->where('kdbrg', $search['kode_barang']);
            $query->where('nup', $search['nup']);
            $query->where('kdsatker', $search['kdsatker']);
            // $query->where('kduakpb',$search['kduakpb']);
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    $tableName = $this->table;
                    if ($value) {
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
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }
}
