<?php

namespace App\Models\Asset;

use App\Blameable;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class Konstruksi extends Model
{
    use Blameable;
    use HasFactory;

    protected $table = 'siman.siman_aset_kdp_kl';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'id',
        'id_satker',
        'kdsatker_keu',
        'nm_satker',
        'id_barang',
        'kode_barang',
        'nm_barang',
        'nup',
        'kondisi',
        'merk',
        'tgl_rekam_pertama',
        'tgl_perolehan',
        'nilai_perolehan_pertama',
        'nilai_mutasi',
        'nilai_perolehan',
        'nilai_penyusutan',
        'nilai_buku',
        'kuantitas',
        'jml_foto',
        'status_penggunaan',
        'status_pengelolaan',
        'no_psp',
        'tgl_psp',
    ];

    public function getDataGrid($paging, $search = [], $select = [])
    {
        /** @var \App\Services\Grpc\IntegrasiGrpcClient $client */
        $client = app('integrasi.gateway');

        $params = [
            'page' => ($paging['start'] / $paging['length']) + 1,
            'per_page' => $paging['length'],
        ];

        if (! empty($search) && isset($search['columns'])) {
            foreach ($search['columns'] as $col) {
                if (! empty($col['search']['value'])) {
                    if ($col['data'] === 'kdsatker_keu') {
                        $params['kode_satker'] = $col['search']['value'];
                    }
                }
            }
        }

        $result = $client->getAssets($client::SIMAN_CAT_KDP, $params);

        if ($result && isset($result['items'])) {
            $items = collect($result['items'])->map(function ($item) {
                return (object) [
                    'id' => $item['id'],
                    'kdsatker_keu' => $item['kode_satker'],
                    'nm_satker' => $item['nama_satker'],
                    'kode_barang' => $item['kode_barang'],
                    'nm_barang' => $item['nama_barang'],
                    'nup' => $item['nup'],
                    'kondisi' => $item['kondisi'],
                    'nilai_perolehan' => $item['nilai_perolehan'],
                    'nilai_buku' => $item['nilai_buku'],
                    'tahun_perolehan' => $item['tahun_perolehan'],
                    'status_penggunaan' => $item['status_penggunaan'],
                    'inst_nama' => $item['nama_satker'],
                ];
            });

            return [
                'total' => $result['pagination']['total_items'] ?? count($items),
                'data' => $items,
            ];
        }

        return ['total' => 0, 'data' => []];
    }

    public function getDataExport($search = [], $defColumn = null)
    {
        $paging = ['start' => 0, 'length' => 1000];
        $result = $this->getDataGrid($paging, $search);

        return ['data' => $result['data']];
    }

    public static function findOne($id)
    {
        $query = DB::table('asset_konstruksi_dalam_pengerjaan as a');
        $query->select('a.*');
        $query->where('a.id', '=', $id);

        return $query->first();
    }
}
