<?php

namespace App\Models\Bantuan;

use App\Traits\LogTrait;
use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;
use Illuminate\Database\Query\Builder;

class Kritik extends Model
{
    use HasFactory;
    use Blameable;
    use LogTrait;
    protected $table = 'support_kritik';
    const tableKet = 'Masukan kritik dan saran';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'kode_tiket',
        'judul',
        'kritik',
        'saran',
        'status'
    ];

    function getDataGrid($paging, $search = [])
    {
        $query = DB::table($this->table)->select('*');

        if (!empty($search)) {
            $searchVal = strtolower($search['search']['value']);
            if (isset($search['filterBy'])) {
                if($search['filterBy']=='kode_tiket'){
                    if(is_numeric($searchVal))
                        $query->where(DB::raw($search['filterBy']), '=', "{$searchVal}");
                }else{
                    $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
                }
            } else {
                $query->where(function (Builder $q) use ($searchVal) {
                    $q->orWhere(DB::raw("lower(judul)"), 'like', "%{$searchVal}%")
                        ->orWhere(DB::raw("lower(kritik)"), 'like', "%{$searchVal}%")
                        ->orWhere(DB::raw("lower(saran)"), 'like', "%{$searchVal}%");
                         if($searchVal && is_numeric($searchVal))
                        $q->orWhere('kode_tiket', "=", $searchVal);
                });
            }
        }
        $query->orderByDesc('created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();
        return ['total' => $total, 'data' => $data];
    }
}
