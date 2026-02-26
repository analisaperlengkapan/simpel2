<?php

namespace App\Models\Monsakti;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class BastNonKontrakHeader extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'monsakti_bast_nonkontrak_header';

    function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table.' as a');
        $query->select('a.*','b.inst_nama');
        $query->leftJoin('ms_satker as b','a.kdsatker','=','b.kdsatker_keu');
        $roleSatker = session('userData.current_role.ms_satker_id');
        if ($roleSatker != '00') {
            $query->where('b.inst_satkerkd', 'like', "{$roleSatker}%");
        }
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

                            if ($columnName == 'nilai_bast' && $value) {
                                if($value == 'seratus'){
                                    $q->where(DB::raw('cast(a.nilai_bast as numeric)'), '<=', 200000000);
                                }elseif($value == 'seratus_lebih'){
                                    $q->where(DB::raw('cast(a.nilai_bast as numeric)'), '>', 200000000);
                                }
                            }else if ($columnName == 'inst_nama' && $value) {
                                if($value){
                                    $q->where('a.kdsatker', '=', $value);
                                }
                            }else if($value){
                                $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
                            }
                            
                        }
                    }
                }
            });
        }
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }

    function getDetail($id_kontrak){
        $query = DB::table('monsakti_bast_nonkontrak_detail as a');
        $query->where('a.id_bast', $id_kontrak);
        return $query->get();
    }
}
