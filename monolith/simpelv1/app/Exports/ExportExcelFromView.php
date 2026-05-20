<?php

namespace App\Exports;

use Illuminate\Contracts\View\View;
use Maatwebsite\Excel\Concerns\FromView;

class ExportExcelFromView implements FromView
{
    private $setData;

    private $setHeadings;

    private $setSheetTitle;

    private $setView;

    public function __construct($data, $view)
    {
        $this->setData = $data;
        $this->setView = $view;
        // $this->setHeadings = $headings;
        // $this->setSheetTitle = $title;
    }

    // public function array(): array
    // {
    //     return $this->setData;
    // }

    // public function headings(): array
    // {
    //     return $this->setHeadings;
    // }

    // public function title(): string
    // {
    //     return is_null($this->setSheetTitle) ? 'Main' : $this->setSheetTitle;
    // }
    public function view(): View
    {
        return view($this->setView, $this->setData);
    }
}
