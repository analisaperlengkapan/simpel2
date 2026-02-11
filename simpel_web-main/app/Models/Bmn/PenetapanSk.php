<?php

namespace App\Models\Bmn;

use App\Helpers\MyHelper;
use App\Traits\HooksTrait;
use App\Traits\LogTrait;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class PenetapanSk extends Model
{
    use HooksTrait, LogTrait;

    protected $table = 'bmn_penetapan';
    const tableKet = 'Pengajuan SK Penetapan Status Penggunaan';
    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'ms_satker_id',
        'ms_aktifitas_id',
        'id_satker_keu',
        'keterangan',
        'nm_satker',
        'sp_no',
        'sp_tgl',
        'sp_file',
        'sk_tgl',
        'sk_no',
        'sk_file',
        'sk_jenis',
    ];

    function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table . ' as a');
        $query->leftJoin('ms_satker as b', 'a.ms_satker_id', '=', 'b.inst_satkerkd');
        $query->leftJoin('ms_aktifitas as c', 'a.ms_aktifitas_id', '=', 'c.id');
        $query->select(['a.*', 'b.inst_nama', 'c.nama as aktifitas_nama', 'c.id as aktifitas_id']);
        $currentRole = session('userData.current_role');
        if (MyHelper::isPelaksanaSatker()) {
            $query->where('a.ms_satker_id', $currentRole['ms_satker_id']);
        }
        if (MyHelper::isValidatorWilayah()) {
            $query->where('a.ms_satker_id',  'like', "{$currentRole['ms_satker_id']}%");
        }

        if (!empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    $tableName = $this->table;
                    if ($columnName == 'inst_nama') {
                        $kolom = 'b.inst_nama';
                        $tableName = 'ms_satker';
                    } else if ($columnName == 'aktifitas_nama') {
                        $kolom = "c.nama";
                        $tableName = 'ms_aktifitas';
                    } else {
                        $kolom = 'a.' . $columnName;
                    }
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
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }

    static function findOne($id)
    {
        $query = DB::table('asset_tik_hakcipta as a');
        $query->leftJoin('ms_satker as b', 'a.kdsatker_keu', '=', 'b.kdsatker_keu');
        $query->select('a.*', 'b.inst_nama');
        $query->where('a.id', '=', $id);
        return $query->first();
    }

    function getDataGridMonitoring($paging, $search)
    {
        $currentRole = session('userData.current_role');
        $query = DB::table('vw_aset_psp as a')
            ->leftJoin('ms_jenis_asset as b', 'a.ms_jenis_asset_id', '=', 'b.id')
            ->leftJoin('ms_satker as e', 'e.kdsatker_keu', '=', 'a.kdsatker_keu')
            ->select([
                'a.*',
                'b.name as nm_aset',
                'e.inst_nama'
            ]);
        $query->where('a.kdsatker_keu', $currentRole['ms_satker_id_keu'])
            ->orderBy('b.name')->orderBy('a.nm_barang');
        // if ($currentRole['ms_role_id'] == config('constants.pelaksana_satker_role_id')) {
        // }

        if (!empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    $tableName = 'vw_aset_psp';
                    if ($columnName == 'nama_asset') {
                        $kolom = 'b.name';
                        $tableName = 'ms_jenis_asset';
                    } else if ($columnName == 'inst_nama') {
                        $kolom = "e.inst_nama";
                        $tableName = 'ms_satker';
                    } else {
                        $kolom = 'a.' . $columnName;
                    }
                    if ($value) {
                        $dataType = DB::table('information_schema.columns')->select('data_type')->where('table_name', $tableName)->where('column_name', $columnName)->value('data_type');
                        if (in_array($dataType, ['integer', 'numeric', 'smallint', 'bigint'])) {
                            $q->where($kolom, '=', $value);
                        } elseif (in_array($dataType, ['timestamp without time zone', 'timestamp', 'date'])) {
                            if (strtotime($value)) {
                                $q->whereDate($kolom, '=', date('Y-m-d', strtotime($value)));
                            }
                        } else {
                            if ($columnName == 'no_siman') {
                                if (strtolower($value) == 'belum') {
                                    $q->whereNull($columnName);
                                } elseif (strtolower($value) == 'sudah') {
                                    $q->whereNotNull($columnName);
                                }
                            } else {
                                $q->where(DB::raw("lower({$kolom})"), 'like', strtolower("%{$value}%"));
                            }
                        }
                    }
                }
            });
        }
        // $query->orderByDesc('a.created_at');
        $total = $query->distinct()->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }

    static function asetSudahDiPsp()
    {
        $udahDiajukanIds = DB::table('bmn_penetapan_asset as a')
            ->select(['a.vw_aset_psp_id'])
            ->join('bmn_penetapan as b', 'a.bmn_penetapan_id', '=', 'b.id')
            ->whereNotIn('b.ms_aktifitas_id', ['2003'])->get()->pluck('vw_aset_psp_id')->toArray();
        return $udahDiajukanIds;
    }
    static function getAset($filter = [], $raw = true)
    {
        $pspStatus = $filter['pspStatus'] ?? null;
        $whereIn = $filter['whereIn'] ?? null;
        $whereNotIn = $filter['whereNotIn'] ?? null;

        $query = DB::table('vw_aset_psp as a')
            ->select(['a.*', 'b.name as nm_aset'])
            ->join('ms_jenis_asset as b', 'a.ms_jenis_asset_id', '=', 'b.id')
            ->where('kdsatker_keu', session('userData.current_role.ms_satker_id_keu'))
            ->orderByRaw('ms_jenis_asset_id, kode_barang, nup');

        switch ($pspStatus) {
            case 'DIAJUKAN':
                $query->whereIn('a.id', self::asetSudahDiPsp());
                break;
            case 'SUDAH':
                $query->whereNotNull('no_psp');
                break;
            case 'BELUM':
                $query->whereNull('no_psp');
                break;
        }

        if ($whereIn) {
            $query->whereIn('a.id', $whereIn);
        }
        if ($whereNotIn) {
            $query->whereNotIn('a.id', $whereNotIn);
        }


        if ($raw)
            return $query->get();

        return $query;
    }

    static function getPengajuanAset($id)
    {
        return DB::table('bmn_penetapan_asset as a')->select(['b.*', 'c.name as nm_aset'])
            ->join('vw_aset_psp as b', 'b.id', '=', 'a.vw_aset_psp_id')
            ->join('ms_jenis_asset as c', 'b.ms_jenis_asset_id', '=', 'c.id')
            ->where('a.bmn_penetapan_id', $id)
            ->get();
    }
    function getDataGridAset($paging, $search = [])
    {
        $query = self::getAset($search, false);
        if (!empty($search) && isset($search['columns'])) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    $tableName = $this->table;
                    $kolom = 'a.' . $columnName;
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
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }
}
