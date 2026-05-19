<?php

namespace App\Models\AnalisisKebutuhan;

// use Illuminate\Contracts\Auth\MustVerifyEmail;

use App\Helpers\MyHelper;
use App\Models\MsSatker;
use App\Traits\LogTrait;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Arr;
use Illuminate\Support\Facades\DB;

class PakaianDinas extends Model
{
    use LogTrait;

    protected $table = 'pengajuan_pakaian_dinas';

    const tableKet = 'Pengajuan Kebutuhan Pakaian Dinas';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'nama',
        'deskripsi',
        'tgl_mulai',
        'tgl_selesai',
        'created_by',
        'is_reguler',
        'ms_aktifitas_id',
        'pilihan_satker',
        'tahun',
        'dengan_unit_kerja',
        'ms_jenis_pakaian_dinas_id',
    ];

    /**
     * The attributes that should be hidden for serialization.
     *
     * @var array<int, string>
     */

    /**
     * The attributes that should be cast.
     *
     * @var array<string, string>
     */
    // protected $casts = [
    //     'email_verified_at' => 'datetime',
    //     'password' => 'hashed',
    // ];

    public function getGridData($paging, $search = [], $filter = [])
    {
        $query = DB::table("{$this->table} as a")->select(['a.*']);
        $currentRole = session('userData.current_role');
        if (MyHelper::isPelaksanaSatker()) {
            $query->join('pengajuan_pakaian_dinas_satker_terpilih as b', function (Builder $join) use ($currentRole) {
                $j = $join->on('a.id', '=', 'b.pengajuan_pakaian_dinas_id');
                if ($currentRole['ms_satker_id'] == 00) {
                    $j->where('b.ms_satker_pusat_id', '=', session('userData.current_role.ms_satker_pusat_id'));
                } else {
                    $j->where('b.ms_satker_id', '=', session('userData.current_role.ms_satker_id'));
                }
            });
        }

        if (! empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    $tableName = $this->table;
                    $kolom = $columnName;
                    if ($value) {
                        $dataType = DB::table('information_schema.columns')->select('data_type')->where('table_name', $tableName)->where('column_name', $columnName)->value('data_type');
                        if (in_array($dataType, ['integer', 'numeric', 'smallint', 'bigint'])) {
                            $q->where($kolom, '=', $value);
                        } elseif (in_array($dataType, ['timestamp without time zone', 'timestamp', 'date'])) {
                            if (strtotime($value)) {
                                $q->whereDate($kolom, '=', date('Y-m-d', strtotime($value)));
                            }
                        } else {
                            $q->where(DB::raw("lower({$kolom})"), 'like', strtolower("%{$value}%"));
                        }
                    }
                }
            });
        }

        $query->orderBy('created_at', 'desc');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }

    public function getGridDataSatker($paging, $search, $pengajuanId, $filters = [], $isPusat = false)
    {
        // dd($filters);
        $query = DB::table('ms_satker')
            ->distinct()->select(
                [
                    'ms_satker.inst_nama as satker',
                    DB::raw("coalesce(ms_aktifitas.nama, 'Belum Input') as status"),
                    'pengajuan_pakaian_dinas_satker.ms_aktifitas_id',
                    'pengajuan_pakaian_dinas_satker.id',
                    'ms_satker.inst_satkerkd',
                    DB::raw($pengajuanId.' as pengajuan_pakaian_dinas_id'),
                    'ms_satker.is_pusat',
                ]
            )
            ->join('pengajuan_pakaian_dinas_satker_terpilih', function ($join) use ($pengajuanId, $isPusat) {
                $joinField = $isPusat ? 'ms_satker_pusat_id' : 'ms_satker_id';
                $join->on('ms_satker.inst_satkerkd', '=', "pengajuan_pakaian_dinas_satker_terpilih.{$joinField}");
                $join->where('pengajuan_pakaian_dinas_satker_terpilih.pengajuan_pakaian_dinas_id', '=', $pengajuanId);
            })
            ->leftJoin('pengajuan_pakaian_dinas_satker', function ($join) use ($pengajuanId, $isPusat) {
                $joinField = $isPusat ? 'ms_satker_pusat_id' : 'ms_satker_id';
                $join->on('ms_satker.inst_satkerkd', '=', "pengajuan_pakaian_dinas_satker.{$joinField}");
                $join->where('pengajuan_pakaian_dinas_satker.pengajuan_pakaian_dinas_id', '=', $pengajuanId);
            })
            ->leftJoin('ms_aktifitas', 'ms_aktifitas.id', '=', 'pengajuan_pakaian_dinas_satker.ms_aktifitas_id')
            ->orderBy('ms_satker.inst_satkerkd');

        foreach ($filters as $key => $value) {
            $query->where($value[0], $value[1], $value[2]);
        }

        if (! empty($search)) {
            $searchVal = strtolower($search['search']['value']);
            if (isset($search['filterBy'])) {
                $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            } else {
                $query->where(function ($q) use ($searchVal) {
                    $q->orWhere(DB::raw('lower(ms_satker.inst_nama)'), 'like', "%{$searchVal}%")
                        ->orWhere(DB::raw('lower(ms_satker.inst_satkerkd)'), 'like', "%{$searchVal}%")
                        ->orWhere(DB::raw('lower(ms_aktifitas.nama)'), 'like', "%{$searchVal}%");
                });
            }
        }
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        // ddd($data);
        return ['total' => $total, 'data' => $data];
    }

    public function getGridDataSatkerPusat($paging, $search, $pengajuanId)
    {
        $query = DB::table('ms_satker')
            ->select(['ms_satker.inst_nama as satker', DB::raw("coalesce(ms_aktifitas.nama, 'Belum Input') as status"), 'pengajuan_pakaian_dinas_satker.id'])
            ->leftJoin('pengajuan_pakaian_dinas_satker', function ($join) use ($pengajuanId) {
                $join->on('ms_satker.inst_satkerkd', '=', 'pengajuan_pakaian_dinas_satker.ms_satker_id')
                    ->where('pengajuan_pakaian_dinas_satker.pengajuan_pakaian_dinas_id', '=', $pengajuanId);
            })
            ->leftJoin('ms_aktifitas', 'ms_aktifitas.id', '=', 'pengajuan_pakaian_dinas_satker.ms_aktifitas_id')
            ->where('ms_satker.inst_satkerkd', '<>', '00')->orderBy('ms_satker.inst_satkerkd');

        if (! empty($search)) {
            $searchVal = strtolower($search['search']['value']);
            if (isset($search['filterBy'])) {
                $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            } else {
                $query->where(function ($q) use ($searchVal) {
                    $q->orWhere(DB::raw('lower(ms_satker.inst_nama)'), 'like', "%{$searchVal}%")
                        ->orWhere(DB::raw('lower(ms_satker.inst_satkerkd)'), 'like', "%{$searchVal}%")
                        ->orWhere(DB::raw('lower(ms_aktifitas.nama)'), 'like', "%{$searchVal}%");
                });
            }
        }

        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }

    public function sqlPerpakaian($satkerSudahInputIds, $filterSql)
    {
        $sqlPerPakaian = "WITH
        ukuran_l as (SELECT count(*) as jumlah, ukuran, b.ms_satker_id, b.ms_satker_pusat_id
            from pengajuan_pakaian_dinas_satker_pegawai_ukuran a
            join pengajuan_pakaian_dinas_satker b on a.pengajuan_pakaian_dinas_satker_id = b.id
            join pengajuan_pakaian_dinas_satker_pegawai c on a.pengajuan_pakaian_dinas_satker_pegawai_id = c.id
            where ukuran  is not null
            and c.jenis_kelamin = 'L'
            and b.id in ($satkerSudahInputIds)
            and pengajuan_pakaian_dinas_pakaian_id = :id
            {$filterSql}
            group by ukuran, b.ms_satker_id, b.ms_satker_pusat_id
            ),
        ukuran_p as (SELECT count(*) as jumlah, ukuran, b.ms_satker_id, b.ms_satker_pusat_id
            from pengajuan_pakaian_dinas_satker_pegawai_ukuran a
            join pengajuan_pakaian_dinas_satker b on a.pengajuan_pakaian_dinas_satker_id = b.id
            join pengajuan_pakaian_dinas_satker_pegawai c on a.pengajuan_pakaian_dinas_satker_pegawai_id = c.id
            where ukuran  is not null
            and c.jenis_kelamin = 'P'
            and b.id in ($satkerSudahInputIds)
            and pengajuan_pakaian_dinas_pakaian_id = :id
            {$filterSql}
            group by ukuran, b.ms_satker_id, b.ms_satker_pusat_id
        ),
        ukuran as ( SELECT count(*) as jumlah, ukuran, b.ms_satker_id, b.ms_satker_pusat_id
            from pengajuan_pakaian_dinas_satker_pegawai_ukuran a
            join pengajuan_pakaian_dinas_satker b on a.pengajuan_pakaian_dinas_satker_id = b.id
            join pengajuan_pakaian_dinas_satker_pegawai c on a.pengajuan_pakaian_dinas_satker_pegawai_id = c.id
            where ukuran  is not null
            and b.id in ($satkerSudahInputIds)
            and pengajuan_pakaian_dinas_pakaian_id = :id
            {$filterSql}
            group by ukuran, b.ms_satker_id, b.ms_satker_pusat_id)

        select a.*,  coalesce(l.jumlah,0) as l, coalesce(p.jumlah,0) as p
        From ukuran a
        left join ukuran_l l on a.ukuran = l.ukuran and a.ms_satker_id = l.ms_satker_id
        left join ukuran_p p on a.ukuran = p.ukuran and a.ms_satker_id = p.ms_satker_id ";

        return $sqlPerPakaian;
    }

    public function sqlPerpakaianPusat($satkerSudahInputIds, $filterSql)
    {
        $sqlPerPakaian = "WITH
        ukuran_l as (SELECT count(*) as jumlah, ukuran, b.ms_satker_id, d.eselon1 as ms_satker_pusat_id
            from pengajuan_pakaian_dinas_satker_pegawai_ukuran a
            join pengajuan_pakaian_dinas_satker b on a.pengajuan_pakaian_dinas_satker_id = b.id
            join pengajuan_pakaian_dinas_satker_pegawai c on a.pengajuan_pakaian_dinas_satker_pegawai_id = c.id
            join unit_kerja d on d.id = b.ms_satker_pusat_id
            where ukuran  is not null
            and c.jenis_kelamin = 'L'
            and d.eselon1 in ($satkerSudahInputIds)
            and pengajuan_pakaian_dinas_pakaian_id = :id
            {$filterSql}
            group by ukuran, b.ms_satker_id, d.eselon1
            ),
        ukuran_p as (SELECT count(*) as jumlah, ukuran, b.ms_satker_id, d.eselon1 as ms_satker_pusat_id
            from pengajuan_pakaian_dinas_satker_pegawai_ukuran a
            join pengajuan_pakaian_dinas_satker b on a.pengajuan_pakaian_dinas_satker_id = b.id
            join pengajuan_pakaian_dinas_satker_pegawai c on a.pengajuan_pakaian_dinas_satker_pegawai_id = c.id
            join unit_kerja d on d.id = b.ms_satker_pusat_id
            where ukuran  is not null
            and c.jenis_kelamin = 'P'
            and d.eselon1 in ($satkerSudahInputIds)
            and pengajuan_pakaian_dinas_pakaian_id = :id
            {$filterSql}
            group by ukuran, b.ms_satker_id, d.eselon1
        ),
        ukuran as ( SELECT count(*) as jumlah, ukuran, b.ms_satker_id, d.eselon1 as ms_satker_pusat_id
            from pengajuan_pakaian_dinas_satker_pegawai_ukuran a
            join pengajuan_pakaian_dinas_satker b on a.pengajuan_pakaian_dinas_satker_id = b.id
            join pengajuan_pakaian_dinas_satker_pegawai c on a.pengajuan_pakaian_dinas_satker_pegawai_id = c.id
            join unit_kerja d on d.id = b.ms_satker_pusat_id
            where ukuran  is not null
            and d.eselon1 in ($satkerSudahInputIds)
            and pengajuan_pakaian_dinas_pakaian_id = :id
            {$filterSql}
            group by ukuran, b.ms_satker_id, d.eselon1)

        select a.*,  coalesce(l.jumlah,0) as l, coalesce(p.jumlah,0) as p
        From ukuran a
        left join ukuran_l l on a.ukuran = l.ukuran and a.ms_satker_id = l.ms_satker_id and a.ms_satker_pusat_id = l.ms_satker_pusat_id
        left join ukuran_p p on a.ukuran = p.ukuran and a.ms_satker_id = p.ms_satker_id and a.ms_satker_pusat_id = p.ms_satker_pusat_id";

        return $sqlPerPakaian;
    }

    public function sqlSummaryPerPakaian($satkerSudahInputIds, $filterSql)
    {
        $summary = "WITH
        ukuran_l as (SELECT count(*) as jumlah, ukuran
            from pengajuan_pakaian_dinas_satker_pegawai_ukuran a
            join pengajuan_pakaian_dinas_satker b on a.pengajuan_pakaian_dinas_satker_id = b.id
            join pengajuan_pakaian_dinas_satker_pegawai c on a.pengajuan_pakaian_dinas_satker_pegawai_id = c.id
            where ukuran  is not null
            and c.jenis_kelamin = 'L'
            and b.id in ($satkerSudahInputIds)
            and pengajuan_pakaian_dinas_pakaian_id = :id
            {$filterSql}
            group by ukuran),
        ukuran_p as (SELECT count(*) as jumlah, ukuran
            from pengajuan_pakaian_dinas_satker_pegawai_ukuran a
            join pengajuan_pakaian_dinas_satker b on a.pengajuan_pakaian_dinas_satker_id = b.id
            join pengajuan_pakaian_dinas_satker_pegawai c on a.pengajuan_pakaian_dinas_satker_pegawai_id = c.id
            where ukuran  is not null
            and c.jenis_kelamin = 'P'
            and b.id in ($satkerSudahInputIds)
            and pengajuan_pakaian_dinas_pakaian_id = :id
            {$filterSql}
            group by ukuran
        ),
        ukuran as    ( SELECT count(*) as jumlah, ukuran
            from pengajuan_pakaian_dinas_satker_pegawai_ukuran a
            join pengajuan_pakaian_dinas_satker b on a.pengajuan_pakaian_dinas_satker_id = b.id
            join pengajuan_pakaian_dinas_satker_pegawai c on a.pengajuan_pakaian_dinas_satker_pegawai_id = c.id
            where ukuran  is not null
            and b.id in ($satkerSudahInputIds)
            and pengajuan_pakaian_dinas_pakaian_id = :id
            {$filterSql}
            group by ukuran )

        select a.*,  coalesce(l.jumlah,0) as l, coalesce(p.jumlah,0) as p
        from ukuran a
        left join ukuran_l l on a.ukuran = l.ukuran
        left join ukuran_p p on a.ukuran = p.ukuran ";

        return $summary;
    }

    public function sqlSummaryPerPakaianPusat($satkerSudahInputIds, $filterSql)
    {

        $summary = "WITH
        ukuran_l as (SELECT count(*) as jumlah, ukuran
            from pengajuan_pakaian_dinas_satker_pegawai_ukuran a
            join pengajuan_pakaian_dinas_satker b on a.pengajuan_pakaian_dinas_satker_id = b.id
            join pengajuan_pakaian_dinas_satker_pegawai c on a.pengajuan_pakaian_dinas_satker_pegawai_id = c.id
            join unit_kerja d on d.id = b.ms_satker_pusat_id
            where ukuran  is not null
            and c.jenis_kelamin = 'L'
            and d.eselon1 in ($satkerSudahInputIds)
            and pengajuan_pakaian_dinas_pakaian_id = :id
            {$filterSql}
            group by ukuran),
        ukuran_p as (SELECT count(*) as jumlah, ukuran
            from pengajuan_pakaian_dinas_satker_pegawai_ukuran a
            join pengajuan_pakaian_dinas_satker b on a.pengajuan_pakaian_dinas_satker_id = b.id
            join pengajuan_pakaian_dinas_satker_pegawai c on a.pengajuan_pakaian_dinas_satker_pegawai_id = c.id
            join unit_kerja d on d.id = b.ms_satker_pusat_id
            where ukuran  is not null
            and c.jenis_kelamin = 'P'
            and d.eselon1 in ($satkerSudahInputIds)
            and pengajuan_pakaian_dinas_pakaian_id = :id
            {$filterSql}
            group by ukuran
        ),
        ukuran as    ( SELECT count(*) as jumlah, ukuran
            from pengajuan_pakaian_dinas_satker_pegawai_ukuran a
            join pengajuan_pakaian_dinas_satker b on a.pengajuan_pakaian_dinas_satker_id = b.id
            join pengajuan_pakaian_dinas_satker_pegawai c on a.pengajuan_pakaian_dinas_satker_pegawai_id = c.id
            join unit_kerja d on d.id = b.ms_satker_pusat_id
            where ukuran  is not null
            and d.eselon1 in ($satkerSudahInputIds)
            and pengajuan_pakaian_dinas_pakaian_id = :id
            {$filterSql}
            group by ukuran )

        select a.*,  coalesce(l.jumlah,0) as l, coalesce(p.jumlah,0) as p
        from ukuran a
        left join ukuran_l l on a.ukuran = l.ukuran
        left join ukuran_p p on a.ukuran = p.ukuran ";

        return $summary;
    }

    public function dataRekap($params)
    {

        $header = self::where(['id' => $params['pengajuan_id']])->first();
        if ($params['kejati_id'] == '00') {
            $isPusat = true;
            [$header, $listSatker, $satkerSudahInput] = $this->getDataRekapPusat($params);
            $eselon1s = implode(',', Arr::map($satkerSudahInput->toArray(), function ($xx) {
                return $xx['eselon1'].'::text';
            }));
        } else {
            $isPusat = false;
            [$header, $listSatker, $satkerSudahInput] = $this->getDataRekap($params);
        }

        // dd($listSatker, $satkerSudahInput);

        if (count($satkerSudahInput) == 0) {
            throw new \Exception('Satker Belum Input', 1);
        }

        $satkerSudahInputIds = implode(',', Arr::pluck($satkerSudahInput, 'id'));

        // $pakaians = PakaianDinasPakaian::where(['pengajuan_pakaian_dinas_id' => $params['pengajuan_id']])->get();

        $pakaians = DB::select(
            'WITH ms_ukurans as (
                SELECT string_agg(ukuran, \',\') as ukurans, "group" from ms_ukuran mu group by "group"
                )
            SELECT a.*, b.ukurans from pengajuan_pakaian_dinas_pakaian a
            join ms_ukurans b on a.spesifikasi_ukuran_group = b."group"
            where pengajuan_pakaian_dinas_id = ?',
            [$params['pengajuan_id']]
        );

        $filterSql = '';

        foreach ($params['filter'] ?? [] as $field => $value) {
            if ($value == '' || $field == 'jenis_kelamin') {
                continue;
            }

            if ($field == 'eselon') {
                if ($value == 'non') {
                    $filterSql .= " AND  c.{$field} is NULL ";
                } else {
                    $filterSql .= " AND  c.{$field} like '{$value}/%'";
                }
            } else {
                $filterSql .= " AND  c.{$field} = '{$value}'";
            }
        }

        $dataSummaryPakaian = [];
        foreach ($pakaians as $key => $pakaian) {
            $ukuranya = explode(',', $pakaian->ukurans);
            if (
                $isPusat &&
                $params['ms_satker_id'] == 'all'
            ) {
                $dataPerPakaian = DB::select($this->sqlPerpakaianPusat($eselon1s, $filterSql), ['id' => $pakaian->id]);
                $dataSummary = DB::select($this->sqlSummaryPerPakaianPusat($eselon1s, $filterSql), ['id' => $pakaian->id]);
            } else {
                $dataPerPakaian = DB::select($this->sqlPerPakaian($satkerSudahInputIds, $filterSql), ['id' => $pakaian->id]);
                $dataSummary = DB::select($this->sqlSummaryPerPakaian($satkerSudahInputIds, $filterSql), ['id' => $pakaian->id]);
            }
            $dataSummaryPakaian[$pakaian->id] = $this->mapSummary($dataSummary);
            $dataUkuran[$pakaian->id] = $this->mapSatker($dataPerPakaian, $isPusat);
            // $dataPerSatker[$inputanSatker->ms_satker_id] = DB::table('pengajuan_pakaian_dinas_satker_pegawai as a')
            //     ->select(['a.*', 'b.nama', 'c.inst_nama'])
            //     ->join('mv_curr_pegawai_all as b', 'a.nip', '=', 'b.peg_nip_baru')
            //     ->join('ms_satker as c', 'c.inst_satkerkd', '=', $inputanSatker->ms_satker_id, 'inner', true)
            //     ->where('a.pengajuan_pakaian_dinas_satker_id', $inputanSatker['id'])->get();
        }

        return [$header, $pakaians, $dataUkuran, $listSatker, $dataSummaryPakaian];
    }

    public function getDataRekapPusat($params)
    {
        $header = self::where(['id' => $params['pengajuan_id']])->first();
        $satkerSudahInputQ = PakaianDinasSatker::where(['pengajuan_pakaian_dinas_id' => $params['pengajuan_id'], 'ms_satker_id' => '00'])
            ->join('unit_kerja as b', 'b.id', '=', 'pengajuan_pakaian_dinas_satker.ms_satker_pusat_id')
            ->select(['pengajuan_pakaian_dinas_satker.*', 'b.eselon1']);

        if ($params['ms_satker_id'] == 'all') {
            $listSatker = DB::table('pengajuan_pakaian_dinas_satker as a')
                ->select(['a.*', 'c.id as inst_satkerkd', 'c.nama as inst_nama'])
                ->join('unit_kerja as b', 'a.ms_satker_pusat_id', '=', 'b.id')
                ->join('unit_kerja as c', 'b.eselon1', '=', 'c.id')
                ->where('a.ms_satker_id', '=', '00')->orderBy('c.id')->get();
            $satkerInfo = MsSatker::where(['inst_satkerkd' => '00'])->first();
            $header['inst_nama'] = 'WILAYAH '.$satkerInfo->inst_nama;
        } else {
            $listSatker = DB::table('ms_satker')->where(['is_pusat' => 1, 'inst_satkerkd' => $params['ms_satker_id']])->orderBy('inst_satkerkd')->get();
            $satkerInfo = $listSatker[0];
            $header['inst_nama'] = $satkerInfo->inst_nama;
            $satkerSudahInputQ->where(['ms_satker_pusat_id' => $satkerInfo->inst_satkerkd]);
        }
        $satkerSudahInput = $satkerSudahInputQ->get();

        // $eselon = DB::table('unit_kerja')
        //     ->select('id', 'nama')->whereIn('id', $satkerSudahInputQ->pluck('eselon1'))->get();
        // dd($eselon);
        return [$header, $listSatker, $satkerSudahInput];
    }

    public function getDataRekap($params)
    {
        $header = self::where(['id' => $params['pengajuan_id']])->first();
        if ($params['ms_satker_id'] == 'all') {
            $filterSatker = $params['kejati_id'];
            $listSatker = DB::table('ms_satker')->where('inst_satkerkd', 'like', "{$filterSatker}%")->orderBy('inst_satkerkd')->get();
            $satkerInfo = MsSatker::where(['inst_satkerkd' => $filterSatker])->first();
            $satkerSudahInputQ = PakaianDinasSatker::where(['pengajuan_pakaian_dinas_id' => $params['pengajuan_id']])->where('ms_satker_id', 'like', "{$filterSatker}%");
            $header['inst_nama'] = 'WILAYAH '.$satkerInfo->inst_nama;
        } else {
            $filterSatker = $params['ms_satker_id'];
            $satkerInfo = MsSatker::where(['inst_satkerkd' => $filterSatker])->first();
            $listSatker = DB::table('ms_satker')->where('inst_satkerkd', $filterSatker)->get();
            $satkerSudahInputQ = PakaianDinasSatker::where(['pengajuan_pakaian_dinas_id' => $params['pengajuan_id']])->where('ms_satker_id', $filterSatker);
            $header['inst_nama'] = $satkerInfo->inst_nama;
        }

        $satkerSudahInput = $satkerSudahInputQ->get();

        return [$header, $listSatker, $satkerSudahInput];
    }

    public function mapSatker($data, $isPusat)
    {
        foreach ($data as $key => $value) {
            $satkerId = $isPusat ? $value->ms_satker_pusat_id : $value->ms_satker_id;
            $ukuran = $value->ukuran;
            $dataPerSatker[$satkerId][$ukuran] = (array) $value;
        }

        return $dataPerSatker ?? [];
    }

    public function mapSummary($data)
    {
        foreach ($data as $key => $value) {
            $dataSummary[$value->ukuran] = (array) $value;
        }

        return $dataSummary ?? [];
    }

    public function getDataDaftar($params)
    {
        $header = self::where(['id' => $params['pengajuan_id']])->first();
        if ($params['kejati_id'] == '00') {
            $isPusat = true;
            [$header, $listSatker, $satkerSudahInput] = $this->getDataRekapPusat($params);
        } else {
            $isPusat = false;
            [$header, $listSatker, $satkerSudahInput] = $this->getDataRekap($params);
        }

        if (count($satkerSudahInput) == 0) {
            throw new \Exception('Satker Belum Input', 1);
        }

        $satkerSudahInputIds = Arr::pluck($satkerSudahInput, 'id');

        $pakaians = PakaianDinasPakaian::where(['pengajuan_pakaian_dinas_id' => $params['pengajuan_id']])->get();

        $ukuranPegawais = PakaianDinasSatkerPegawaiUkuran::whereIn('pengajuan_pakaian_dinas_satker_id', $satkerSudahInputIds)->get();
        foreach ($satkerSudahInput as $key => $inputanSatker) {
            $satkerId = $isPusat ? $inputanSatker->ms_satker_pusat_id : $inputanSatker->ms_satker_id;
            $que = DB::table('pengajuan_pakaian_dinas_satker_pegawai as a')
                ->select(['a.*', 'b.nama', 'c.inst_nama'])
                ->join('mv_curr_pegawai_all as b', 'a.nip', '=', 'b.peg_nip_baru')
                ->join('ms_satker as c', 'c.inst_satkerkd', '=', $satkerId, 'inner', true)
                ->where('a.pengajuan_pakaian_dinas_satker_id', $inputanSatker['id'])->orderBy('b.eselon')->orderByDesc('b.gol_kd');
            foreach ($params['filter'] ?? [] as $field => $value) {
                if ($value == '') {
                    continue;
                }

                // if ($field == 'eselon') {
                //     $value = "{$value}/%";
                // }

                if ($field == 'eselon') {
                    $op = 'like';
                    $sVal = "{$value}/%";
                    if ($value == 'non') {
                        $op = 'is';
                        $sVal = 'NULL';
                        $que->whereNull("a.{$field}");
                    } else {
                        $que->where("a.{$field}", 'like', $sVal);
                    }
                } else {
                    $que->where("a.{$field}", '=', $value);
                }
                // $que->where("a.{$field}", $field == 'eselon' ? 'like' : '=', $value);
            }

            $dataPerSatker[$satkerId] = $que->get();
        }

        return [$header, $pakaians, $ukuranPegawais, $listSatker, $dataPerSatker, $isPusat];
    }
}
