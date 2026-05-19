<?php

namespace App\Models\AnalisisKebutuhan;

use App\Blameable;
use App\Traits\LogTrait;
use Carbon\Carbon;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Str;

class Bmn extends Model
{
    use Blameable, HasFactory, LogTrait;

    protected $table = 'pengajuan_kebutuhan_bmn';

    const tableKet = 'Pengajuan Kebutuhan BMN';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'created_at',
        'updated_at',
        'created_by',
        'updated_by',
        'id_jenis_asset',
        'nama',
        'deskripsi',
        'ms_aktifitas_id',
        'pilihan_satker',
        'tahun',
        'tgl_mulai',
        'tgl_selesai',
        'is_appv_daskrimti',
    ];

    /**
     * The attributes that should be hidden for serialization.
     *
     * @var array<int, string>
     */

    /**
     * The attributes that should be cast.
     *
     * @var array<string, string>
     */
    // protected $casts = [
    //     'email_verified_at' => 'datetime',
    //     'password' => 'hashed',
    // ];

    public function getGridData($paging, $search = [], $filter = [])
    {
        $query = DB::table($this->table)->orderBy('pengajuan_kebutuhan_bmn.created_at', 'desc');
        $query->select('pengajuan_kebutuhan_bmn.*');
        if (! empty($filter)) {
            $query->where($filter);
        }
        $roleSatker = session('userData.current_role.ms_satker_id');
        if ($roleSatker != '00') {
            $query->leftJoin('pengajuan_kebutuhan_bmn_satker', 'pengajuan_kebutuhan_bmn_satker.pengajuan_kebutuhan_bmn_id', '=', 'pengajuan_kebutuhan_bmn.id');
            $query->where('pengajuan_kebutuhan_bmn_satker.ms_satker_id', 'like', "{$roleSatker}%");
        }

        if (! empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    if ($value) {
                        $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
                    }
                }
            });
        }
        $query->orderByDesc('pengajuan_kebutuhan_bmn.created_at');
        $query->groupBy('pengajuan_kebutuhan_bmn.id');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }

    public function getGridDataSatker($paging, $search = [])
    {
        $id = $search['id'] ?? '';
        $query = DB::table('ms_satker')
            ->select(['pengajuan_kebutuhan_bmn_satker.prioritas', DB::raw('SUM(pengajuan_kebutuhan_bmn_satker_barang.skor) as total_skor'), 'pengajuan_kebutuhan_bmn.tahun', 'pengajuan_kebutuhan_bmn.nama', 'pengajuan_kebutuhan_bmn.deskripsi', 'ms_satker.inst_nama as satker', DB::raw("coalesce(ms_aktifitas_user.nama, 'Belum Input') as status"), 'pengajuan_kebutuhan_bmn_satker.ms_aktifitas_id', 'pengajuan_kebutuhan_bmn_satker.id', 'ms_satker.inst_satkerkd', 'pengajuan_kebutuhan_bmn_id'])
            ->join('pengajuan_kebutuhan_bmn_satker', 'pengajuan_kebutuhan_bmn_satker.ms_satker_id', '=', 'ms_satker.inst_satkerkd')
            ->leftJoin('ms_aktifitas_user', 'ms_aktifitas_user.id', '=', 'pengajuan_kebutuhan_bmn_satker.ms_aktifitas_id')
            ->leftJoin('pengajuan_kebutuhan_bmn', 'pengajuan_kebutuhan_bmn.id', '=', 'pengajuan_kebutuhan_bmn_satker.pengajuan_kebutuhan_bmn_id')
            ->leftJoin('pengajuan_kebutuhan_bmn_satker_barang', 'pengajuan_kebutuhan_bmn_satker_barang.pengajuan_kebutuhan_bmn_satker_id', '=', 'pengajuan_kebutuhan_bmn_satker.id')
            ->groupBy('pengajuan_kebutuhan_bmn.created_at', 'ms_satker.inst_satkerkd', 'pengajuan_kebutuhan_bmn_satker.id', 'pengajuan_kebutuhan_bmn.tahun', 'pengajuan_kebutuhan_bmn.nama', 'pengajuan_kebutuhan_bmn.deskripsi', 'ms_satker.inst_nama', 'ms_aktifitas_user.nama', 'pengajuan_kebutuhan_bmn_satker.ms_aktifitas_id', 'pengajuan_kebutuhan_bmn_id')
            ->orderBy('pengajuan_kebutuhan_bmn.tahun', 'desc')
            ->orderBy('pengajuan_kebutuhan_bmn.created_at', 'desc')
            ->orderBy('pengajuan_kebutuhan_bmn_satker.prioritas', 'asc')
            ->orderByRaw('SUM(pengajuan_kebutuhan_bmn_satker_barang.skor) DESC')
            ->orderBy('ms_satker.inst_satkerkd');
        if ($id != '') {
            $query->where('pengajuan_kebutuhan_bmn.id', '=', $search['id']);
        }
        $roleSatker = session('userData.current_role.ms_satker_id');
        $msRole = session('userData.current_role.ms_role_id');

        if ($roleSatker == '00' && $msRole == config('constants.pelaksana_satker_role_id')) {
            $query->where('ms_satker.inst_satkerkd', '=', session('userData.current_role.ms_satker_pusat_id'));
        } elseif ($msRole == config('constants.validator_wilayah_role_id')) {
            $query->where('ms_satker.inst_satkerkd', 'like', "{$roleSatker}%");
        } elseif ($msRole == config('constants.pelaksana_satker_role_id')) {
            $query->where('ms_satker.inst_satkerkd', '=', $roleSatker);
        }

        if (! empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    if ($columnName == 'satker') {
                        $columnName = 'ms_satker.inst_nama';
                    }
                    if ($columnName == 'nama') {
                        $columnName = 'pengajuan_kebutuhan_bmn.nama';
                    }
                    if ($columnName == 'status') {
                        $columnName = 'ms_aktifitas_user.nama';
                    }
                    if (Str::contains(strtolower($value), 'input') || Str::contains(strtolower($value), 'belum')) {
                        $q->whereNull($columnName);
                        $value = null;
                    }
                    if ($value) {
                        $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
                    }
                }
            });
        }
        $total = count($query->get());
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }

    public function getGridDataSatkerDetail($paging, $search = [])
    {
        $id = $search['id'] ?? '';
        $query = DB::table('ms_satker')
            ->select([
                'pengajuan_kebutuhan_bmn_satker_barang.id',
                'pengajuan_kebutuhan_bmn.tahun',
                'pengajuan_kebutuhan_bmn.nama',
                'ms_satker.inst_nama as satker',
                'pengajuan_kebutuhan_bmn_satker_barang.nama as nm_barang',
                'pengajuan_kebutuhan_bmn_satker_barang.kode_barang',
                DB::raw('COUNT(d.kode_barang) as jumlah_exist'),
                'pengajuan_kebutuhan_bmn_satker_barang.jumlah',
                'pengajuan_kebutuhan_bmn_satker_barang.jml_setuju',
                DB::raw('(pengajuan_kebutuhan_bmn_satker_barang.jumlah - pengajuan_kebutuhan_bmn_satker_barang.jml_setuju) as jml_tolak'),
                'pengajuan_kebutuhan_bmn_satker_barang.keterangan',
                'pengajuan_kebutuhan_bmn_satker_barang.alasan',
                'pengajuan_kebutuhan_bmn_satker_barang.prioritas',
                'pengajuan_kebutuhan_bmn_satker.id as detail_satker_id',
            ])
            ->join('pengajuan_kebutuhan_bmn_satker', 'pengajuan_kebutuhan_bmn_satker.ms_satker_id', '=', 'ms_satker.inst_satkerkd')
            ->where('ms_satker.inst_satkerkd', '!=', '00')
            ->leftJoin('ms_aktifitas_user', 'ms_aktifitas_user.id', '=', 'pengajuan_kebutuhan_bmn_satker.ms_aktifitas_id')
            ->leftJoin('pengajuan_kebutuhan_bmn', 'pengajuan_kebutuhan_bmn.id', '=', 'pengajuan_kebutuhan_bmn_satker.pengajuan_kebutuhan_bmn_id')
            ->join('pengajuan_kebutuhan_bmn_satker_barang', 'pengajuan_kebutuhan_bmn_satker_barang.pengajuan_kebutuhan_bmn_satker_id', '=', 'pengajuan_kebutuhan_bmn_satker.id')
            ->leftJoin('vw_asset_barang_kdsatker as d', function ($join) {
                $join->on('d.kdsatker_keu', '=', 'ms_satker.kdsatker_keu')
                    ->on('d.kode_barang', '=', 'pengajuan_kebutuhan_bmn_satker_barang.kode_barang');
            })
            ->groupBY('pengajuan_kebutuhan_bmn_satker_barang.id',
                'pengajuan_kebutuhan_bmn.tahun',
                'pengajuan_kebutuhan_bmn.nama',
                'pengajuan_kebutuhan_bmn_satker.id',
                'ms_satker.inst_nama')
            ->orderBy('pengajuan_kebutuhan_bmn_satker_barang.prioritas', 'asc');
        if ($id != '') {
            $query->where('pengajuan_kebutuhan_bmn.id', '=', $search['id']);
        }
        // $roleSatker = session('userData.current_role.ms_satker_id');
        // if ($roleSatker != '00') {
        //     $query->where('ms_satker.inst_satkerkd', 'like', "{$roleSatker}%");
        // }

        if (! empty($search['columns'])) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    if ($value) {
                        $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
                    }
                }
            });
        }
        $total = count($query->get());
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }

    public function gridDataSatkerAset($paging, $table_asset, $id)
    {
        $query = null;

        foreach ($table_asset as $table) {
            if ($query === null) {
                $query = DB::table('pengajuan_kebutuhan_bmn_satker as a')
                    ->select('d.kode_barang', 'd.nm_barang', 'd.nup', 'd.tgl_perolehan', 'd.nilai_perolehan', 'd.kondisi')
                    ->leftJoin('pengajuan_kebutuhan_bmn as b', 'a.pengajuan_kebutuhan_bmn_id', '=', 'b.id')
                    ->leftJoin('ms_satker as c', 'a.ms_satker_id', '=', 'c.inst_satkerkd')
                    ->leftJoin($table->nm_table.' as d', 'c.kdsatker_keu', '=', 'd.id_satker')
                    ->where('d.kode_barang', $table->kode_barang)
                    ->where('a.id', $id);
            } else {
                $query->unionAll(DB::table('pengajuan_kebutuhan_bmn_satker as a')
                    ->select('d.kode_barang', 'd.nm_barang', 'd.nup', 'd.tgl_perolehan', 'd.nilai_perolehan', 'd.kondisi')
                    ->leftJoin('pengajuan_kebutuhan_bmn as b', 'a.pengajuan_kebutuhan_bmn_id', '=', 'b.id')
                    ->leftJoin('ms_satker as c', 'a.ms_satker_id', '=', 'c.inst_satkerkd')
                    ->leftJoin($table->nm_table.' as d', 'c.kdsatker_keu', '=', 'd.id_satker')
                    ->where('d.kode_barang', $table->kode_barang)
                    ->where('a.id', $id));
            }
        }
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }

    public function getDataCetak($id)
    {
        return DB::table('pengajuan_kebutuhan_bmn_satker as a')
            ->select('a.*', 'b.*', 'c.inst_nama')
            ->leftJoin('pengajuan_kebutuhan_bmn_satker_barang as b', 'a.id', '=', 'b.pengajuan_kebutuhan_bmn_satker_id')
            ->leftJoin('ms_satker as c', 'a.ms_satker_id', '=', 'c.inst_satkerkd')
            ->where('a.pengajuan_kebutuhan_bmn_id', $id)
            ->orderBy('b.id')
            ->get();
    }

    public function getTglMulaiAttribute()
    {
        return Carbon::parse($this->attributes['tgl_mulai'])->translatedFormat('d-F-Y');
    }

    public function getTglSelesaiAttribute()
    {
        return Carbon::parse($this->attributes['tgl_selesai'])->translatedFormat('d-F-Y');
    }

    public function getGridDataRusak($paging, $search = [])
    {
        $query = DB::table('vw_asset_rusak_berat');
        if (! empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    if ($value) {
                        $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
                    }
                }
            });
        }
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }
}
