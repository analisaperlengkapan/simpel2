<?php

namespace App\Models\Sdm;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class Sdmpengadaan extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'sdm_pengadaan';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'kode_unit',
        'kdsatker',
        'nip',
        'nama',
        'telpon',
        'email',
        'jabatan',
        'nik',
        'npwp',
        'tgl_sertifikat',
        'file_sertifikat',
    ];

    public function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table.' as a');
        $query->leftJoin('ms_satker as b', 'a.kdsatker', '=', 'b.kdsatker_keu');
        $query->select('a.*', 'b.inst_nama');
        $ms_satker_id = session('userData.current_role.ms_satker_id_keu');
        if (in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'))) {
            $query->where('a.kdsatker', $ms_satker_id);
        }
        // if (!empty($search)) {
        //     $searchVal = strtolower($search['search']['value']);
        //     if (isset($search['filterBy'])) {
        //         $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
        //     } else {
        //         $query->where(function (Builder $q) use ($searchVal) {
        //             $q->orWhere(DB::raw('lower(a.kdsatker_keu)'), "like", "%{$searchVal}%")
        //                 ->orWhere(DB::raw("lower(b.inst_nama)"), 'like', "%{$searchVal}%")
        //                 ->orWhere(DB::raw("lower(nip)"), 'like', "%{$searchVal}%")
        //                 ->orWhere(DB::raw("lower(nama)"), 'like', "%{$searchVal}%");
        //         });
        //     }
        // }
        if (! empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    $kolom = $columnName == 'inst_nama' ? 'b.'.$columnName : 'a.'.$columnName;
                    $tableName = $columnName == 'inst_nama' ? 'ms_satker' : $this->table;
                    if ($value) {
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

    public static function findOne($nip)
    {
        $query = DB::table('sdm_pengadaan as a');
        $query->leftJoin('ms_satker as b', 'a.kdsatker', '=', 'b.kdsatker_keu');
        $query->select('a.*', 'b.inst_nama');
        $query->where('a.nip', '=', $nip);

        return $query->first();
    }
}
