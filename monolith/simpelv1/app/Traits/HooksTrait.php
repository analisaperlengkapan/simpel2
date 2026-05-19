<?php

namespace App\Traits;

use App\Helpers\MyHelper;

trait HooksTrait
{
    public static function bootHooksTrait()
    {
        static::creating(function ($model) {
            $role = session('userData.current_role');
            $satkerCms = MyHelper::toIdSatkerCms($role['ms_satker_id']);
            $model->id_kejati = $satkerCms['id_kejati'];
            $model->id_kejari = $satkerCms['id_kejari'];
            $model->id_cabjari = $satkerCms['id_cabjari'];
            $model->id_satker_keu = $role['ms_satker_id_keu'] ?? null;
            $model->ms_satker_id = $role['ms_satker_id'];
            $model->ms_satker_pusat_id = $role['ms_satker_pusat_id'];
            $model->created_by = session('userData.username');
        });

        // self::created(function ($model) {
        //     // ... code here
        // });

        // self::updating(function ($model) {
        //     // ... code here
        // });

        // self::updated(function ($model) {
        //     // ... code here
        // });

        // self::deleting(function ($model) {
        //     // ... code here
        // });

        // self::deleted(function ($model) {
        //     // ... code here
        // });
    }
}
