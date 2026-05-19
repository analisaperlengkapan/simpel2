<?php

namespace App\Traits;

use App\Helpers\MyHelper;
use App\Models\Pengguna\Aktifitas;

trait LogTrait
{
    public static function bootLogTrait()
    {

        self::created(function ($model) {
            // ... code here
            $log = MyHelper::generateLogData('TAMBAH', $model);
            Aktifitas::create($log);
        });

        // self::updating(function ($model) {
        //     // ... code here
        // });

        self::updating(function ($model) {
            $log = MyHelper::generateLogData('UBAH', $model);
            Aktifitas::create($log);
        });

        // self::updated(function ($model) {
        //     // ... code here
        // });

        // self::deleting(function ($model) {
        //     // ... code here
        // });

        self::deleted(function ($model) {
            $log = MyHelper::generateLogData('HAPUS', $model);
            Aktifitas::create($log);
        });
    }
}
