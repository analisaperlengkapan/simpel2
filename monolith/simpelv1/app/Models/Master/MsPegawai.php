<?php

namespace App\Models\Master;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class MsPegawai extends Model
{
    protected $table = 'mv_curr_pegawai_all';

    // protected $primaryKey = 'inst_satkerkd';
    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'peg_nip_baru',
        'nama',
        'pns_mail',
        'pangkat',
        'jabatan',
        'inst_satkerkd',
        'satker',
        'jenis_kelamin',
        'tempat_lahir',
        'tgl_lahir',
        'alamat',
        'unitkerja_kd',
        'unitkerja_idk',
        'agama',
        'unitkerja_nama',
    ];

    public function getDataGrid($paging, $search = [], $select = [])
    {
        /** @var \App\Services\Grpc\IntegrasiGrpcClient $client */
        $client = app('integrasi.gateway');

        $params = [
            'page' => ($paging['start'] / $paging['length']) + 1,
            'per_page' => $paging['length'],
        ];

        // Map DataTables search parameters to API filters
        if (! empty($search) && isset($search['columns'])) {
            foreach ($search['columns'] as $col) {
                if (! empty($col['search']['value'])) {
                    if ($col['data'] === 'peg_nip_baru') {
                        $params['nip_filter'] = $col['search']['value'];
                    } elseif ($col['data'] === 'nama') {
                        $params['nama_filter'] = $col['search']['value'];
                    } elseif ($col['data'] === 'inst_satkerkd') {
                        $params['kode_satker'] = $col['search']['value'];
                    }
                }
            }
        }

        $result = $client->getEmployees($params);

        if ($result && isset($result['items'])) {
            // Map API response to match v1 structure
            $items = collect($result['items'])->map(function ($item) {
                return (object) [
                    'peg_nip_baru' => $item['nip'],
                    'nama' => $item['nama'],
                    'pns_mail' => $item['email'] ?? '',
                    'pangkat' => $item['pangkat'] ?? '',
                    'jabatan' => $item['jabatan'] ?? '',
                    'inst_satkerkd' => $item['kode_satker'] ?? '',
                    'satker' => $item['unit_kerja'] ?? '',
                    'jenis_kelamin' => $item['jk'] ?? '',
                    'agama' => $item['agama'] ?? '',
                    'tempat_lahir' => $item['tempat_lahir'] ?? '',
                    'tgl_lahir' => $item['tgl_lahir'] ?? '',
                    'unitkerja_nama' => $item['unit_kerja'] ?? '',
                ];
            });

            return [
                'total' => $result['pagination']['total_items'] ?? count($items),
                'data' => $items,
            ];
        }

        // Fallback to empty if API fails
        return ['total' => 0, 'data' => []];
    }

    public function getDataExport($search = [], $defColumns = [])
    {
        // For export, we fetch a larger chunk from the API
        $paging = ['start' => 0, 'length' => 1000];
        $result = $this->getDataGrid($paging, $search);

        return ['data' => $result['data']];
    }
}
