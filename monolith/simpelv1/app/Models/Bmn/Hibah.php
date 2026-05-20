<?php

namespace App\Models\Bmn;

use App\Blameable;
use App\Helpers\MyHelper;
use App\Traits\LogTrait;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class Hibah extends Model
{
    use Blameable;
    use HasFactory;
    use LogTrait;

    protected $table = 'bmn_hibah';

    const tableKet = 'Pengajuan Persetujuan Penerimaan Hibah';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'id_satker',
        'kdsatker_keu',
        'jenis_hibah',
        'no_register',
        'tgl_register',
        'hibah_ke',
        'file_hibah',
        'kategori',
        'nilai',
    ];

    public function getDataGrid($paging, $search = [])
    {
        // $query = DB::table($this->table)->select('*');
        $query = DB::table($this->table.' as a');
        $query->leftJoin('ms_satker as b', 'a.kdsatker_keu', '=', 'b.kdsatker_keu');
        $query->select('a.*', 'b.inst_nama');
        $ms_satker_id = session('userData.current_role.ms_satker_id');
        if (MyHelper::isPelaksanaSatker()) {
            $query->where('b.inst_satkerkd', $ms_satker_id);
        }
        if (! empty($select)) {
            $query->select($select);
        }
        if (! empty($search)) {
            // $searchVal = strtolower($search['search']['value']);
            $searchVal = $search['columns'];
            // if (isset($search['filterBy'])) {
            //    $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            // } else {
            //    $query->where(function (Builder $q) use ($searchVal) {
            //        $q->orWhere(DB::raw('lower(kdsatker_keu)'), "like", "%{$searchVal}%")
            //            ->orWhere(DB::raw("lower(inst_nama)"), 'like', "%{$searchVal}%")
            //            ->orWhere(DB::raw("lower(no_register)"), 'like', "%{$searchVal}%")
            //            ->orWhere(DB::raw("lower(hibah_ke)"), 'like', "%{$searchVal}%");
            //    });
            // }
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
        $query->orderByDesc('created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }

    public static function findOne($id)
    {
        $query = DB::table('asset_tik_hakcipta as a');
        $query->leftJoin('ms_satker as b', 'a.kdsatker_keu', '=', 'b.kdsatker_keu');
        $query->select('a.*', 'b.inst_nama');
        $query->where('a.id', '=', $id);

        return $query->first();
    }
}
