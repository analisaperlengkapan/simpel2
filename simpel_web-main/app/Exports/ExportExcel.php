<?php

namespace App\Exports;

use Illuminate\Contracts\View\View;
use Maatwebsite\Excel\Concerns\FromArray;
use Maatwebsite\Excel\Concerns\FromView;
use Maatwebsite\Excel\Concerns\ShouldAutoSize;
use Maatwebsite\Excel\Concerns\WithCustomValueBinder;
use Maatwebsite\Excel\Concerns\WithHeadings;
use Maatwebsite\Excel\Concerns\WithTitle;
use PhpOffice\PhpSpreadsheet\Cell\Cell;
use PhpOffice\PhpSpreadsheet\Cell\DataType;
use PhpOffice\PhpSpreadsheet\Cell\DefaultValueBinder;

class ExportExcel extends DefaultValueBinder implements FromArray, WithHeadings, ShouldAutoSize, WithTitle, WithCustomValueBinder
{
    private $setData;
    private $setHeadings;
    private $setSheetTitle;

    public function __construct($data = [], $headings = [], $title = null)
    {
        $this->setData = $data;
        $this->setHeadings = $headings;
        $this->setSheetTitle = $title;
    }

    public function array(): array
    {
        return $this->setData;
    }

    public function headings(): array
    {
        return $this->setHeadings;
    }

    public function title(): string
    {
        return is_null($this->setSheetTitle) ? 'Main' : $this->setSheetTitle;
    }
    public function bindValue(Cell $cell, $value)
    {
        // Fix the bug: is_numeric($value) == 'id' should be is_numeric($value) && $cell->getColumn() == 'A'
        if (is_numeric($value) && $cell->getColumn() == 'A') {
            $cell->setValueExplicit($value, DataType::TYPE_STRING);
            return true;
        }

        // else return default behavior
        return parent::bindValue($cell, $value);
    }
}
