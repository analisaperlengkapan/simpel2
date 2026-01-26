<?php

namespace App\Models\Master;

use App\Helpers\MyHelper;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class Master extends Model
{
    use HasFactory;

    function getMenus($role, $isMobile = false)
    {

        $sql = "WITH user_menu as (
        select distinct c.menu_id
        from ms_role_menu c
        join ms_role b on c.role_id = b.id ";


        if ($role) {
            $sql .= " JOIN user_role a on b.id = {$role} ";
        }

        $filterAktif = $isMobile ? 'is_active_mobile' : 'is_active';

        $sql .= ")";
        $sql .= " SELECT a.*, b.menu_id, a.route||'?active_route='||a.id as active_route
        from ms_menu a
        join user_menu b on a.id = b.menu_id
        where {$filterAktif} = 1 ";

        $sql .= " order by urutan, name";

        $menus = DB::select($sql);
        if (!empty($menus))
            return $this->buildMenu($menus);
    }

    function buildMenu($menus)
    {
        $level = DB::selectOne('SELECT MAX(level) as max_level from ms_menu');
        $maxLevel = $level->max_level;

        foreach ($menus as $menu) {
            $menu = (array) $menu;
            $levels[$menu['level']][] = $menu;
        }

        for ($i = $maxLevel; $i > 1; $i--) {
            $parentLevel = $i - 1;
            if (isset($levels[$i])) {
                foreach ($levels[$i] as $menu) {
                    $parentKey = array_search($menu['parent'], array_column($levels[$parentLevel], 'id'));
                    $levels[$parentLevel][$parentKey]['childs'][] = $menu;
                }
            }
        }
        return $levels[1];
    }

    static function getSatkerDashboard($excludeKejagung = false)
    {
        $q = DB::table('ms_satker')
            ->whereRaw('length(inst_satkerkd) > 2')
            ->whereRaw('kdsatker_keu is not null')
            //->whereNotNull('kdsatker_keu')
            ->orderBy('inst_satkerkd', 'asc');

        return
            $q->get()->toArray();
    }
    static function getWilayahDashboard($excludeKejagung = false)
    {
        //DB::enableQueryLog();
        return DB::table('ms_satker')
            ->whereRaw('length(inst_satkerkd) = 2')
            ->whereRaw('kdsatker_keu is not null')
            ->orderBy('inst_satkerkd')->get()->toArray();


        //echo dd(DB::getQueryLog()); exit;
    }

    static function getSatkers($excludeKejagung = false)
    {
        $q = DB::table('ms_satker')
            ->orderBy('inst_satkerkd', 'asc');
        if ($excludeKejagung) {
            $q->where('inst_satkerkd', '<>', '00');
        }
        return
            $q->get()->toArray();
    }

    static function getSatkerWilayah($userSatkerId = null)
    {
        $q = DB::table('ms_satker')->orderBy('inst_satkerkd', 'asc');
        if (!$userSatkerId) {
            $userSatkerId = session('userData.current_role.ms_satker_id');
        }
        $satkers = [];
        $pusats = [];
        $wilayahs = [];

        if ($userSatkerId != '00') {
            $q->where('inst_satkerkd', 'like', $userSatkerId . '%');
        }
        $satkerRaw = $q->get();

        foreach ($satkerRaw as $key => $satker) {
            if (strlen($satker->inst_satkerkd) == 2) {
                $wilayahs[] = $satker;
            }

            if ($satker->is_pusat == 1) {
                $pusats[] = $satker;
            } else {
                $satkers[] = $satker;
            }
        }
        return ['wilayahs' => $wilayahs, 'satkers' => $satkers, 'pusats' => $pusats];
    }

    static function getSatkersPusat()
    {
        $q = DB::table('ms_satker_pusat')
            ->orderBy('id', 'asc');
        return
            $q->get()->toArray();
    }

    static function getSatkersKeu()
    {
        $q = DB::table('ms_satker')
            ->whereNotNull('kdsatker_keu') // Filter where the column is not null
            ->where('kdsatker_keu', '!=', '')
            ->orderBy('inst_satkerkd', 'asc');
        return
            $q->get()->toArray();
    }

    static function getSatkersSakti()
    {
        $q = DB::table('ms_satker_sakti')
            ->orderBy('kdsatker', 'asc');
        return
            $q->get()->toArray();
    }

    static function getRoles()
    {
        $q = DB::table('ms_role');
        return
            $q->where('id', '<>', config('constants.superadmin_role_id'))->get()->toArray();
    }

    static function getPegawaiByNip($nip)
    {
        return DB::selectOne("SELECT a.*
        , case when c.eselon_simple in('3', '4') then c.eselon2 else c.id end as mapped_unit_kerja
        , d.akronim  as mapped_unit_kerja_nama
        from mv_curr_pegawai_all a
        left join ms_satker b on a.inst_satkerkd = b.inst_satkerkd and a.inst_satkerkd = '00'
        left join unit_kerja c on a.unitkerja_kd  = c.id
        left join unit_kerja d on  case when c.eselon_simple in('3', '4') then c.eselon2 else c.id end = d.id
        WHERE a.peg_nip_baru = ? ", [$nip]);
        // return DB::table('mv_curr_pegawai_all')->where(['peg_nip_baru' => $nip])->first();
    }

    function gridDataPegawaiDashboard($paging, $search = [], $filter = [])
    {
        $query = DB::table('vw_pegawai_dashboard');
        if (!empty($search)) {
            $searchVal = $search['columns'];
            $query->where(function (Builder $q) use ($searchVal) {
                foreach ($searchVal as $k => $v) {
                    $value = $v['search']['value'];
                    $columnName = $v['data'];
                    if ($value) {
                        $q->where(DB::raw("lower({$columnName})"), 'like', strtolower("%{$value}%"));
                    }
                }
            });
        }
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }

    static function getPegawaiBySatker($where = [])
    {
        return DB::table('mv_curr_pegawai_all')->where($where)->select(['*', 'peg_nip_baru as nip'])->orderBy('nama')->get();
    }

    static function getPeg()
    {
        $q = DB::table('mv_curr_pegawai_all')
            ->whereNotNull('peg_nip_baru') // Filter where the column is not null
            ->where('peg_nip_baru', '!=', '')
            ->orderBy('peg_nip_baru', 'asc');
        return
            $q->get()->toArray();
    }

    static function getSatuan()
    {
        return DB::table('ms_satuan')->get()->toArray();
    }

    static function getJenisAsset()
    {
        return DB::table('ms_jenis_asset')->get()->toArray();
    }

    static function getMsUkuranGroup($isHeader = false)
    {
        $q = DB::table('ms_ukuran');
        if ($isHeader) {
            $q->distinct()->select('group');
        } else {
            $q->orderBy('urutan');
        }
        return $q->get()->toArray();
    }

    static function getMsUkuranGroupMapped($groups, $default = null, $json = null)
    {
        $ukurans = DB::table('ms_ukuran')->whereIn('group', $groups)->orderBy('group')->get();
        foreach ($ukurans as $ukuran) {
            $data[$ukuran->group][] = $ukuran->ukuran;
        }
        foreach ($data as $ukuranGroup => $ukurans) {
            $options = MyHelper::generateSelectOptions(['data' => $ukurans, 'selected' => $default[$ukuranGroup] ?? null, 'type' => $json]);
            $return[$ukuranGroup] = $options;
        }
        return $return;
    }

    static function getBarangAset($where = [])
    {
        $query = DB::table('vw_asset_barang')->select(['kode_barang', 'nm_barang as nama_barang', 'ms_jenis_asset_id', 'ms_jenis_asset.nm_table'])->distinct();
        $query->leftJoin('ms_jenis_asset', 'vw_asset_barang.ms_jenis_asset_id', 'ms_jenis_asset.id');
        if (!empty($where)) {
            $query->whereIn('kode_barang', $where);
        }
        return $query->get()->toArray();
    }
    static function getBarangAsetBmn($where = [])
    {
        //$query = DB::table('vm_aset_bmn_bangunan')->select(['nup','kode_barang', 'nama_barang', 'keterangan'])->distinct();
        //if (!empty($where)) {
        //    $query->whereIn('kode_barang', $where);
        //}
        //return $query->get()->toArray();
        $satker = session('userData.current_role.ms_satker_id_keu');
        $q = DB::table('vm_aset_bmn_bangunan')
            ->where('id_satker', '=', $satker)
            ->orderBy('kode_barang', 'asc');
        //if (!empty($where)) {
        //    $q->whereIn('kode_barang', $where);
        //}
        return $q->get()->toArray();
    }

    static function gettopik()
    {
        $q = DB::table('support_topik')
            ->where('status', '!=', 'Close')
            ->orderBy('id', 'asc');
        return
            $q->get()->toArray();
    }

    static function getrole()
    {
        $q = DB::table('ms_role')
            ->where('is_active', '!=', '0')
            ->orderBy('id', 'asc');
        return
            $q->get()->toArray();
    }
}
