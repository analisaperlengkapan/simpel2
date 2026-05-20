<?php

namespace App\Models\Bmn;

use Maatwebsite\Excel\Concerns\ToModel;
use Maatwebsite\Excel\Concerns\WithBatchInserts;
use Maatwebsite\Excel\Concerns\WithChunkReading;
use Maatwebsite\Excel\Concerns\WithHeadingRow;
use Maatwebsite\Excel\Concerns\WithUpserts;

class AsuransiImport implements ToModel, WithBatchInserts, WithChunkReading, WithHeadingRow, WithUpserts
{
    public function model(array $row)
    {
        if (empty($row['polis_no'])) {
            return null;
        }

        return AsuransiTransaksi::updateOrCreate(
            [
                'id_asset' => $row['id'],
            ],
            [
                'id_asset' => $row['id'],
                'kode_barang' => $row['kode_barang'],
                'polis_no' => $row['polis_no'],
                'polis_tgl' => $row['polis_tgl'],
                'polis_premi' => $row['polis_premi'],
                'ms_satker_id' => $row['ms_satker_id'],
                'nup' => $row['nup'],
            ]
        );
    }

    public function batchSize(): int
    {
        return 1000; // Adjust the batch size as needed
    }

    public function chunkSize(): int
    {
        return 1000; // Adjust the chunk size as needed
    }

    public function uniqueBy()
    {
        return 'id_asset';
    }
}
