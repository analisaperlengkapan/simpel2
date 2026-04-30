<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RINGKASAN KONTRAK</title>
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
    <div style="text-align: center;font-size:20px"><b>JAKSA AGUNG MUDA BIDANG PEMBINAAN</b></div>
    <hr size="14px"/>
    <br/><br/>
    <div style="text-align: center;font-size:12px"> <u>RINGKASAN KONTRAK</u> </div>
    <br/><br/>
    <div style="text-align: left;font-size:12px">Untuk kegiatan yang dananya berasal dari Rupiah Murni :</div>
    <table>
        <tr>
            <td width="5%">1.</td>
            <td width="35%">Nomor dan Tanggal DIPA</td>
            <td width="3%">:</td>
            <td>{{$ringkasan['no_dipa']}} <br/>Tanggal {{$ringkasan['tgl_dipa']}}</td>
        </tr>
        <tr>
            <td>2.</td>
            <td>Kode Kegiatan/Output/Akun</td>
            <td>:</td>
            <td>{{$model['kode_anggaran']}}</td>
        </tr>
        <tr>
            <td>3.</td>
            <td>Nomor dan Tanggal SPK/Kontrak</td>
            <td>:</td>
            <td>{{$spk['no_spk']}} <br/>Tanggal {{$spk['tgl_spk']}}</td>
        </tr>
        <tr>
            <td>4.</td>
            <td>Nama Kontraktor/Perusahaan</td>
            <td>:</td>
            <td>{{$pemenang}}</td>
        </tr>
        <tr>
            <td>5.</td>
            <td>Alamat Kantor</td>
            <td>:</td>
            <td>{{$ringkasan['alamat_penyedia']}}</td>
        </tr>
        <tr>
            <td>6.</td>
            <td>Nilai SPK/Kontrak</td>
            <td>:</td>
            <td>Rp.  {{ number_format($nilai_spk,0,'','.') }}</td>
        </tr>
        <tr>
            <td>7.</td>
            <td>Uraian dan Volume Pekerjaan</td>
            <td>:</td>
            <td>{{$model['nama_pengadaan']}}</td>
        </tr>
        <tr>
            <td>8.</td>
            <td>Cara Pembayaran</td>
            <td>:</td>
            <td>{{$ringkasan['cara_pembayaran']}} <br/> {{$pemenang}} <br/> {{$ringkasan['alamat_penyedia']}} <br/>
            {{$ringkasan['nama_bank']}} <br/> {{$ringkasan['kantor_bank']}} <br/> Rekening : {{$ringkasan['no_rek']}} <br/> NPWP : {{$ringkasan['npwp']}}</td>
        </tr>
        <tr>
            <td>9.</td>
            <td>Jangka Waktu Pelaksanaan</td>
            <td>:</td>
            <td>Terhitung Sejak Tanggal {{$spk['tgl_mulai']}} s/d {{$spk['tgl_selesai']}}</td>
        </tr>
        <tr>
            <td>10.</td>
            <td>Tanggal Penyelesaian Pekerjaan</td>
            <td>:</td>
            <td>{{$spk['tgl_selesai']}}</td>
        </tr>
        <tr>
            <td>11.</td>
            <td>Jangka Waktu Pemeliharaan</td>
            <td>:</td>
            <td>-</td>
        </tr>
        <tr>
            <td>12.</td>
            <td>Ketentuan Sanksi</td>
            <td>:</td>
            <td>{{$ringkasan['sanksi']}}</td>
        </tr>
        <tr><td colspan="4"></td></tr>
        <tr><td colspan="4"></td></tr>
        <tr><td colspan="4"></td></tr>
        <tr><td colspan="4"></td></tr>
        <tr><td colspan="4"></td></tr>
        <tr>
            <td colspan="2"></td>
            <td colspan="2" style="text-align: center;">
                Jakarta,<br/>
                An. KUASA PENGGUNA ANGGARAN<br/>
                PEJABAT PEMBUAT KOMITMEN<br/><br/><br/><br/><br/><br/>
               <b><u>{{$skppbj['nama_penandatangan']}}</u></b> <br/>
               {{' NIP. '.$skppbj['nip_penandatangan']}}
            </td>
        </tr>
    </table>
</body>
</html>
