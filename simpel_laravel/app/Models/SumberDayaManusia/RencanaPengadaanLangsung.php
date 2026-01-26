<?php

namespace App\Models\SumberDayaManusia;

use App\Blameable;
use Carbon\Carbon;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class RencanaPengadaanLangsung extends Model
{
    use HasFactory;
    use Blameable;

    protected $table = 'pengadaan_rencana_langsung';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'inst_satkerkd',
        'kdsatker_keu',
        'jenis_kontrak',
        'no_kontrak',
        'nilai_kontrak',
        'tgl_kontrak',
        'file_kontrak',
        'jenis_pengadaan',
        'konsep_hps',
        'surat_keputusan_penyedia',
        'no_spk',
        'tgl_spk',
        'jangka_waktu_pelaksanaan',
        'bast',
        'ba_pembayaran',
        'nodis_pengantar_kuitansi',
    ];

    function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table.' as a');
        $query->leftJoin('ms_satker as b', 'a.kdsatker_keu', '=', 'b.kdsatker_keu');
        $query->select('a.*', 'b.inst_nama');
        $ms_satker_id = session('userData.current_role.ms_satker_id');
        if(in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'))){
            $query->where('a.inst_satkerkd', $ms_satker_id);
        }
        if (!empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach($searchVal as $k => $v){
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    if ($columnName == 'nilai_kontrak' && $value) {
                        $q->where($columnName, '=', $value);
                    }else if ($columnName == 'tgl_kontrak' && $value) {
                        $q->whereDate($columnName, '=', date('Y-m-d', strtotime($value)));
                    }else if($value){
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

    static function findOne($id){
        $query = DB::table('pengadaan_rencana_langsung as a');
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
