<?php

namespace App\Models;

use App\Helpers\MyHelper;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class Approval extends Model
{
    public static function roleCheck($data)
    {
        $currentRole = session('userData.current_role');
        $msAct = DB::table('ms_aktifitas')->where(['id' => $data['ms_aktifitas_id']])->first();

        $toSatkerInduk = $data['to_satker_induk'] ?? false;
        $nextSatker = Approval::getSatkerInduk($currentRole['ms_satker_id']);

        $currentAct = [
            'ms_aktifitas_id' => $data['ms_aktifitas_id'],
            $data['idKey'] => $data['idValue'],
            'komentar' => $data['komentar'],
            'ms_satker_id' => $toSatkerInduk ? $nextSatker : null,
            'nip' => session('userData.username'),
            'nama' => session('userData.name'),
            'pangkat' => session('userData.pangkat'),
            'jabatan' => session('userData.jabatan'),
            'role' => $currentRole['name'].MyHelper::getTingkatRole($currentRole['ms_satker_id']),
        ];
        $msAct->tingkat = MyHelper::getTingkatRole($currentRole['ms_satker_id']);
        $msAct->nextSatker = $nextSatker;
        $act = $currentAct;

        // if ($msAct->next_aktifitas) {
        //     $nextAct = [
        //         'ms_aktifitas_id' => $msAct->next_aktifitas,
        //         'pengajuan_pakaian_dinas_satker_id' => $data['pengajuan_pakaian_dinas_satker_id'],
        //     ];
        //     $act = [$currentAct, $nextAct];
        // }
        return ['act' => $act, 'nextAct' => $msAct->next_aktifitas, 'msAct' => $msAct];
    }

    public static function getSatkerInduk($currentSatkerId)
    {
        if ($currentSatkerId == '00') {
            return null;
        }

        $satker = DB::table('ms_satker')->where(['inst_satkerkd' => $currentSatkerId])->first();

        return $satker->inst_satkerinduk;
    }

    public static function getAktifitas($data)
    {
        $currentRole = session('userData.current_role');
        $msAktifitasId = $data['ms_aktifitas_id'] ?? 0;
        $group = $data['group'] ?? 'BASIC';

        return DB::table('ms_aktifitas')->where(['jawaban_dari_aktifitas' => $msAktifitasId])->get()->toArray();
    }

    public static function getCurrentAktifitas($msAktifitasId)
    {
        $currentRole = session('userData.current_role');
        $role = $currentRole['ms_role_id'];
        if ($msAktifitasId) {
            $where = ['id' => $msAktifitasId];
        } else {
            $where = ['jawaban_dari_aktifitas' => 0];
        }
        $res = DB::table('ms_aktifitas')->where($where)->first();
        $views = explode(',', $res->can_view);
        $changes = explode(',', $res->can_change);

        $canChange = in_array($role, $changes); // && $currentRole['satker_level'] == $res->tingkat;
        $canView = in_array($role, $views);

        $res->canChange = $canChange;
        $res->canView = $canView;

        return $res;
    }
}
