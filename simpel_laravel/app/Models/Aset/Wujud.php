<?php

namespace App\Models\Aset;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class Wujud extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'siman.siman_aset_tak_berwujud_kl';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'id_satker',
        'kdsatker_keu',
        'nm_satker',
        'id_barang',
        'kode_barang',
        'nm_barang',
        'nup',
        'kondisi',
        'merk',
        'tgl_rekam_pertama',
        'tgl_perolehan',
        'nilai_perolehan_pertama',
        'nilai_mutasi',
        'nilai_perolehan',
        'nilai_penyusutan',
        'nilai_buku',
        'kuantitas',
        'jml_foto',
        'status_penggunaan',
        'status_pengelolaan',
        'no_psp',
        'tgl_psp'
    ];

    function getDataGrid($paging, $search = [], $select = [])
    {
        $query = DB::table($this->table.' as a');
        if(!empty($select)){
            $query->select($select,'b.inst_nama');
        }else{
            $query->select("a.*", 'b.inst_nama');
        }
        $query->leftJoin('ms_satker as b','a.id_satker_keu','=','b.kdsatker_keu');

        $roleSatker = session('userData.current_role.ms_satker_id');
        $roleSadmin = session('userData.current_role.ms_role_id');

        //return $roleSatker;exit;

        if ($roleSatker != '00' && $roleSadmin != '3') {
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
                            $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
                        }
                    }
                }
            });
        }
        $query->orderByDesc('tanggal_perolehan');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }

    function getDataExport($search = [],$defColumn = null)
    {
        $select = array("a.kdsatker_keu", 'a.nm_satker', 'a.kode_barang', 'a.nm_barang', 'a.nup', 'a.kondisi',
        'a.merk', 'a.tgl_rekam_pertama','a.tgl_perolehan','a.nilai_perolehan_pertama','a.nilai_mutasi','a.nilai_perolehan','a.nilai_penyusutan','a.nilai_buku','a.kuantitas',
        'a.jml_foto','a.status_penggunaan','a.status_pengelolaan','a.no_psp','a.tgl_psp');
        if($defColumn) $select = array_intersect_key($select, array_flip($defColumn));
        $query = DB::table($this->table.' as a');
        $query->select($select);
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
        $query->orderByDesc('a.created_at');
        $data = $query->get();
        return ['data' => $data];
    }

    static function findOne($id){
        $query = DB::table('asset_tak_berwujud as a');
        $query->select('a.*');
        $query->where('a.id', '=', $id);
        return $query->first();
    }
}
