<?php

namespace App\Models\Bmn;

use App\Blameable;
use App\Helpers\MyHelper;
use App\Traits\LogTrait;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class Asuransi extends Model
{
    use Blameable;
    use HasFactory;
    use LogTrait;

    protected $table = 'vw_asset_asuransi';

    const tableKet = 'Obyek Asuransi';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'id_satker',
        'kdsatker_keu',
        'nm_satker',
        'id_barang',
        'kode_barang',
        'nm_barang',
        'nup',
        'kondisi',
        'jenis_dokumen',
        'merk',
        'tgl_rekam_pertama',
        'tgl_perolehan',
        'nilai_perolehan_pertama',
        'nilai_mutasi',
        'nilai_perolehan',
        'nilai_penyusutan',
        'nilai_buku',
        'kuantitas',
        'luas_bangunan',
        'luas_dasar_bangunan',
        'luas_dasar_bangunan',
        'nama_kpknl',
        'jalan',
        'nama_sub_satker',
    ];

    protected $casts = [
        'id' => 'string',
    ];

    public function getDataGrid($paging, $search = [])
    {
        // DB::enableQueryLog();
        $query = DB::table($this->table.' as a')
            ->select([
                'a.*',
                'c.polis_no',
                'c.polis_tgl',
                'c.polis_premi',
                'c.filename',
                'b.inst_nama',
                'b.inst_satkerkd as ms_satker_id',
            ])
            ->leftJoin('ms_satker as b', 'a.id_satker', '=', 'b.kdsatker_keu')
            ->leftJoin('asuransi_transaksi as c', function ($join) {
                $join->on('a.id', '=', 'c.id_asset');
            });
        $ms_satker_id = session('userData.current_role.ms_satker_id');
        if (MyHelper::isPelaksanaSatker()) {
            $query->where('b.inst_satkerkd', $ms_satker_id);
        }
        if (! empty($search) && isset($search['columns'])) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    $tableName = $this->table;
                    $kolom = 'a.'.$columnName;
                    if ($value) {
                        $dataType = DB::table('information_schema.columns')->select('data_type')->where('table_name', $tableName)->where('column_name', $columnName)->value('data_type');
                        if (in_array($dataType, ['integer', 'numeric', 'smallint', 'bigint'])) {
                            $q->where($kolom, '=', $value);
                        } elseif (in_array($dataType, ['timestamp without time zone', 'timestamp', 'date'])) {
                            if (strtotime($value)) {
                                $q->whereDate($kolom, '=', date('Y-m-d', strtotime($value)));
                            }

                        } elseif ($columnName == 'polis_no') {
                            if (strtolower($value) == 'sudah') {
                                $q->whereNotNull('c.polis_no');
                            } else {
                                $q->whereNull('c.polis_no');
                            }
                        } else {
                            $q->where(DB::raw("lower({$kolom})"), 'like', strtolower("%{$value}%"));
                        }
                    }
                }
            });
        }
        // $query->orderByAsc('id');
        $query->orderBy('tipe', 'ASC');
        $query->orderBy('id', 'ASC');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        // dd(DB::getQueryLog()); exit;

        return ['total' => $total, 'data' => $data];
    }
}
