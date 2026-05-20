<?php

namespace App\Models\Bmn\Wasdal;

use App\Blameable;
use App\Traits\LogTrait;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class Hapusppbmn extends Model
{
    use Blameable;
    use HasFactory;
    use LogTrait;

    protected $table = 'siman.siman_wasdal_sk_hapus_pp_kl';

    const tableKet = 'WASDAL SIMAN Hapus PP BMN';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'kode_kl',
        'kpknl',
        'tahun_anggaran',
        'kode_satker',
        'nama_satker',
        'kode_barang',
        'nup',
        'noreg',
        'nama_barang',
        'dasar_penertiban',
        'no_laporan',
        'tgl_laporan',
        'bentuk_penertiban',
        'ur_bentuk_penertiban',
        'no_surat_penertiban',
        'tgl_surat_penertiban',
        'uraian_penertiban',
        'tindak_lanjut',
        'tgl_tarik',
    ];

    protected $casts = [
        'id' => 'string',
    ];

    public function getDataGrid($paging, $search = [])
    {
        DB::enableQueryLog();

        $query = DB::table($this->table.' as a');
        $query->select('a.*');
        // $query->leftJoin('ms_satker as b','a.kdsatker','=','b.kdsatker_keu');
        $roleSatker = session('userData.current_role.ms_satker_id');
        if ($roleSatker != '00') {
            $query->where('a.id_satker_keu', 'like', "{$roleSatker}%");
        }
        if (! empty($search)) {
            $searchVal = $search['columns'];

            // echo "<pre>"; print_r($searchVal);exit;

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

                            if ($columnName == 'nilai_kontrak' && $value) {
                                if ($value == 'duaratus') {
                                    $q->where(DB::raw('cast(a.nilai_kontrak as numeric)'), '<=', 200000000);
                                } elseif ($value == 'duaratus_lebih') {
                                    $q->where(DB::raw('cast(a.nilai_kontrak as numeric)'), '>', 200000000);
                                }
                            } elseif ($columnName == 'inst_nama' && $value) {
                                if ($value) {
                                    $q->where('a.kdsatker', '=', $value);
                                }
                            } elseif ($value) {
                                $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
                            }
                        }
                    }
                }
            });
        }
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        // $querys = DB::getQueryLog();
        // dd($querys);exit;

        return ['total' => $total, 'data' => $data];
    }

    // function getDataDetail($id_sk){
    //     $query = DB::table('siman.siman_wasdal_sk_guna_sementara_bmn_kl as a');
    //     $query->where('a.id_sk', $id_sk);
    //     return $query->get();
    // }

}
