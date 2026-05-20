<?php

namespace App\Models\Master;

use Illuminate\Database\Eloquent\Model;

class MsJenisAset extends Model
{
    protected $table = 'ms_jenis_asset';

    // protected $primaryKey = 'inst_satkerkd';
    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'name',
        'nm_table',
        'is_active',
        'updated_at',
        'created_at',
    ];
}
