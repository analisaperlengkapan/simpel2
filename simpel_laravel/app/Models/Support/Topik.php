<?php

namespace App\Models\Support;

use App\Traits\LogTrait;
use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class Topik extends Model
{
    use HasFactory;
    use Blameable;
    use LogTrait;
    protected $table = 'support_topik';
    const tableKet = 'Topik Kategori';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'topik',
        'status',
    ];

    function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table)->select('*');


        $query->orderByDesc('created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }
}
