<?php

namespace App\Models\Bmn;

use App\Helpers\MyHelper;
use App\Traits\LogTrait;
use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class AsuransiTransaksiKlaim extends Model
{
    use HasFactory;
    use Blameable;
    use LogTrait;
    protected $table = 'asuransi_transaksi_klaim';
    const tableKet = 'Claim Asuransi';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'created_by',
        'surat_no',
        'surat_tgl',
        'filename',
        'ms_satker_id',
        'asuransi_transaksi_id'
    ];

    function getDataGrid($paging, $search = [])
    {
        //DB::enableQueryLog();
        $query = DB::table("{$this->table} as a")
            ->join('ms_satker as b', 'a.ms_satker_id', '=', 'b.inst_satkerkd')
            ->join('asuransi_transaksi as c', 'a.asuransi_transaksi_id', '=', 'c.id')
            ->join('vw_asset_asuransi as d', 'c.id_asset', '=', 'd.id');

        $query->select([
            'b.inst_nama',
            'a.*',
            'c.polis_no',
            'c.polis_tgl',
            'c.polis_premi',
            'd.kode_barang',
            'd.nm_barang',
            'd.nup',
        ]);
        $ms_satker_id = session('userData.current_role.ms_satker_id');
        if (MyHelper::isPelaksanaSatker()) {
            $query->where('a.ms_satker_id', $ms_satker_id);
        }
        if (!empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    if (empty($value))
                        continue;
                    $columnName = $v['data'];
                    $tableName = $this->table;
                    $kolom = 'a.' . $columnName;
                    if ($columnName == 'polis_no') {
                        if (strtolower($value) == 'sudah') {
                            $q->whereNotNull('c.polis_no');
                        } else {
                            $q->whereNull('c.polis_no');
                        }
                    } elseif ($columnName == 'inst_nama') {
                        $kolom = 'b.' . $columnName;
                        $q->where(DB::raw("lower({$kolom})"), 'like', strtolower("%{$value}%"));
                    } else {
                        $q->where(DB::raw("lower({$kolom})"), 'like', strtolower("%{$value}%"));
                    }
                }
            });
        }

        //$query->orderByAsc('id');
        $query->orderByDesc('a.created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }

}
