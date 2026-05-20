<?php

namespace App\Models\Bmn;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class HibahFile extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'bmn_hibah_file';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id_hibah',
        'no',
        'tgl',
        'file',
    ];
}
