<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Nodis</title>
    <style>
        html {
            font-family: 'Calibri', sans-serif;
            font-size: 12px;
        }
        td {
            font-size:12px;
        }
    </style>
</head>
<body>
    <div style="text-align: center;font-size:18px"><b>KEJAKSAAN AGUNG</b></div>
    <div style="text-align: center;font-size:18px"><b>JAKSA AGUNG MUDA BIDANG PEMBINAAN</b></div>
    <hr size="14px"/>
    <br/>
    <div style="text-align: center;font-size:12px">Nota Dinas</div>
    <div style="text-align: center;font-size:12px">NOMOR : {{$nodis['no_nodis']}}</div>
    <br/>
    <table>
        <tr>
            <td width="35%" style="text-align: justify;font-size:12px;vertical-align:top">Yth.</td>
            <td width="2%" style="text-align: justify;font-size:12px;vertical-align:top">:</td>
            <td style="text-align: justify;font-size:12px">{{$nodis['yth']}} <br> Selaku Kuasa Pengguna Anggaran</td>
        </tr>
        <tr>
            <td width="35%" style="text-align: justify;font-size:12px">Dari</td>
            <td width="2%" style="text-align: justify;font-size:12px">:</td>
            <td style="text-align: justify;font-size:12px">{{$nodis['dari']}}</td>
        </tr>
        <tr>
            <td width="35%" style="text-align: justify;font-size:12px">Tanggal</td>
            <td width="2%" style="text-align: justify;font-size:12px">:</td>
            <td style="text-align: justify;font-size:12px">{{$nodis['tgl_nodis']}}</td>
        </tr>
        <tr>
            <td width="35%" style="text-align: justify;font-size:12px">Sifat</td>
            <td width="2%" style="text-align: justify;font-size:12px">:</td>
            <td style="text-align: justify;font-size:12px">{{$nodis['sifat']}}</td>
        </tr>
        <tr>
            <td width="35%" style="text-align: justify;font-size:12px">Lampiran</td>
            <td width="2%" style="text-align: justify;font-size:12px">:</td>
            <td style="text-align: justify;font-size:12px">{{$nodis['lampiran']}}</td>
        </tr>
        <tr>
            <td width="35%" style="text-align: justify;font-size:12px">Hal</td>
            <td width="2%" style="text-align: justify;font-size:12px">:</td>
            <td style="text-align: justify;font-size:12px">{{$nodis['hal']}}</td>
        </tr>
    </table>
    <div style="text-align: justify;font-size:12px">Bersama ini disampaikan :</div>
    <table>
        <tr>
            <td width="35%" style="text-align: justify;font-size:12px">Kuitansi</td>
            <td width="2%" style="text-align: justify;font-size:12px">:</td>
            <td style="text-align: justify;font-size:12px">{{$spk['nama_penyedia']}}</td>
        </tr>
        <tr>
            <td width="35%" style="text-align: justify;font-size:12px">Tanggal</td>
            <td width="2%" style="text-align: justify;font-size:12px">:</td>
            <td style="text-align: justify;font-size:12px">{{$nodis['tgl_nodis']}}</td>
        </tr>
        <tr>
            <td width="35%" style="text-align: justify;font-size:12px;">Tagihan Sebesar</td>
            <td width="2%" style="text-align: justify;font-size:12px">:</td>
            <td style="text-align: justify;font-size:12px">Rp. {{ number_format($nilai_spk,0,'','.')}}</td>
        </tr>
        <tr>
            <td width="35%" style="text-align: justify;font-size:12px;vertical-align:top">Untuk Pembayaran</td>
            <td width="2%" style="text-align: justify;font-size:12px;vertical-align:top">:</td>
            <td style="text-align: justify;font-size:12px">{{ $model['nama_pengadaan'] }} berdasarkan <br>
            SURAT PERINTAH KERJA (SPK) <br>
            NOMOR : {{$spk['no_spk']}} <br>
            TANGGAl : {{$spk['tgl_spk']}} <br>
            BERITA ACARA SERAH TERIMA BARANG <br>
            NOMOR : {{$bast['no_bast']}} <br> TANGGAL : {{ $bast->tgl_bast_normal}} </td>
        </tr>
    </table>
    <div style="text-align: justify;font-size:12px">Selanjutnya untuk diproses lebih lanjut sesuai ketantuan yang berlaku, atas perhatiannya diucapkan terima kasih.</div>
    <br/><br/>
    <table  width="100%">
        <tr>
            <td width="50%" style="border: 0px;border-collapse: collapse;text-align: center;font-size:12px">
                <br/><br/><br/><br/><br/>
                <u></u><br/>
            </td>
            <td width="50%" style="border: 0px;border-collapse: collapse;text-align: center;font-size:12px">
                {{$nodis['jabatan_pejabat']}},<br/><br/><br/><br/><br/>
                {{$nodis['nama_pejabat']}}<br/>
                <span style="text-decoration:overline">
                {{$nodis['pangkat_pejabat'].'NIP. '.$nodis['nip_pejabat']}}
                </span>
            </td>

        </tr>
    </table>
</body>
</html>
