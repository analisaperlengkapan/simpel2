<?php

namespace App\Http\Controllers\Aset;
use Illuminate\Routing\Controller;

use App\Helpers\MyHelper;
use App\Models\Aset\Wujud;
use App\Models\Master\Master;
use App\Models\Dashboard\Dashboard;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Symfony\Component\HttpKernel\Exception\NotFoundHttpException;
use Illuminate\Support\Arr;

class SebaranController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    protected $breadcums = ['Asset', 'Pemetaan Sebaran Asset'];
    public function index()
    {
        $satkers = DB::table('ms_satker as a')->where('a.inst_satkerinduk', '=', '00')->where('a.inst_satkerkd', '!=', '00')->get()->toArray();
        $gps = DB::table('ms_satker as a')->where('a.inst_satkerinduk', '=', '00')->get();
        $arrkordinat = array();
        $idx = 0;
        foreach($gps as $k){
            $arrkordinat[$idx]['kdsatker'] = $k->inst_satkerkd;
            $arrkordinat[$idx]['long'] = $k->long;
            $arrkordinat[$idx]['lat'] = $k->lat;
            $idx++;
        }

        // echo "<pre>"; print_r($arrkordinat);exit;

        $data = [
            'satkerOptions' => MyHelper::generateSelectOptions([
                'data' => $satkers,
                'text' => 'inst_nama',
                //'textkode' => 'kdsatker_keu',
                'value' => 'inst_satkerkd',
                'selected' => null,
            ]),
            'breadcums' => $this->breadcums,
            'tableId' => 'dt-wujud',
            // 'koordinat' => json_encode($arrkordinat)
            'koordinat' => $arrkordinat,
            'gps' => $gps
        ];

        return view('asset.sebaran.sebaranV', $data);
    }

    public function detailSatker(Request $request){
        $kdsatker = $request->input('kdsatker');
        $dashboard = new Dashboard();
        $dataAset = $dashboard->getdata('statistik_bmn','',$kdsatker);
        $satker = DB::table('ms_satker as a')->where('a.inst_satkerkd', '=', $kdsatker)->first();

        //print_r($dataAset);exit;
        if(!empty($dataAset)){
            $kategori = Arr::pluck($dataAset, 'kategori');
            $totalKategori = Arr::pluck($dataAset, 'total');
        }
        $data = [
            'statistik' => $dataAset,
            'kategori' => $kategori,
            'totalKategori' => $totalKategori,
            'longitude' => (float)$satker->long,
            'latitude' => (float)$satker->lat,
            'satker' => $satker,
        ];

        return view('asset.sebaran.detailsebaranV', $data);
    }

    public function getSatkerKoordinat(Request $request){
        $kdsatker_induk = $request->input('kdsatker');
        $satkerinduk = DB::table('ms_satker as a')->where('a.inst_satkerkd', '=', $kdsatker_induk)->first();
        $satkerkejari = DB::table('ms_satker as a')->where('a.inst_satkerinduk', '=', $kdsatker_induk)->get();
        $arrkordinat = array();
        $idx = 0;
        foreach($satkerkejari as $k){
            $arrkordinat[$idx]['kdsatker'] = $k->inst_satkerkd;
            $arrkordinat[$idx]['long'] = (float)$k->long;
            $arrkordinat[$idx]['lat'] = (float)$k->lat;
            $arrkordinat[$idx]['nama'] = $k->inst_nama;
            $idx++;
        }

        $data = [
            'longitude' => (float)$satkerinduk->long,
            'latitude' => (float)$satkerinduk->lat,
            'kejari' => $arrkordinat
        ];

        return response()->json($data);
    }

}