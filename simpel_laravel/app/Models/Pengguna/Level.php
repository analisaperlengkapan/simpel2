<?php

namespace App\Models\Pengguna;

// use Illuminate\Contracts\Auth\MustVerifyEmail;

use App\Models\Master\Master;
use Illuminate\Contracts\Database\Query\Builder;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class Level extends Model
{
    protected $table = 'ms_role';
    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'name',
        'description',
        'is_active',
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

    function getGridData($paging, $search = [])
    {
        $query = DB::table("{$this->table} as a");

        if (!empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    $tableName = $this->table;
                    $kolom = 'a.' . $columnName;
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

        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }

    function getRoleMenu($roleId)
    {
        $master = new Master();
        $sql = "WITH user_menu as (
        select distinct c.menu_id
        from ms_role_menu c
        join ms_role b on c.role_id = b.id
        where c.role_id = ?
        )

        select a.*, b.menu_id
        from ms_menu a
        left join user_menu b on a.id = b.menu_id
        where is_active = 1
        order by urutan
         ";
        $menus = DB::select($sql, [$roleId]);
        return $master->buildMenu($menus);
    }

    function delInsertUserRole($userId, $roles)
    {
        DB::table('user_role')->where(['user_id' => $userId])->delete();
        DB::table('user_role')->insert($roles);
    }
}
