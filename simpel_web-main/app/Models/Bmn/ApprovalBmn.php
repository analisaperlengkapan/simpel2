<?php

namespace App\Models\Bmn;


use App\Helpers\MyHelper;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class ApprovalBmn extends Model
{
    static function roleCheck($data)
    {
        $currentRole = session('userData.current_role');
        $msAct = DB::table('ms_aktifitas_user')->where(['id' => $data['ms_aktifitas_id']])->first();

        $toSatkerInduk = $data['to_satker_induk'] ?? false;
        $nextSatker = ApprovalBmn::getSatkerInduk($currentRole['ms_satker_id']);

        $currentAct = [
            'ms_aktifitas_id' => $data['ms_aktifitas_id'],
            'pengajuan_id' => $data['pengajuan_id'],
            'komentar' => $data['komentar'],
            'ms_satker_id' => $toSatkerInduk ? $nextSatker : null,
            'nip' => session('userData.username'),
            'nama' => session('userData.name'),
            'pangkat' => session('userData.pangkat'),
            'jabatan' => session('userData.jabatan'),
            'role' => $currentRole['name'] . MyHelper::getTingkatRole($currentRole['ms_satker_id']),
            'created_at' => now(),
        ];

        $act = $currentAct;
        return ['act' => $act, 'nextAct' => $msAct->next_aktifitas];
    }

    static function getSatkerInduk($currentSatkerId)
    {
        if ($currentSatkerId == '00') {
            return '00';
        }

        $satker = DB::table('ms_satker')->where(['inst_satkerkd' => $currentSatkerId])->first();
        $satkernya = $satker ? $satker->inst_satkerinduk:'00';
        return $satkernya;
    }

    static function getAktifitas($data)
    {
        $currentRole = session('userData.current_role');
        $msAktifitasId = $data['ms_aktifitas_id'] ?? 0;
        $group = $data['group'] ?? 'BASIC';
        return DB::table('ms_aktifitas_user')->where(['jawaban_dari_aktifitas' => $msAktifitasId, 'group' => $group])->get()->toArray();
    }

    static function getCurrentAktifitas($msAktifitasId)
    {
        $currentRole = session('userData.current_role');
        $role = $currentRole['ms_role_id'];
        if ($msAktifitasId) {
            $where = ['id' => $msAktifitasId];
        } else {
            $where = ['jawaban_dari_aktifitas' => 0];
        }
        $res = DB::table('ms_aktifitas_user')->where($where)->first();
        $views = explode(',', $res->can_view);
        $changes = explode(',', $res->can_change);

        $canChange = in_array($role, $changes);

        if($msAktifitasId == 1002) $canChange = in_array($role, $changes) && session('userData.jabatan') == 'Kepala Sub Bagian LAYANAN PENGADAAN';
        if($msAktifitasId == 1004) $canChange = in_array($role, $changes) && session('userData.jabatan') == 'Kepala Biro PERLENGKAPAN';

        $canView = in_array($role, $views);

        $res->canChange = $canChange;
        $res->canView = $canView;

        return $res;
    }
}
