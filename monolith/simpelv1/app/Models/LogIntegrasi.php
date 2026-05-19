<?php

namespace App\Models;

// use Illuminate\Contracts\Auth\MustVerifyEmail;

use Illuminate\Contracts\Database\Query\Builder;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class LogIntegrasi extends Model
{
    protected $table = 'log_integrasi';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'endpoint',
        'description',
        'total_data',
        'app',
        'duration',
    ];

    public function getDataGrid($paging, $search = [], $isRaw = false)
    {
        $query = DB::table("{$this->table} as a");
        // dd($query->paginate());
        if (! empty($search)) {
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
                        } else {
                            $q->where(DB::raw("lower({$kolom})"), 'like', strtolower("%{$value}%"));
                        }
                    }
                }
            });
        }

        $query->orderByDesc('a.created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'] ?? 10)->skip($paging['start'] ?? 0)->get();

        return ['total' => $total, 'data' => $data];
    }
    // select count(*) as unread from vw_notifikasi where ms_satker_id = '10.05' or username = 'superadmin' ;

}
