<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>SPK</title>
    <style>
        html {
            font-family: 'Calibri', sans-serif;
            font-size: 12px;
        }
        #tb-utama, #tb-utama tr, #tb-utama td {
            border: 1px solid black;
            border-collapse: collapse;
        }
        #tb-barang, #tb-barang th, #tb-barang td {
            border: 1px solid black;
            border-collapse: collapse;
        }
    </style>
</head>
<body>
    <div style="text-align: center;font-size:18px"><b>KEJAKSAAN AGUNG</b></div>
    <div style="text-align: center;font-size:20px"><b>JAKSA AGUNG MUDA BIDANG PEMBINAAN</b></div>
    <hr size="14px"/>
    <br/><br/>
    <table id="tb-utama" width="100%">
        <tr>
            <td width="50%" style="text-align: center;font-size:12px">
                <b><u>SURAT PERINTAH KERJA</u></b><br/>(SPK)
            </td>
            <td width="50%" style="text-align: left;font-size:12px">
                Satuan Kerja PPK :<br/>
                BIRO PERLENGKAPAN PEMBINAAN<br/>
                <table width="100%">
                    <tr>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px;width:80px">Nomor SPK</td>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px;width:10px">:</td>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px">{{$spk['no_spk']}}</td>
                    </tr>
                    <tr>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px">Tanggal</td>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px">:</td>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px">{{$spk['tgl_spk']}}</td>
                    </tr>
                </table>
            </td>
        </tr>
        <tr>
            <td  rowspan="2" width="50%" style="text-align: left;font-size:12px">
                <b><u>PAKET PEKERJAAN :</u></b><br/>
                {{$model['nama_pengadaan']}}
            </td>
            <td width="50%" style="text-align: left;font-size:12px">
                Nomor dan Tanggal Surat Permintaan Penawaran<br/>
                <table width="100%">
                    <tr>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px;width:80px">Nomor</td>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px;width:10px">:</td>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px">{{$spk['no_permintaan']}}</td>
                    </tr>
                    <tr>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px">Tanggal</td>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px">:</td>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px">{{$spk['tgl_permintaan']}}</td>
                    </tr>
                </table>
            </td>
        </tr>
        <tr>
            <td width="50%" style="text-align: left;font-size:12px">
                Nomor dan Tanggal Berita Acara Hasil Negosiasi<br/>
                <table width="100%">
                    <tr>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px;width:80px">Nomor</td>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px;width:10px">:</td>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px">{{$spk['no_ba']}}</td>
                    </tr>
                    <tr>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px">Tanggal</td>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px">:</td>
                        <td style="border: 0px;border-collapse: collapse;text-align: left;font-size:12px">{{$spk['tgl_ba']}}</td>
                    </tr>
                </table>
            </td>
        </tr>
        <tr>
            <td colspan="2" width="50%" style="text-align: left;font-size:12px">
                <b><u>SUMBER DANA :</u></b><br/>
                @php
                    $year = date('Y', strtotime($spk['tgl_spk']));
                @endphp
                Dibebankan atas DIPA Kejaksaan Agung R.I Tahun Anggaran {{$year}}.
            </td>
        </tr>
        <tr>
            <td colspan="2" width="50%" style="text-align: left;font-size:12px">
                <b>WAKTU PELAKSANAAN PEKERJAAN :</b>
                @php
                    $differenceInDays = \Carbon\Carbon::parse(strtotime($spk['tgl_mulai']))->diffInDays(\Carbon\Carbon::parse(strtotime($spk['tgl_selesai'])));
                @endphp
                {{$spk['tgl_mulai']}} s/d {{$spk['tgl_selesai']}}. {{$differenceInDays}} ({{ Riskihajar\Terbilang\Facades\Terbilang::make($differenceInDays)}}) hari kerja
            </td>
        </tr>
        <tr>
            <td colspan="2" width="50%" style="text-align: center;font-size:12px">
                <b>NILAI PEKERJAAN</b>
        </tr>
        <tr>
            <td colspan="2">
            <table width="100%" id="tb-barang">
                <thead>
                    <tr>
                        <th rowspan="2" style="text-align: center;font-size:12px">No.</th>
                        <th rowspan="2" style="text-align: center;font-size:12px" width="30%">Uraian Pekerjaan</th>
                        <th colspan="2" style="text-align: center;font-size:12px">Volume</th>
                        <th rowspan="2" style="text-align: center;font-size:12px" width="20%">Harga Satuan (Rp)</th>
                        <th rowspan="2" style="text-align: center;font-size:12px" width="20%">Jumlah Harga (Rp)</th>
                    </tr>
                    <tr>
                        <th style="text-align: center;font-size:12px" width="10%">qts</th>
                        <th style="text-align: center;font-size:12px" width="10%">Satuan</th>
                    </tr>
                </thead>
                <tbody>
                @php
                $decodedData = json_decode($hps['barang'], true);
                $subtotal = 0;
                Config::set('terbilang.locale', 'id');
                @endphp
                @if(!empty($decodedData))
                @foreach($decodedData as $key => $value)
                    @php
                        $subtotal += $value['harga_total']; // Accumulating the subtotal
                    @endphp
                <tr>
                    <td style="text-align: center;font-size:12px">{{$key+1}}</td>
                    <td style="font-size:12px">{{$value['uraian_barang']}}</td>
                    <td style="font-size:12px">{{$value['qty_barang']}}</td>
                    <td style="font-size:12px">{{$value['satuan_barang']}}</td>
                    <td style="text-align: right;font-size:12px">{{ number_format($value['harga_satuan'],0,'','.')}}</td>
                    <td style="text-align: right;font-size:12px">{{ number_format($value['harga_total'],0,'','.')}}</td>
                </tr>
                @endforeach
                <tr>
                    <td colspan="5" style="text-align: right;font-size:12px">Subtotal</td>
                    <td style="text-align: right;font-size:12px">{{ number_format($subtotal,0,'','.') }}</td>
                </tr>
                <tr>
                    <td colspan="5" style="text-align: right;font-size:12px">PPN 11%</td>
                    <td style="text-align: right;font-size:12px">{{ number_format(($subtotal*0.11),0,'','.') }}</td>
                </tr>
                <tr>
                    <td colspan="5" style="text-align: right;font-size:12px">Total</td>
                    <td style="text-align: right;font-size:12px">{{ number_format(($subtotal*0.11)+$subtotal,0,'','.') }}</td>
                </tr>
                @endif
                </tbody>
            </table>
            <br/>
            <div style="font-size:12px">Terbilang :<br/>{{ Riskihajar\Terbilang\Facades\Terbilang::make((($subtotal*0.11)+$subtotal))}}</div>
            <br/>
            <div style="font-size:12px">Keterangan :</div>
            <div style="font-size:12px">{!! $spk['keterangan'] !!}</div>
            <br/><br/>
            </td>
        </tr>
        <tr>
            <td colspan="2" width="50%" style="text-align: left;font-size:12px">
                <b><u>INSTRUKSI KEPADA PENYEDIA :</u></b><br/>
                {{$spk['instruksi']}}
            </td>
        </tr>
        <tr>
            <td width="50%" style="border: 0px;border-collapse: collapse;text-align: center;font-size:12px">
                <b>Untuk dan atas nama Kejaksaan Agung R.I</b><br/>
                <b>PEJABAT PEMBUAT KOMITMEN</b><br/><br/><br/><br/><br/>
                {{$skppbj['nama_penandatangan']}}<br/>
                <span style="text-decoration:overline">
                {{$skppbj['pangkat_penandatangan'].' NIP. '.$skppbj['nip_penandatangan']}}
                </span>
            </td>
            <td width="50%" style="border: 0px;border-collapse: collapse;text-align: center;font-size:12px">
                <b>Untuk dan atas nama Penyedia Jasa</b><br/>
                @php
                $decodedData = json_decode($skppbj['penyedia'], true);
                @endphp
                @if(!empty($decodedData))
                @foreach($decodedData as $key => $value)
                @php
                    if($value['pemenang_skppbj'] == 1){
                        $pemenang = $value['penyedia_skppbj'];
                    }
                @endphp
                @endforeach
                @endif
                <b>{{$pemenang}}</b><br/><br/><br/><br/><br/>
                <u>{{$spk['nama_penyedia']}}</u><br/>
                <span style="font-size:12px">
                DIREKTUR
                </span>
            </td>
        </tr>
    </table>
</body>
</html>
