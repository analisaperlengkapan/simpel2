<?php

namespace App\Models\Bmn\PermohonanPenghapusan;

use App\Blameable;
use App\Helpers\MyHelper;
use App\Traits\LogTrait;
use Carbon\Carbon;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class PermohonanPenghapusanBmn extends Model
{
    // use HasFactory;
    // use Blameable;
    use LogTrait;

    protected $table = 'permohonan_penghapusan_bmn';

    const tableKet = 'Pengajuan Permohonan Penghapusan BMN';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'jenis_barang',
        'no_surat_permohonan',
        'tgl_surat_permohonan',
        'file_surat_permohonan',
        'inst_satkerkd',
        'ms_aktifitas_id',
        'created_at',
        'created_by',
        'updated_at',
        'updated_by',
        'kategori',
    ];

    function getDataGrid($paging, $search = [], $kategori = [])
    {
        $query = DB::table($this->table . ' as a');
        $query->leftJoin('ms_satker as b', 'a.inst_satkerkd', '=', 'b.inst_satkerkd');
        $query->leftJoin('ms_aktifitas_user as c', 'a.ms_aktifitas_id', '=', 'c.id');
        $query->leftJoin('permohonan_penghapusan_bmn_sk as d', 'a.id', '=', 'd.pengajuan_id');
        $query->leftJoin('permohonan_penghapusan_bmn_monitor as e', 'a.id', '=', 'e.pengajuan_id');
        $query->select(
            'a.*',
            'b.inst_nama',
            DB::raw("CASE WHEN e.pengajuan_id is not null then 'Selesai'
        WHEN d.ms_aktifitas_id = 3006 THEN 'Surat Keputusan Terbit'
        WHEN a.ms_aktifitas_id = 3006 THEN 'Surat Persetujuan Terbit'
        ELSE c.nama END as aktifitas")
        );

        $ms_satker_id = session('userData.current_role.ms_satker_id');

        if (MyHelper::isPelaksanaSatker()) {
            $query->where('a.inst_satkerkd', $ms_satker_id);
        }
        if (MyHelper::isValidatorWilayah()) {
            $query->where('a.inst_satkerkd',  'like', "{$ms_satker_id}%");
        }
        if (session('userData.current_role.ms_role_id') != 1) {
            $query->where('a.ms_aktifitas_id', '!=', 3000);
        }
        if (!empty($kategori)) $query->whereIn('a.kategori', $kategori);

        if (!empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    if($columnName == 'aktifitas') $columnName = 'c.nama';
                    if($columnName == 'nama') $columnName = 'a.nama';
                    if($columnName == 'no_surat_permohonan') $columnName = 'a.no_surat_permohonan';
                    if($columnName == 'tgl_surat_permohonan') $columnName = 'a.tgl_surat_permohonan';
                    if($columnName == 'kategori') $columnName = 'a.kategori';

                    if ($columnName == 'a.tgl_surat_permohonan' && strtotime($value)) {
                        $q->whereDate($columnName, '=', date('Y-m-d', strtotime($value)));
                    } else if ($value) {
                        $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
                    }
                }
            });
        }
        $query->orderByDesc('updated_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }

    function getDataGridDetail($paging, $search = [])
    {
        $query = DB::table($this->table . ' as a');
        $query->leftJoin('ms_satker as b', 'a.inst_satkerkd', '=', 'b.inst_satkerkd');
        $query->leftJoin('ms_aktifitas_user as c', 'a.ms_aktifitas_id', '=', 'c.id');
        $query->leftJoin('pengajuan_penghapusan_bmn_asset as d', 'a.id', '=', 'd.pengajuan_id');
        $query->select('a.*', 'b.inst_nama', 'c.nama as aktifitas', 'd.kode_barang', 'd.nm_barang');

        $ms_satker_id = session('userData.current_role.ms_satker_id');
        if (in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'))) {
            $query->where('a.inst_satkerkd', $ms_satker_id);
        }

        if (!empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'] == 'aktifitas' ? 'c.nama' : $v['data'];
                    $columnName = $v['data'] == 'nama' ? 'a.nama' : $v['data'];
                    if (strtotime($value)) {
                        $q->whereDate($columnName, '=', date('Y-m-d', strtotime($value)));
                    } else if ($value) {
                        $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
                    }
                }
            });
        }
        $query->orderByDesc('updated_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }

    public function getTglSuratPermohonanAttribute()
    {
        return Carbon::parse($this->attributes['tgl_surat_permohonan'])->translatedFormat('d-F-Y');
    }
}
