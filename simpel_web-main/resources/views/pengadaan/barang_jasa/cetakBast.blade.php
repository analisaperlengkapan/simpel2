<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BAST</title>
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
    <div style="text-align: center;font-size:18px"><b>KEJAKSAAN REPUBLIK INDONESIA</b></div>
    <div style="text-align: center;font-size:18px"><b>KEJAKSAAN AGUNG</b></div>
    <div style="text-align: center;font-size:10px">JL. Sultan Hasanuddin Nomor 1, Kebayoran Baru - Jakarta Selatan<</div>
    <div style="text-align: center;font-size:10px">Telp. (021) 7247844, fax. (021) 7247844, www.kejaksaan.go.id</div>
    <hr size="14px"/>
    <br/>
    <div style="text-align: center;font-size:12px"><b><u>BERITA ACARA SERAH TERIMA BARANG</u></b></div>
    <div style="text-align: center;font-size:12px">Nomor : {{$bast['no_bast']}}</div>
    <br/>
    <div style="text-align: justify;font-size:12px">Pada hari ini {{ $bast['tgl_bast'] }}. Kami yang bertandatangan dibawah ini:</div>
    <table>
        <tr>
            <td width="5%" style="text-align: justify;font-size:12px">1.</td>
            <td width="35%" style="text-align: justify;font-size:12px">Nama</td>
            <td width="2%" style="text-align: justify;font-size:12px">:</td>
            <td style="text-align: justify;font-size:12px">{{$skppbj['nama_penandatangan']}}</td>
        </tr>
        <tr>
            <td></td>
            <td>Pangkat/Nip</td>
            <td>:</td>
            <td style="text-align: justify;font-size:12px">{{$skppbj['pangkat_penandatangan']}} NIP. {{$skppbj['nip_penandatangan']}}
            </td>
        </tr>
        <tr>
            <td></td>
            <td>Tugas</td>
            <td>:</td>
            <td>Pejabat Pembuat Komitmen (PPK)</td>
        </tr>
        <tr>
            <td>2.</td>
            <td>Nama</td>
            <td>:</td>
            <td>{{$spk['nama_penyedia']}}</td>
        </tr>
        <tr>
            <td></td>
            <td>Pekerjaan</td>
            <td>:</td>
            <td>Direktur {{$pemenang}}</td>
        </tr>
        <tr>
            <td></td>
            <td>Alamat</td>
            <td>:</td>
            <td>{{$ringkasan['alamat_penyedia']}}</td>
        </tr>
    </table>
    <div style="text-align: justify;font-size:12px">Berdasarkan {{$model['nama_pengadaan']}} dengan Surat Perintah Kerja ( SPK ) Nomor {{$spk['no_spk']}} Tanggal {{$spk['tgl_spk']}} telah menyerahterimakan barang-barang
    sesuai dengan jumlah atau jenis seperti tercantum di dalam surat pesanan tersebut di atas dalam keadaan baik dan cukup. </div>
    <div style="text-align: justify;font-size:12px">Demikian Berita Acara Serah Terima Barang ini dibuat dengan sebenarnya dan ditandatangani atas persetujuan bersama tentang kebenarannya</div>
    <br>
    <div style="text-align: right;font-size:12px">Jakarta, {{$bast['tgl_bast']}} </div>
    <table  width="100%">
        <tr>
            <td width="50%" style="border: 0px;border-collapse: collapse;text-align: center;font-size:12px">
                Yang Menyerahkan,<br/>
                {{$pemenang}}<br/><br/><br/><br/><br/>
                <u>{{$spk['nama_penyedia']}}</u><br/>
                <span style="font-size:12px">
                Direktur
                </span>
            </td>
            <td width="50%" style="border: 0px;border-collapse: collapse;text-align: center;font-size:12px">
                Yang Menerima<br/><br/><br/><br/><br/>
                {{$skppbj['nama_penandatangan']}}<br/>
                <span style="text-decoration:overline">
                {{'NIP. '.$skppbj['nip_penandatangan']}}
                </span>
            </td>

        </tr>
    </table>
    <div style="text-align: center;font-size:12px">Mengetahui,</div>
    <div style="text-align: center;font-size:12px">BENDAHARA BARANG</div><br><br><br><br>
    <div style="text-align: center;font-size:12px"> {{$bast['nama_pejabat']}} </div>
    <span style="text-decoration:overline">
    <div style="text-align: center;font-size:12px"><span style="text-decoration:overline"> {{$bast['pangkat_pejabat']}} NIP. {{$bast['nip_pejabat']}} </span></div>
</body>
</html>
