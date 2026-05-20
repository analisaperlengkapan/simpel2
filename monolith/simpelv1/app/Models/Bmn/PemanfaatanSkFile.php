<?php

namespace App\Models\Bmn;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class PemanfaatanSkFile extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'bmn_pemanfaatan_sk';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id_pemanfaatan',
        'no_sk',
        'tgl_sk',
        'file_sk',
    ];
}
