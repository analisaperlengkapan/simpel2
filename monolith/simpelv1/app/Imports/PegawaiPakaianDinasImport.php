<?php

namespace App\Imports;

use App\Models\PegawaiPakaianDinas;
use Illuminate\Database\Eloquent\Model;
use Maatwebsite\Excel\Concerns\ToModel;

class PegawaiPakaianDinasImport implements ToModel
{
    /**
     * @return Model|null
     */
    public function model(array $row)
    {
        return new PegawaiPakaianDinas([
            //
        ]);
    }
}
