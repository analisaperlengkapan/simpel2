<?php

namespace App\Models\Bmn;


use App\Helpers\MyHelper;
use ErrorException;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class Approval extends Model
{
    static function roleCheck($data)
    {
        $currentRole = session('userData.current_role');
        $msAct = DB::table('ms_aktifitas_user')->where(['id' => $data['ms_aktifitas_id']])->first();

        if ($msAct->role_id != $currentRole['ms_role_id']) {
            throw new ErrorException('Role Tidak Sesuai');
        }

        $toSatkerInduk = $data['to_satker_induk'] ?? false;
        $nextSatker = Approval::getSatkerInduk($currentRole['ms_satker_id']);
        $tingkat = MyHelper::getTingkatRole($currentRole['ms_satker_id']);
        $currentAct = [
            'ms_aktifitas_id' => $data['ms_aktifitas_id'],
            'pengajuan_pakaian_dinas_satker_id' => $data['pengajuan_pakaian_dinas_satker_id'],
            'komentar' => $data['komentar'],
            'ms_satker_id' => $toSatkerInduk ? $nextSatker : null,
            'nip' => session('userData.username'),
            'nama' => session('userData.name'),
            'pangkat' => session('userData.pangkat'),
            'jabatan' => session('userData.jabatan'),
            'role' => "{$currentRole['name']}  ({$tingkat})",
        ];

        $act = $currentAct;
        if ($msAct->next_aktifitas) {
            $nextAct = [
                'ms_aktifitas_id' => $msAct->next_aktifitas,
                'pengajuan_pakaian_dinas_satker_id' => $data['pengajuan_pakaian_dinas_satker_id'],
            ];
            $act = [$currentAct, $nextAct];
        }
        return $act;
    }

    static function getSatkerInduk($currentSatkerId)
    {
        if ($currentSatkerId == '00') {
            return null;
        }

        $satker = DB::table('ms_satker')->where(['inst_satkerkd' => $currentSatkerId])->first();
        return $satker->inst_satkerinduk;
    }

    static function getAktivitas($data)
    {
        $msAktivitasId = $data['ms_aktifitas_id'] ?? 0;
        $group = $data['group'] ?? 'BASIC';
        $role = session('userData.current_role.ms_role_id');
        return DB::table('ms_aktifitas_user')->where(['role_id' => $role, 'jawaban_dari_aktifitas' => $msAktivitasId, 'group' => $group])->get()->toArray();
    }

    static function getCurrentAktivitas($msAktivitasId)
    {
        if (!$msAktivitasId)
            return [];
        return DB::table('ms_aktifitas_user')->where(['id' => $msAktivitasId])->first();
    }



}
