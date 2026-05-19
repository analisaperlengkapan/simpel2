<?php

namespace App\Models\Asset;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class Senjata extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'asset_alat_persenjataan';

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
        'tgl_psp',
        'jml_kib',
    ];

    public function getDataGrid($paging, $search = [], $select = [])
    {
        $query = DB::table($this->table.' as a');
        if (! empty($select)) {
            $query->select($select);
        } else {
            $query->select('a.*');
        }
        $roleSatker = session('userData.current_role.ms_satker_id');
        if ($roleSatker != '00') {
            $query->leftJoin('ms_satker as b', 'a.id_satker', '=', 'b.kdsatker_keu');
            $query->where('b.inst_satkerkd', 'like', "{$roleSatker}%");
        }
        if (! empty($search)) {
            $searchVal = $search['columns'];
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
        $query->orderByDesc('tgl_perolehan');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }

    public function getDataExport($search = [], $defColumn = null)
    {
        $select = [DB::raw("CONCAT(b.kode_unit,'.',b.kdsatker,'.',RIGHT(b.kode_uappbw,6)) as kdsatker_keu"), 'b.deskripsi as inst_nama', 'a.kode_barang', 'a.nm_barang', 'a.nup', 'a.kondisi', 'a.merk', 'a.tgl_rekam_pertama', 'a.tgl_perolehan',
            'a.nilai_perolehan_pertama', 'a.nilai_mutasi', 'a.nilai_perolehan', 'a.nilai_penyusutan', 'a.nilai_buku', 'a.kuantitas', 'a.jml_foto', 'a.status_penggunaan',
            'a.status_pengelolaan', 'a.no_psp', 'a.tgl_psp', 'a.jml_kib'];
        if ($defColumn) {
            $select = array_intersect_key($select, array_flip($defColumn));
        }
        $query = DB::table($this->table.' as a');
        $query->leftJoin('ms_satker_sakti as b', 'a.kdsatker_keu', '=', 'b.kdsatker');
        $query->select($select);
        if (! empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    if ($columnName == 'inst_nama') {
                        $kolom = 'b.deskripsi';
                    } elseif ($columnName == 'kdsatker') {
                        $kolom = "(b.kode_unit||'.'||b.kdsatker||'.'||RIGHT(b.kode_uappbw,6))";
                    } else {
                        $kolom = 'a.'.$columnName;
                    }
                    $tableName = $columnName == 'kdsatker' || $columnName == 'inst_nama' ? 'ms_satker_sakti' : $this->table;
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
        $query->orderByDesc('a.created_at');
        $data = $query->get();

        return ['data' => $data];
    }

    public static function findOne($id)
    {
        $query = DB::table('asset_alat_persenjataan as a');
        $query->leftJoin('ms_satker as b', 'a.kdsatker_keu', '=', 'b.kdsatker_keu');
        $query->select('a.*', 'b.inst_nama');
        $query->where('a.id', '=', $id);

        return $query->first();
    }
}
