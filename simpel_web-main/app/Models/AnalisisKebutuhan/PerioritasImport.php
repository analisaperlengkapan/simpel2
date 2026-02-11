<?php
namespace App\Models\AnalisisKebutuhan;
use App\Models\AnalisisKebutuhan\BmnSatkerBarang;
use Maatwebsite\Excel\Concerns\ToModel;
use Maatwebsite\Excel\Concerns\WithBatchInserts;
use Maatwebsite\Excel\Concerns\WithChunkReading;
use Maatwebsite\Excel\Concerns\WithHeadingRow;

class PerioritasImport implements ToModel, WithHeadingRow, WithBatchInserts, WithChunkReading
{
    /**
     * @param array $row
     *
     * @return User|null
     */
    public function model(array $row)
    {
        BmnSatkerBarang::where('id', $row['id'])->update([
            'prioritas' => $row['prioritas']
        ]);
    }

    public function batchSize(): int
    {
        return 1000; // Adjust the batch size as needed
    }

    public function chunkSize(): int
    {
        return 1000; // Adjust the chunk size as needed
    }
}
