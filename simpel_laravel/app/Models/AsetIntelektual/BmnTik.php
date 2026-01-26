<?php

namespace App\Models\AsetIntelektual;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class BmnTik extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'vw_asset_tak_wujud_tik';

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


    function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table.' as a');
        $query->leftJoin('ms_satker_sakti as b', 'a.kdsatker_keu', '=', 'b.kdsatker');
        $query->select('a.*', 'b.deskripsi as inst_nama',DB::raw("CONCAT(b.kode_unit,'.',b.kdsatker,'.',RIGHT(b.kode_uappbw,6)) as kdsatker"));
        $ms_satker_id = session('userData.current_role.ms_satker_id_keu');
        if(in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'))){
            $query->where('a.id_satker', $ms_satker_id);
        }
        if (!empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach($searchVal as $k => $v){
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    if($columnName=='inst_nama'){
                        $kolom = 'b.deskripsi';
                    }else if($columnName=='kdsatker'){
                        $kolom = "(b.kode_unit||'.'||b.kdsatker||'.'||RIGHT(b.kode_uappbw,6))";
                    }else{
                        $kolom = 'a.'.$columnName;
                    }
                    $tableName = $columnName=='kdsatker'||$columnName=='inst_nama'?'ms_satker_sakti':$this->table;
                    if($value){
                        $dataType = DB::table('information_schema.columns')->select('data_type')->where('table_name', $tableName)->where('column_name', $columnName)->value('data_type');
                        if (in_array($dataType, ['integer', 'numeric', 'smallint', 'bigint'])) {
                            $q->where($kolom, '=', $value);
                        } elseif (in_array($dataType, ['timestamp without time zone', 'timestamp', 'date'])) {
                            if (strtotime($value)) {
                                $q->whereDate($kolom, '=', date('Y-m-d', strtotime($value)));
                            }
                        } else {
                            $q->where(DB::raw("lower({$kolom})"), 'like', strtolower("%{$value}%"));
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

    function findOne($id){
        $query = DB::table('vw_asset_tak_wujud_tik as a');
        $query->leftJoin('ms_satker as b', 'a.kdsatker_keu', '=', 'b.kdsatker_keu');
        $query->select('a.*', 'b.inst_nama');
        $query->where('a.id', '=', $id);
        return $query->first();
    }
}
