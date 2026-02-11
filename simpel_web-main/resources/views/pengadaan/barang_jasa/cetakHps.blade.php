<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>HPS</title>
    <style>
        html {
            font-family: 'Calibri', sans-serif;
            font-size: 12px;
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
    <div style="text-align: center;font-size:12px"><b>HARGA PERKIRAAN SENDIRI (HPS)</b></div>
    <br/>
    <table width="100%">
        <tr>
            <td width="20%" style="font-size:12px">Nomor HPS</td>
            <td width="5px" style="font-size:12px">:</td>
            <td style="font-size:12px">{{$hps['no_hps']}}</td>
        </tr>
        <tr>
            <td width="20%" style="font-size:12px">Tanggal</td>
            <td style="font-size:12px">:</td>
            <td style="font-size:12px">{{$hps['tgl_hps']}}</td>
        </tr>
        <tr>
            <td width="20%" style="font-size:12px">Jenis Pekerjaan</td>
            <td width="5px" style="font-size:12px">:</td>
            <td style="font-size:12px">{{$model['nama_pengadaan']}}</td>
        </tr>
    </table>
    <br/>
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
    <div style="font-size:12px">Keterangan :</div>
    <div style="font-size:12px">{!! $hps['keterangan'] !!}</div>
    <br/><br/><br/>
    <table  width="100%">
        <tr>
            <td width="50%"></td>
            <td width="50%" style="font-size:12px;text-align: center;">
            {{$hps['nama_penandatangan']}}
            </td>
        </tr>
        <tr>
            <td width="50%"></td>
            <td width="50%" style="font-size:12px;text-align: center;text-decoration:overline">
            {{$hps['pangkat_penandatangan'].' NIP .'.$hps['nip_penandatangan']}}
            </td>
        </tr>
    </table>
</body>
</html>
