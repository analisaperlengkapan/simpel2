<?php

namespace App\Models\Master;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class MsPegawai extends Model
{
    protected $table = 'mv_curr_pegawai_all';

    // protected $primaryKey = 'inst_satkerkd';
    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'peg_nip_baru',
        'nama',
        'pns_mail',
        'pangkat',
        'jabatan',
        'inst_satkerkd',
        'satker',
        'jenis_kelamin',
        'tempat_lahir',
        'tgl_lahir',
        'alamat',
        'unitkerja_kd',
        'unitkerja_idk',
        'agama',
        'unitkerja_nama',
    ];

    public function getDataGrid($paging, $search = [], $select = [])
    {
        // Check if select contains alias 'a.'
        $hasAlias = false;
        if (! empty($select)) {
            foreach ($select as $col) {
                if (strpos($col, 'a.') === 0) {
                    $hasAlias = true;
                    break;
                }
            }
        }

        if ($hasAlias) {
            $query = DB::table($this->table.' as a');
        } else {
            $query = DB::table($this->table);
        }

        // Handle select columns
        if (! empty($select)) {
            $query->select($select);
        } else {
            $query->select('*');
        }

        // Handle search/filter
        if (! empty($search) && isset($search['columns'])) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal, $hasAlias) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    $tableName = $this->table;
                    if ($value) {
                        try {
                            $dataType = DB::table('information_schema.columns')->select('data_type')->where('table_name', $tableName)->where('column_name', $columnName)->value('data_type');
                            if (in_array($dataType, ['integer', 'numeric', 'smallint', 'bigint'])) {
                                if ($hasAlias) {
                                    $q->where('a.'.$columnName, '=', $value);
                                } else {
                                    $q->where($columnName, '=', $value);
                                }
                            } elseif (in_array($dataType, ['timestamp without time zone', 'timestamp', 'date'])) {
                                if (strtotime($value)) {
                                    if ($hasAlias) {
                                        $q->whereDate('a.'.$columnName, '=', date('Y-m-d', strtotime($value)));
                                    } else {
                                        $q->whereDate($columnName, '=', date('Y-m-d', strtotime($value)));
                                    }
                                }
                            } else {
                                if ($hasAlias) {
                                    $q->where(DB::raw("lower(a.{$columnName})"), 'like', strtolower("%{$value}%"));
                                } else {
                                    $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
                                }
                            }
                        } catch (\Exception $e) {
                            // If we can't determine data type, use string comparison
                            if ($hasAlias) {
                                $q->where(DB::raw("lower(a.{$columnName})"), 'like', strtolower("%{$value}%"));
                            } else {
                                $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
                            }
                        }
                    }
                }
            });
        }

        // Get total count
        $total = $query->count();

        // Apply pagination
        if (isset($paging['length']) && $paging['length'] > 0) {
            $query->limit($paging['length'])->skip($paging['start']);
        }

        $data = $query->get();

        return ['total' => $total, 'data' => $data];
    }

    public function getDataExport($search = [], $defColumns = [])
    {
        $select = ['a.peg_nip_baru', 'a.nama', 'a.pns_mail', 'a.pangkat', 'a.jabatan', 'a.alamat', 'a.satker', 'a.jenis_kelamin', 'a.agama', 'a.tempat_lahir', 'a.tgl_lahir', 'a.jenis', 'a.unitkerja_nama'];
        if ($defColumns) {
            $select = array_intersect_key($select, array_flip($defColumns));
        }

        $query = DB::table($this->table.' as a');
        $query->select($select);

        if (! empty($search) && isset($search['columns'])) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    $tableName = $this->table;
                    if ($value) {
                        try {
                            $dataType = DB::table('information_schema.columns')->select('data_type')->where('table_name', $tableName)->where('column_name', $columnName)->value('data_type');
                            if (in_array($dataType, ['integer', 'numeric', 'smallint', 'bigint'])) {
                                $q->where($columnName, '=', $value);
                            } elseif (in_array($dataType, ['timestamp without time zone', 'timestamp', 'date'])) {
                                if (strtotime($value)) {
                                    $q->whereDate($columnName, '=', date('Y-m-d', strtotime($value)));
                                }
                            } else {
                                $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
                            }
                        } catch (\Exception $e) {
                            // If we can't determine data type, use string comparison
                            $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
                        }
                    }
                }
            });
        }

        $data = $query->get();

        return ['data' => $data];
    }
}
