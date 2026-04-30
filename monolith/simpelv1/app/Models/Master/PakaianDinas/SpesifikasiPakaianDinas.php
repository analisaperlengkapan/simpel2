<?php

namespace App\Models\Master\PakaianDinas;

use App\Blameable;
use App\Traits\LogTrait;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class SpesifikasiPakaianDinas extends Model
{
    use HasFactory, Blameable, LogTrait;


    protected $table = 'ms_spesifikasi_pakaian_dinas';
    const tableKet = 'Referensi Spesifikasi Pakaian Dinas';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'ms_jenis_pakaian_dinas_id',
        'ms_ukuran_group',
        'nama',
        'gender'

    ];

    function getDataGrid($paging, $search = [], $isRaw = false)
    {
        $query = DB::table("{$this->table} as a")->join('ms_jenis_pakaian_dinas as b', "a.ms_jenis_pakaian_dinas_id", '=', 'b.id');
        $query->select(["a.*", 'b.nama as jenis']);
        if ($isRaw) {
            return $query->orderBy('ms_jenis_pakaian_dinas.nama')->get();
        }
        if (!empty($select))
            $query->select($select);
        if (!empty($search)) {
            //$searchVal = strtolower($search['search']['value']);
            $searchVal = $search['columns'];
            //if (isset($search['filterBy'])) {
            //     $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            //} else {
            //    $query->where(function (Builder $q) use ($searchVal) {
            //        $q->orWhere(DB::raw('lower(ms_spesifikasi_pakaian_dinas.nama)'), "like", "%{$searchVal}%")
            //            ->orWhere(DB::raw('lower(ms_jenis_pakaian_dinas.nama)'), "like", "%{$searchVal}%");
            //    });
            //}
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
        $query->orderByDesc('b.nama')->orderByDesc('a.nama')->orderByDesc('created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }

    static function getSelections()
    {
        return DB::table('ms_spesifikasi_pakaian_dinas as a')
            ->join('ms_jenis_pakaian_dinas as b', "a.ms_jenis_pakaian_dinas_id", '=', 'b.id')
            ->selectRaw("b.nama || ' - ' || a.nama as nama, a.id ")->orderBy('b.nama')->get();

    }

    static function getComplete($ids = [])
    {
        $query = DB::table('ms_spesifikasi_pakaian_dinas as a')->join('ms_jenis_pakaian_dinas as b', "a.ms_jenis_pakaian_dinas_id", '=', 'b.id');
        $query->select(["a.*", 'b.nama as jenis'])->whereIn('a.id', $ids);
        return $query->get();
    }

}
