<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>SKPPBJ</title>
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
    <div style="text-align: center;font-size:12px"><b><u>SURAT KEPUTUSAN PENETAPAN PENYEDIA BARANG/JASA (SKPPBJ)</u></b></div>
    <div style="text-align: center;font-size:12px"><b>(PENGADAAN LANGSUNG)</b></div>
    <br/><br/>
    <div style="text-align: center;font-size:12px">Yang bertanda tangan dibawah ini :</div>
    <br/>
    <table width="100%">
        <tr>
            <td width="10%" style="font-size:12px"></td>
            <td width="20%" style="font-size:12px">Nama</td>
            <td width="5px" style="font-size:12px">:</td>
            <td style="font-size:12px">{{$skppbj['nama_penandatangan']}}</td>
        </tr>
        <tr>
            <td width="10%" style="font-size:12px"></td>
            <td width="20%" style="font-size:12px">Pangkat</td>
            <td style="font-size:12px">:</td>
            <td style="font-size:12px">{{$skppbj['pangkat_penandatangan'].' NIP. '.$skppbj['nip_penandatangan']}}</td>
        </tr>
        <tr>
            <td width="10%" style="font-size:12px"></td>
            <td width="20%" style="font-size:12px">Jabatan</td>
            <td width="5px" style="font-size:12px">:</td>
            <td style="font-size:12px">{{$skppbj['jabatan_penandatangan']}}</td>
        </tr>
        <tr>
            <td width="10%" style="font-size:12px"></td>
            <td width="20%" style="font-size:12px">Alamat</td>
            <td width="5px" style="font-size:12px">:</td>
            <td style="font-size:12px">{{$skppbj['alamat']}}</td>
        </tr>
    </table>
    <br/>
    <div style="font-size:12px;text-align: justify;">
        Berdasarkan Peraturan Presiden Republik Indonesia Nomor 12 Tahun 2021 Tentang Perubahan atas Peraturan Presiden Nomor 16 Tahun 2018
        Tentang Pengadaan Barang / Jasa Pemerintah. Menetapkan dan mengesahkan hasil Pengadaan Panitia / Pejabat Pengadaan sesuai kewenangannya,
        setelah memeriksa penawarannya yang telah diajukan :
    </div>
    <br/>
    <table width="100%">
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
        <tr>
            <td width="10%" style="font-size:12px;text-align: right;"><b>{{$key+1}}.</b></td>
            <td width="30%" style="font-size:12px"><b>{{$value['penyedia_skppbj']}}</b></td>
            <td width="10%" style="font-size:12px"><b>Rp.</b></td>
            <td style="font-size:12px"><b>{{number_format($value['nilai_skppbj'],0,'','.')}}</b>,- (terlampir)</td>
        </tr>
        @endforeach
        @endif
    </table>
    <br/>
    <div style="font-size:12px;text-align: justify;">
        Maka dengan pertimbangan berdasarkan harga yang paling rendah dan menguntungkan Negara, kami menunjuk : <b>{{$pemenang}}</b>
        di Jakarta sebagai Penawaran yang terendah dan oleh karena itu penawaran tersebut dapat diterima dan dipertanggungjawabkan.
    </div>
    <br/>
    <div style="font-size:12px;text-align: justify;">
        Demikian Surat Keputusan Penetapan Penyedia / Barang Jasa (SKPPBJ) ini kami buat dengan sebenarnya untuk dipergunakan sebaimana mestinya.
    </div>
    <br/>
    <table width="100%">
        <tr>
            <td width="60%"></td>
            <td width="40%" style="font-size:12px;text-align: center;"><b>
                Jakarta, {{$skppbj['tgl_skppbj']}} <br/>
                PEJABAT PEMBUAT KOMITMEN<br/><br/><br/><br/><br/><br/>
                {{$skppbj['nama_penandatangan']}}<br/>
                <span style="text-decoration:overline">
                {{$skppbj['pangkat_penandatangan'].' NIP. '.$skppbj['nip_penandatangan']}}
                </span>
            </b></td>
        </tr>
    </table>
</body>
</html>
