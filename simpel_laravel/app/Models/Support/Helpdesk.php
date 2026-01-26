<?php

namespace App\Models\Support;

use App\Traits\LogTrait;
use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class Helpdesk extends Model
{
    use HasFactory;
    use Blameable;
    use LogTrait;
    protected $table = 'support_helpdesk';
    const tableKet = 'Pengajuan Tiket Helpdesk';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'kode_tiket',
        'kode_satker',
        'judul',
        'tipe_tiket',
        'deskripsi',
        'attachment',
        'image',
        'status',
        'catatan',
		'tgl_pengajuan',
        'topik',
        'tujuan'
    ];

    function getDataGrid($paging, $search = [])
    {

        $query = DB::table($this->table)->select('*');
        if(!empty($select)) $query->select($select);
        if (!empty($search)) {
            $searchVal = $search['columns'];
            //$searchVal = strtolower($search['search']['value']);
            //if (isset($search['filterBy'])) {
            //    if($search['filterBy']=='kode_tiket'){
            //        if(is_numeric($searchVal))
            //            $query->where(DB::raw($search['filterBy']), '=', "{$searchVal}");
            //    }else{
            //        $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            //    }
            //} else {
            //    $query->where(function (Builder $q) use ($searchVal) {
            //        $q->orWhere(DB::raw("lower(judul)"), 'like', "%{$searchVal}%")
            //            ->orWhere(DB::raw("lower(tipe_tiket)"), 'like', "%{$searchVal}%")
            //            ->orWhere(DB::raw("lower(deskripsi)"), 'like', "%{$searchVal}%");
            //        if($searchVal && is_numeric($searchVal))
            //            $q->orWhere('kode_tiket', "=", $searchVal);
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

        // echo "<pre>";
        // print_r(session('userData'));exit;

        if(session('userData.current_role.ms_role_id') == 1){
            $kode_satker = session('userData.current_role.ms_satker_id_keu');
            if (!empty($kode_satker)) {
                $query->where('kode_satker','=',$kode_satker);
            }
        }elseif(session('userData.current_role.ms_role_id') == 20){
            $kode_satker = session('userData.current_role.ms_satker_id_keu');
            if (!empty($kode_satker)) {
                $query->where('kode_satker','=',$kode_satker);
            }
        }elseif(session('userData.current_role.ms_role_id') == 21){
            $query->where('tujuan','=',session('userData.current_role.ms_role_id'));
        }

        $query->orderByDesc('created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }
}
