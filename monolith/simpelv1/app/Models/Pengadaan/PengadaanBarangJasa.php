<?php

namespace App\Models\Pengadaan;

use App\Blameable;
use App\Traits\LogTrait;
use Carbon\Carbon;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class PengadaanBarangJasa extends Model
{
    use Blameable;
    use HasFactory;
    use LogTrait;

    const tableKet = 'Data Pengadaan Barang dan Jasa';

    protected $table = 'pengadaan_barang_jasa';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'inst_satkerkd',
        'kdsatker_keu',
        'jenis_pengadaan',
        'nama_pengadaan',
        'kode_barang',
        'nilai',
        'kode_anggaran',
    ];

    public function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table.' as a');
        $query->leftJoin('ms_satker as b', 'a.kdsatker_keu', '=', 'b.kdsatker_keu');
        $query->select('a.*', 'b.inst_nama');
        $ms_satker_id = session('userData.current_role.ms_satker_id');
        if (in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'))) {
            $query->where('a.inst_satkerkd', $ms_satker_id);
        }
        if (! empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    if ($columnName == 'nilai_kontrak' && $value) {
                        $q->where($columnName, '=', $value);
                    } elseif ($columnName == 'tgl_kontrak' && $value) {
                        $q->whereDate($columnName, '=', date('Y-m-d', strtotime($value)));
                    } elseif ($value) {
                        $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
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
        $query = DB::table('pengadaan_barang_jasa as a');
        $query->leftJoin('ms_satker as b', 'a.kdsatker_keu', '=', 'b.kdsatker_keu');
        $query->select('a.*', 'b.inst_nama');
        $query->where('a.id', '=', $id);

        return $query->first();
    }

    public function getTglKontrakAttribute()
    {
        return Carbon::parse($this->attributes['tgl_kontrak'])->translatedFormat('d-F-Y');
    }

    public function getTglSpkAttribute()
    {
        return Carbon::parse($this->attributes['tgl_spk'])->translatedFormat('d-F-Y');
    }
}
