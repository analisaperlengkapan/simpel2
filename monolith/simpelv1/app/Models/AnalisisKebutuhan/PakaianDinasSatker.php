<?php

namespace App\Models\AnalisisKebutuhan;

use App\Traits\HooksTrait;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class PakaianDinasSatker extends Model
{
    use HasFactory, HooksTrait;

    protected $table = 'pengajuan_pakaian_dinas_satker';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'pengajuan_pakaian_dinas_id',
        'id_satker_keu',
        'id_kejati',
        'id_kejari',
        'id_cabjari',
        'created_by',
        'updated_by',
        'ms_aktifitas_id',
        'ms_satker_id',
        'ms_satker_pusat_id',
    ];

    /**
     * The attributes that should be hidden for serialization.
     *
     * @var array<int, string>
     */

    /**
     * The attributes that should be cast.
     *
     * @var array<string, string>
     */
    // protected $casts = [
    //     'email_verified_at' => 'datetime',
    //     'password' => 'hashed',
    // ];

    public function getGridData($paging, $search = [])
    {
        $query = DB::table($this->table)->orderBy('created_at', 'desc');

        if (! empty($search)) {
            $searchVal = strtolower($search['search']['value']);
            if (isset($search['filterBy'])) {
                $query->where(DB::raw("lower({$search['filterBy']})"), 'like', "%{$searchVal}%");
            } else {
                $query->where(function ($q) use ($searchVal) {
                    $q->orWhere(DB::raw('lower(nama)'), 'like', "%{$searchVal}%")
                        ->orWhere(DB::raw('lower(deskripsi)'), 'like', "%{$searchVal}%");
                });
            }
        }

        $total = $query->count();
        $data = $query->limit($paging['length'])->skip($paging['start'])->get();

        return ['total' => $total, 'data' => $data];
    }
}
