<?php

namespace App\Models\Sdm;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class SdmPengelola extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'mv_curr_pegawai_all_mapped';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'peg_nip_baru',
        'nama',
        'inst_satkerkd',
        'nama',
        'pangkat',
        'jabatan',
    ];

    function getDataGrid($paging, $search = [])
    {
        $queryx = DB::table('sdm_pengadaan'.' as a');
        $queryx->select('a.nip');
        if(in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'))){
            $ms_satker_idx = session('userData.current_role.ms_satker_id_keu');
            $queryx->where('a.kdsatker', $ms_satker_idx);
        }
        $sdmpengadaan = $queryx->get();
        $datasdm = collect($sdmpengadaan)->map(function($x){ return (array) $x; })->toArray();

        $query = DB::table($this->table.' as a');
        $query->leftJoin('ms_satker as b', 'a.inst_satkerkd', '=', 'b.inst_satkerkd');
        $query->select('a.*', 'b.inst_nama');
        $ms_satker_id = session('userData.current_role.ms_satker_id');
        if(in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'))){
            $query->where('a.inst_satkerkd', $ms_satker_id);
        }

        // if (!empty($search)) {
        //     $searchVal = strtolower($search['search']['value']);
        //     if (isset($search['filterBy'])) {
        //         $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
        //     } else {
        //         $query->where(function (Builder $q) use ($searchVal) {
        //             $q->orWhere(DB::raw('lower(a.satker)'), "like", "%{$searchVal}%")
        //                 ->orWhere(DB::raw("lower(b.inst_nama)"), 'like', "%{$searchVal}%")
        //                 ->orWhere(DB::raw("lower(peg_nip_baru)"), 'like', "%{$searchVal}%")
        //                 ->orWhere(DB::raw("lower(nama)"), 'like', "%{$searchVal}%");
        //         });
        //     }
        // }

        if (!empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach($searchVal as $k => $v){
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    $kolom = $columnName=='inst_nama'?'b.'.$columnName:'a.'.$columnName;
                    $tableName = $columnName=='inst_nama'?'ms_satker':$this->table;
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

        $query->whereIn('peg_nip_baru', $datasdm );
        $query->orderByDesc('satker');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }

    static function findOne($id){
        $query = DB::table('sdm_pengelola as a');
        $query->leftJoin('ms_satker as b', 'a.kdsatker_keu', '=', 'b.kdsatker_keu');
        $query->select('a.*', 'b.inst_nama');
        $query->where('a.id', '=', $id);
        return $query->first();
    }

    function getdatapengadaan($nip = "")
    {
        $queryx = DB::table('sdm_pengadaan'.' as a');
        $queryx->select('a.*');
        if(in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'))){
            $ms_satker_idx = session('userData.current_role.ms_satker_id_keu');
            $queryx->where('a.kdsatker', $ms_satker_idx);
        }

        if($nip){
            $queryx->where('a.nip', $nip);
        }

        $sdmpengadaan = $queryx->get();
        $datasdm = collect($sdmpengadaan)->map(function($x){ return (array) $x; })->toArray();

        return $datasdm;
    }
}
