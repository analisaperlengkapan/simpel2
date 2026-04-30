<?php

namespace App\Imports;

use App\Models\PegawaiPakaianDinas;
use Maatwebsite\Excel\Concerns\ToModel;

class PegawaiPakaianDinasImport implements ToModel
{
    /**
    * @param array $row
    *
    * @return \Illuminate\Database\Eloquent\Model|null
    */
    public function model(array $row)
    {
        return new PegawaiPakaianDinas([
            //
        ]);
    }
}
