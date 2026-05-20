<?php

namespace App\Models\Monsakti;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class KontrakHeader extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'monsakti_kontrak_header';

    public function getDataGrid($paging, $search = [])
    {
        DB::enableQueryLog();

        $query = DB::table($this->table.' as a');
        $query->select('a.*', 'b.inst_nama');
        $query->leftJoin('ms_satker as b', 'a.kdsatker', '=', 'b.kdsatker_keu');
        $roleSatker = session('userData.current_role.ms_satker_id');
        if ($roleSatker != '00') {
            $query->where('b.inst_satkerkd', 'like', "{$roleSatker}%");
        }
        if (! empty($search)) {
            $searchVal = $search['columns'];

            // echo "<pre>"; print_r($searchVal);exit;

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

                            if ($columnName == 'nilai_kontrak' && $value) {
                                if ($value == 'duaratus') {
                                    $q->where(DB::raw('cast(a.nilai_kontrak as numeric)'), '<=', 200000000);
                                } elseif ($value == 'duaratus_lebih') {
                                    $q->where(DB::raw('cast(a.nilai_kontrak as numeric)'), '>', 200000000);
                                }
                            } elseif ($columnName == 'inst_nama' && $value) {
                                if ($value) {
                                    $q->where('a.kdsatker', '=', $value);
                                }
                            } elseif ($value) {
                                $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
                            }
                        }
                    }
                }
            });
        }
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        // $querys = DB::getQueryLog();
        // dd($querys);exit;

        return ['total' => $total, 'data' => $data];
    }

    public function getDataLine($id_kontrak)
    {
        $query = DB::table('monsakti_kontrak_line as a');
        $query->where('a.id_kontrak', $id_kontrak);

        return $query->get();
    }

    public function getDataTermin($id_kontrak)
    {
        $query = DB::table('monsakti_kontrak_termin as a');
        $query->where('a.id_kontrak', $id_kontrak);
        $query->orderBy('a.termin_ke');

        return $query->get();
    }

    public function getDataBast($no_kontrak)
    {
        $query = DB::table('monsakti_bast_kontrak_header as a');
        $query->where('a.no_kontrak', $no_kontrak);
        $query->orderBy('a.id_bast');

        return (array) $query->first();
    }

    public function getDataBastDetail($id_bast)
    {
        $query = DB::table('monsakti_bast_kontrak_detail as a');
        $query->where('a.id_bast', $id_bast);
        $query->orderBy('a.kode_barang');

        return $query->get();
    }

    public static function getAll()
    {
        $sql = 'SELECT * FROM monsakti_kontrak_header';
        $result = DB::select($sql);

        return $result;
    }
}
