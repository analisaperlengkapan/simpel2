<?php

namespace App\Models\Bmn\PermohonanMonitor;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class PermohonanPenghapusanBmnMonitorFile extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'permohonan_penghapusan_bmn_sk_file_lain';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'pengajuan_id',
        'nomor',
        'tanggal',
        'file',
        'ket',
    ];

    public static function getDetail($pengajuan_id)
    {
        $sql = 'SELECT a.* FROM permohonan_penghapusan_bmn_sk_file_lain a WHERE a.pengajuan_id = ? ';
        $result = DB::select($sql, [$pengajuan_id]);

        return $result;
    }
}
