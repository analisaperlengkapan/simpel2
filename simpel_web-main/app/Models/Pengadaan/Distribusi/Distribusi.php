<?php

namespace App\Models\Pengadaan\Distribusi;

use App\Blameable;
use App\Traits\LogTrait;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class Distribusi extends Model
{
    use HasFactory;
    use Blameable;
    use LogTrait;

    const tableKet = 'Pengadaan, Penyimpanan dan Dsitribusi Perlengkapan';

    protected $table = 'penyimpanan_distribusi';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'no_kontrak',
        'tgl_kontrak',
        'nm_barang',
        'jml_barang',
        'nilai_barang',
        'kdsatker_tujuan',
        'file_spk',
        'file_bast',
        'file_foto',
        'id_status',
        'is_gudang',
        'id_kontrak',
        'nilai_kontrak',
    ];

    function getDataGrid($paging, $search = [], $status = "")
    {
        $query = DB::table($this->table.' as a');
        $query->leftJoin('ms_satker as b', 'a.kdsatker_tujuan', '=', 'b.kdsatker_keu');
        $query->leftJoin('ms_status_penyimpanan_distribusi as c', 'a.id_status', '=', 'c.id');
        $query->select('a.*', 'b.inst_nama', 'c.status');
        if($status == 'masuk-gudang'){
            $query->whereIn('id_status',[2,3,4]);
        }else if($status == 'keluar-gudang'){
            $query->whereIn('id_status',[5,6,7]);
        }else if($status == 'konfirmasi-penerimaan'){
            $query->whereIn('id_status',[8,9]);
        }
        if (!empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach($searchVal as $k => $v){
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    if (($columnName == 'nilai_barang' || $columnName == 'jml_barang') && $value) {
                        $q->where($columnName, '=', $value);
                    }else if ($columnName == 'tgl_kontrak' && $value) {
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

    function getDataGridAdd($paging, $search = [], $status = "")
    {
        $query = DB::table($this->table.' as a');
        $query->leftJoin('ms_satker as b', 'a.kdsatker_tujuan', '=', 'b.kdsatker_keu');
        $query->leftJoin('ms_status_penyimpanan_distribusi as c', 'a.id_status', '=', 'c.id');
        $query->select('a.*', 'b.inst_nama', 'c.status');
        if($status == 'masuk-gudang'){
            $query->where('id_status','=',1);
            $query->where('is_gudang','=',1);
        }else if($status == 'keluar-gudang'){
            $query->where('id_status','=',3);
        }else if($status == 'konfirmasi-penerimaan'){
            $query->where('id_status','=',6);
            $query->orWhere(function (Builder $query) {
                $query->where('id_status', 1)
                      ->where('is_gudang', 2);
            });
        }
        if (!empty($search)) {
            $searchVal = strtolower($search['search']['value']);
            if (isset($search['filterBy'])) {
                $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            } else {
                $query->where(function (Builder $q) use ($searchVal) {
                    $q->orWhere(DB::raw('lower(a.kdsatker_tujuan)'), "like", "%{$searchVal}%")
                        ->orWhere(DB::raw("lower(b.inst_nama)"), 'like', "%{$searchVal}%")
                        ->orWhere(DB::raw("lower(a.no_kontrak)"), 'like', "%{$searchVal}%")
                        ->orWhere(DB::raw("lower(a.nm_barang)"), 'like', "%{$searchVal}%");
                });
            }
        }
        $query->orderByDesc('created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }

    static function findOne($id){
        $query = DB::table('penyimpanan_distribusi as a');
        $query->leftJoin('ms_satker as b', 'a.kdsatker_tujuan', '=', 'b.kdsatker_keu');
        $query->select('a.*', 'b.inst_nama');
        $query->where('a.id', '=', $id);
        return $query->first();
    }

    function getDataHist($id_penyimpanan = '')
    {
        $query = DB::table('penyimpanan_distribusi_hist as a');
        $query->leftJoin('ms_status_penyimpanan_distribusi as c', 'a.id_status', '=', 'c.id');
        $query->select('a.*', 'c.status');
        $query->where('id_penyimpanan','=',$id_penyimpanan);
        $query->orderByDesc('created_at');
        $total = $query->count();
        $data = $query->get();
        return ['total' => $total, 'data' => $data];
    }
}
