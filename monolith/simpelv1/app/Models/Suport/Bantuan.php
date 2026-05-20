<?php

namespace App\Models\Suport;

use App\Blameable;
use App\Traits\LogTrait;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class Bantuan extends Model
{
    use Blameable;
    use HasFactory;
    use LogTrait;

    protected $table = 'suport_bantuan';

    const tableKet = 'Panduan Penggunaan';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    /* peruahan nama file */
    protected $fillable = [
        'id',
        'judul',
        'file_panduan',
        'status',
    ];

    public function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table)->select('*');
        if (! empty($search)) {
            $searchVal = strtolower($search['search']['value']);
            if (isset($search['filterBy'])) {
                $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            } else {
                $query->where(function (Builder $q) use ($searchVal) {
                    $q->orWhere(DB::raw('lower(judul)'), 'like', "%{$searchVal}%");
                });
            }
        }
        $query->orderByDesc('created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }
}
