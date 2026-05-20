<?php

namespace App\Models\Asset;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class Tanah extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'siman.siman_aset_tanah_kl';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'id_satker_keu',
        'nm_satker',
        'id_barang',
        'kode_barang',
        'nm_barang',
        'nup',
        'kondisi',
        'jenis_dokumen',
        'kepemilikan',
        'jenis_sertifikat',
        'merk',
        'tgl_rekam_pertama',
        'tgl_perolehan',
        'nilai_perolehan_pertama',
        'nilai_mutasi',
        'nilai_perolehan',
        'nilai_penyusutan',
        'nilai_buku',
        'kuantitas',
        'luas_tanah_total',
        'luas_tanah_bangunan',
        'luas_tanah_sarana',
        'luas_lahan_kosong',
        'jml_foto',
        'status_penggunaan',
        'status_pengelolaan',
        'no_psp',
        'tgl_psp',
        'alamat',
        'rt_rw',
        'kelurahan',
        'kecamatan',
        'kabkota',
        'kode_kabkota',
        'provinsi',
        'kode_prov',
        'kode_pos',
        'jml_kib',
        'sbsk',
        'optimalisasi',
        'status_sbsn',
    ];

    public function getDataGrid($paging, $search = [], $select = [])
    {
        // DB::enableQueryLog();

        $query = DB::table($this->table.' as a');
        if (! empty($select)) {
            $query->select($select, 'b.inst_nama');
        } else {
            $query->select('a.*', 'b.inst_nama');
        }
        $query->leftJoin('ms_satker as b', 'a.id_satker_keu', '=', 'b.kdsatker_keu');

        $roleSatker = session('userData.current_role.ms_satker_id');
        $roleSadmin = session('userData.current_role.ms_role_id');

        // return $roleSatker;exit;

        if ($roleSatker != '00' && $roleSadmin != '3') {
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

        $query->orderByDesc('tanggal_perolehan');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        // dd(DB::getQueryLog());
        // echo "<pre>";
        // print_r(DB::getQueryLog()); exit;

        return ['total' => $total, 'data' => $data];
    }

    public function getDataExport($search = [], $defColumn = null)
    {
        $select = ['a.kdsatker_keu', 'a.nm_satker', 'a.kode_barang', 'a.nm_barang', 'a.nup', 'a.kondisi', 'a.jenis_dokumen', 'a.kepemilikan', 'a.jenis_sertifikat',
            'a.merk', 'a.tgl_rekam_pertama', 'a.tgl_perolehan', 'a.nilai_perolehan_pertama', 'a.nilai_mutasi', 'a.nilai_perolehan', 'a.nilai_penyusutan', 'a.nilai_buku', 'a.kuantitas',
            'a.luas_tanah_total', 'a.luas_tanah_bangunan', 'a.luas_tanah_sarana', 'a.luas_lahan_kosong', 'a.jml_foto', 'a.status_penggunaan', 'a.status_pengelolaan', 'a.no_psp',
            'a.tgl_psp', 'a.alamat', 'a.rt_rw', 'a.kelurahan', 'a.kecamatan', 'a.kabkota', 'a.kode_kabkota', 'a.provinsi', 'a.kode_prov', 'a.kode_pos', 'a.jml_kib', 'a.sbsk', 'a.optimalisasi', 'a.status_sbsn'];
        if ($defColumn) {
            $select = array_intersect_key($select, array_flip($defColumn));
        }
        $query = DB::table($this->table.' as a');
        $query->select($select);
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
        $query->orderByDesc('a.created_at');
        $data = $query->get();

        return ['data' => $data];
    }

    public static function findOne($id)
    {
        $query = DB::table('asset_tanah as a');
        $query->select('a.*');
        $query->where('a.id', '=', $id);

        return $query->first();
    }
}
