<?php

namespace App\Models\Pengadaan\PokjaPemilihan;

use App\Blameable;
use App\Traits\LogTrait;
use Carbon\Carbon;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class PokjaPemilihan extends Model
{
    use HasFactory;
    use Blameable;
    use LogTrait;

    protected $table = 'pengajuan_pokja_pemilihan';

    const tableKet = 'Pengajuan Pokja Pemilihan';

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
        'tgl_pengajuan'
    ];

    function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table.' as a');
        $query->leftJoin('ms_satker as b', 'a.inst_satkerkd', '=', 'b.inst_satkerkd');
        $query->leftJoin('ms_aktifitas_user as c', 'a.ms_aktifitas_id', '=', 'c.id');
        $query->select('a.*', 'b.inst_nama', DB::raw("CASE WHEN a.ms_aktifitas_id = 1008 THEN 'Terbit SK Penetapan Pokja Pemilihan' ELSE  c.nama END as aktifitas"));

        $ms_satker_id = session('userData.current_role.ms_satker_id');
        if(in_array(session('userData.current_role.ms_role_id'), config('constants.pelaksana_role'))){
            $query->where('a.inst_satkerkd', $ms_satker_id);
        }
        if(session('userData.current_role.ms_role_id') == config('constants.validator_pusat_role_id') ||
            session('userData.current_role.ms_role_id') == config('constants.validator_wilayah_role_id')){
            $query->where('a.ms_aktifitas_id', '!=', 1000);
        }
        if (!empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach($searchVal as $k => $v){
                    $value = $v['search']['value'];
                    $columnName = $v['data']=='aktifitas'?'c.nama':$v['data'];
                    if (strtotime($value)) {
                        $q->whereDate($columnName, '=', date('Y-m-d', strtotime($value)));
                    }else if($value){
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

    public function getTglPengajuanAttribute()
    {
        return Carbon::parse($this->attributes['tgl_pengajuan'])->translatedFormat('d-F-Y');
    }
}
