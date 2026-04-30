<?php

namespace App\Models\Master;

use Illuminate\Database\Eloquent\Model;

class MsDistribusiStatus extends Model
{
    protected $table = 'ms_status_penyimpanan_distribusi';
     /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'status'
    ];
}
