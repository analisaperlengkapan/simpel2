<?php

namespace App\Models\Bmn;

use App\Helpers\MyHelper;
use App\Traits\HooksTrait;
use App\Traits\LogTrait;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class AsuransiTransaksi extends Model
{
    use HooksTrait, LogTrait;

    protected $table = 'asuransi_transaksi';

    const tableKet = 'Obyek Asuransi';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'tipe',
        'id_asset',
        'kode_barang',
        'nup',
        'filename',
        'polis_no',
        'polis_tgl',
        'polis_premi',
        'ms_satker_id',
    ];

    public function getDataGrid($paging, $search = [])
    {
        // DB::enableQueryLog();
        $query = DB::table('vw_asset_asuransi as a');
        $query->leftJoin('ms_satker as b', 'a.id_satker', '=', 'b.kdsatker_keu');
        $query->leftJoin('asuransi_transaksi as c', function ($join) {
            $join->on('a.id', '=', 'c.id_asset');
        });

        $query->select([
            'b.inst_nama',
            'a.*',
            'c.polis_no',
            'c.polis_tgl',
            'c.polis_premi',
            'b.inst_satkerkd as ms_satker_id',
        ]);
        $ms_satker_id = session('userData.current_role.ms_satker_id');
        if (MyHelper::isPelaksanaSatker()) {
            $query->where('b.inst_satkerkd', $ms_satker_id);
        }
        if (! empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    if (empty($value)) {
                        continue;
                    }
                    $columnName = $v['data'];
                    $tableName = $this->table;
                    $kolom = 'a.'.$columnName;
                    if ($columnName == 'polis_no') {
                        if (strtolower($value) == 'sudah') {
                            $q->whereNotNull('c.polis_no');
                        } else {
                            $q->whereNull('c.polis_no');
                        }
                    } elseif ($columnName == 'inst_nama') {
                        $kolom = 'b.'.$columnName;
                        $q->where(DB::raw("lower({$kolom})"), 'like', strtolower("%{$value}%"));
                    } else {
                        $q->where(DB::raw("lower({$kolom})"), 'like', strtolower("%{$value}%"));
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
