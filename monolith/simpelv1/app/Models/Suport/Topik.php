<?php

namespace App\Models\Suport;

use App\Blameable;
use App\Traits\LogTrait;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class Topik extends Model
{
    use Blameable;
    use HasFactory;
    use LogTrait;

    protected $table = 'suport_topik';

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

    public function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table)->select('*');

        $query->orderByDesc('created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }
}
