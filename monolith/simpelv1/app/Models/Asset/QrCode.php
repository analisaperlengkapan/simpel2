<?php

namespace App\Models\Asset;

use App\Blameable;
use App\Traits\LogTrait;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class QrCode extends Model{

    use HasFactory, Blameable, LogTrait;

    protected $table = 'ms_setting_qr';
    const tableKet = 'Setting QR Code';

    function getDataGrid($tableAsset, $inst_satkerkd, $paging, $search = [])
    {
        if($tableAsset == '') return ['total' => 0, 'data' => []];

        $query = DB::table($tableAsset.' as a');
        $query->select('a.*','b.deskripsi');
        $query->join('ms_satker_sakti as b','a.id_satker','=','b.kdsatker','left');
        if (!empty($search)) {
            $searchVal = $search['columns'];
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
        $query->where('a.id_satker',$inst_satkerkd);
        $query->orderByDesc('tgl_perolehan');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }

    function getDataCetak($tableAsset, $id)
    {
        $query = DB::table($tableAsset.' as a');
        $query->select('a.*','b.deskripsi');
        $query->join('ms_satker_sakti as b','a.id_satker','=','b.kdsatker','left');
        $query->whereIn('a.id',$id);
        $query->orderByDesc('created_at');
        $data = $query->get();
        return $data;
    }

}
