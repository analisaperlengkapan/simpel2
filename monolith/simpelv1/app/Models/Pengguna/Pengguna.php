<?php

namespace App\Models\Pengguna;

// use Illuminate\Contracts\Auth\MustVerifyEmail;
use App\Helpers\MyHelper;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Query\Builder;
use Illuminate\Foundation\Auth\User as Authenticatable;
use Illuminate\Notifications\Notifiable;
use Illuminate\Support\Facades\DB;
use Laravel\Sanctum\HasApiTokens;
use Tymon\JWTAuth\Contracts\JWTSubject;
use Tymon\JWTAuth\Facades\JWTAuth;

class Pengguna extends Authenticatable implements JWTSubject
{
    use HasApiTokens, HasFactory, Notifiable;

    protected $table = 'users';
    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'name',
        'username',
        'email',
        'ms_satker_id',
        'satker',
        'ms_satker_pusat_id',
        'satker_pusat',
        'pangkat',
        'is_superadmin',
        'jabatan',
        'password',
        'foto',
        'firebase_token',
        'has_user'
    ];

    /**
     * The attributes that should be hidden for serialization.
     *
     * @var array<int, string>
     */
    protected $hidden = [
        'password',
        'remember_token',
    ];

    /**
     * The attributes that should be cast.
     *
     * @var array<string, string>
     */
    protected $casts = [
        'email_verified_at' => 'datetime',
        'password' => 'hashed',
    ];

    // protected $mainSql = "SELECT a.*, b.name  as role, c.inst_nama as satker from users a
    // join ms_role b on a.ms_role_id = b.id
    // join ms_satker c on a.ms_satker_id = c.inst_satkerkd ";

    static function getRoles(array $where = [])
    {
        if (empty($where))
            $where = ['user_id' => session('userData.id')];
        return DB::table('user_role')
            ->join('ms_role', 'user_role.ms_role_id', '=', 'ms_role.id')
            ->leftJoin('ms_satker', 'user_role.ms_satker_id', '=', 'ms_satker.inst_satkerkd')
            ->leftJoin('unit_kerja', 'user_role.ms_satker_pusat_id', '=', 'unit_kerja.id')
            ->where($where)->get(['user_role.*', 'ms_satker.inst_nama', 'ms_role.name', 'unit_kerja.akronim as satker_pusat']);
    }

    static function getMyInfo()
    {
        $userId = session('userData.id');
        $userInfo = DB::table('users')
            ->select(['users.*', 'ms_satker.inst_nama'])
            ->where(['users.id' => $userId])
            ->leftJoin('ms_satker', 'users.ms_satker_id', '=', 'ms_satker.inst_satkerkd')->first();
        $userInfo->roles = Pengguna::getRoles(['user_id' => $userId]);
        return $userInfo;
    }

    function getUserGrid($paging, $search = [], $isSuperadmin = false)
    {
        $query = DB::table('vw_user as a')->where('is_superadmin', '=', $isSuperadmin ? 1 : 0);

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

    static function isUserHasRole($roleId)
    {
        $userId = session('userData.id');
        $where = ['user_id' => $userId, 'id' => $roleId];
        $res = DB::table('user_role')->where($where)->count();
        if ($res < 1)
            return false;
        return Pengguna::getRoles(['user_role.id' => $roleId])[0];
    }

    public function getJWTIdentifier()
    {
        return $this->getKey();
    }

    public function getJWTCustomClaims()
    {
        $user = JWTAuth::user();
        $userData = $this->setUserdata($user);

        $customClaims = [
            'roles' => $userData['roles'],
            'current_role' => $userData['current_role']
        ];

        return $customClaims;
    }

    static function setUserdata($user)
    {

        $role = self::getRoles(['user_id' => $user->id]);
        $defaultRole = $role[0] ?? $role;
        $defaultRole->satker_level = MyHelper::getSatkerLevel($defaultRole->ms_satker_id);
        $userInfo = [
            'id' => $user->id,
            'name' => $user->name,
            'email' => $user->email,
            'pangkat' => $user->pangkat,
            'jabatan' => $user->jabatan,
            'username' => $user->username,
            'foto' => MyHelper::getFotoMysimkari($user->foto),
            'has_review' => $user->has_review,
            'current_role' => (array) $defaultRole,
            'roles' => $role,
        ];
        return $userInfo;
    }

    static function getUserByRole($msRoleId, $msSatkerId = [], $msSatkerPusat = [])
    {
        $q = DB::table('user_role as a')
            ->select(['b.username', 'b.firebase_token'])
            ->join('users as b', 'a.user_id', '=', 'b.id')
            ->where(['a.ms_role_id' => $msRoleId]);

        if (!empty($msSatkerId)) {
            $q->whereIn('a.ms_satker_id', $msSatkerId);
        }

        return $q->get();
    }

    static function getUserChanger($search = null)
    {
        $query = DB::table('vw_user as a')->limit(20);
        if ($search) {
            $query->orWhere(function (Builder $q) use ($search) {
                $q->orWhere(DB::raw('lower(name)'), 'like', "%{$search}%")
                    ->orWhere(DB::raw('lower(username)'), 'like', "%{$search}%")
                    ->orWhere(DB::raw('lower(satker)'), 'like', "%{$search}%");
            });
        }
        $data =  $query->orderBy('satker')->orderBy('name')->get();
        return $data;
    }
}
