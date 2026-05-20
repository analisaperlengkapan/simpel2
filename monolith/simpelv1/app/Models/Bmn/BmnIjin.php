<?php

namespace App\Models\Bmn;

use App\Blameable;
use App\Traits\LogTrait;
use Carbon\Carbon;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class BmnIjin extends Model
{
    use Blameable;
    use HasFactory;
    use LogTrait;

    protected $table = 'bmn_ijin_pemakaian';

    const tableKet = 'Ijin Pemakaian BMN';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'nama',
        'deskripsi',
        'inst_satkerkd',
        'ms_aktifitas_id',
        'file_sk',
        'file_pbj',
        'tgl_pengajuan1',
        'tgl_pengajuan2',
        'kategori',
    ];

    public function getDataGrid($paging, $search = [], $kategori = '')
    {
        $query = DB::table($this->table.' as a');
        $query->leftJoin('ms_satker as b', 'a.inst_satkerkd', '=', 'b.inst_satkerkd');
        $query->leftJoin('ms_aktifitas_user as c', 'a.ms_aktifitas_id', '=', 'c.id');
        $query->select('a.*', 'b.inst_nama', 'c.nama as aktifitas');
        if ($kategori) {
            $query->where('a.kategori', $kategori);
        }
        $ms_satker_id = session('userData.current_role.ms_satker_id');
        if (in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'))) {
            $query->where('a.inst_satkerkd', $ms_satker_id);
        }
        if (! empty($search)) {
            $searchVal = strtolower($search['search']['value']);
            if (isset($search['filterBy'])) {
                $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            } else {
                $query->where(function (Builder $q) use ($searchVal) {
                    $q->orWhere(DB::raw('lower(c.nama)'), 'like', "%{$searchVal}%")
                        ->orWhere(DB::raw('lower(b.inst_nama)'), 'like', "%{$searchVal}%");
                });
            }
        }
        $query->orderByDesc('updated_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }

    public function getDataGridMonitor($paging, $search = [], $kategori = '')
    {
        $query = DB::table('bmn_ijin_pemakaian_satker_pegawai as a');
        $query->join('mv_curr_pegawai_all as b', 'a.nip', '=', 'b.peg_nip_baru');
        $query->leftJoin('bmn_ijin_pemakaian as c', 'a.pengajuan_id', '=', 'c.id');
        $query->select('a.nip', 'b.nama', 'a.pengajuan_id as id', 'b.satker as inst_nama');
        $query->distinct('a.nip');

        $ms_satker_id = session('userData.current_role.ms_satker_id');
        if (in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'))) {
            $query->where('c.inst_satkerkd', $ms_satker_id);
        }
        /* if($kategori){
            $query->where('a.kategori', $kategori);
        } */
        /* $ms_satker_id = session('userData.current_role.ms_satker_id');
        if(in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'))){
            $query->where('a.inst_satkerkd', $ms_satker_id);
        }
        if (!empty($search)) {
            $searchVal = strtolower($search['search']['value']);
            if (isset($search['filterBy'])) {
                $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            } else {
                $query->where(function (Builder $q) use ($searchVal) {
                    $q->orWhere(DB::raw('lower(c.nama)'), "like", "%{$searchVal}%")
                        ->orWhere(DB::raw("lower(b.inst_nama)"), 'like', "%{$searchVal}%");
                });
            }
        }
        $query->orderByDesc('updated_at');*/
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }

    public function getTglPengajuanAttribute()
    {
        return Carbon::parse($this->attributes['tgl_pengajuan1'])->translatedFormat('d-F-Y');

        return Carbon::parse($this->attributes['tgl_pengajuan2'])->translatedFormat('d-F-Y');
    }

    public function getDataCetak($id)
    {
        return DB::table('bmn_ijin_pemakaian as a')
            ->select('a.*', 'b.*', 'c.*', 'd.inst_nama', 'e.nama')
            ->leftJoin('bmn_ijin_pemakaian_satker_pegawai as b', 'a.id', '=', 'b.pengajuan_id')
            ->leftJoin('bmn_ijin_pemakaian_satker_pegawai_aset as c', 'b.nip', '=', 'c.nip')
            ->leftJoin('ms_satker as d', 'a.inst_satkerkd', '=', 'd.inst_satkerkd')
            ->leftJoin('mv_curr_pegawai_all as e', 'c.nip', '=', 'e.peg_nip_baru')
            ->where('b.id', $id)
            ->orderBy('b.pengajuan_id')
            ->get();
    }
}
