<?php

namespace App\Models\Suport;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class Faquser extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'suport_faq';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'pertanyaan',
        'jawaban',
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
                    $q->orWhere(DB::raw('lower(pertanyaan)'), 'like', "%{$searchVal}%")
                        ->orWhere(DB::raw('lower(jawaban)'), 'like', "%{$searchVal}%");
                    if ($searchVal && is_numeric($searchVal)) {
                        $q->orWhere('kategori', '=', $searchVal);
                    }
                });
            }
        }
        $query->orderByDesc('created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }
}
