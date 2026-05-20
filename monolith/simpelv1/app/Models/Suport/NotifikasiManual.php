<?php

namespace App\Models\Suport;

// use Illuminate\Contracts\Auth\MustVerifyEmail;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Arr;
use Illuminate\Support\Facades\DB;

class NotifikasiManual extends Model
{
    protected $table = 'notifikasi_manual';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'target_role_id',
        'target_role',
        'created_by',
        'isi',
        'judul',
        'is_active',
        'url',
    ];

    public function getDataGrid($paging, $search = [], $isRaw = false)
    {
        $query = DB::table('notifikasi_manual as a');
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
        $query->orderByDesc('created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'] ?? 10)->skip($paging['start'] ?? 0)->get();

        return ['total' => $total, 'data' => $data];
    }

    public static function getRoleNotif($roleId = null)
    {
        $roleId = $roleId ?? session('userData.current_role.ms_role_id');
        $targets = DB::table('notifikasi_manual as a')->select('a.id')
            ->join('notifikasi_manual_target as b', 'a.id', '=', 'b.notifikasi_manual_id')
            ->where(['b.role_id' => $roleId, 'a.is_active' => 1])->get();
        $notifIds = Arr::pluck($targets, 'id');
        $notifs = self::whereIn('id', $notifIds)->get();

        return $notifs;
    }
}
