<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>KONTRAK</title>
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
    <br/>
    <div style="text-align: center;font-size:12px">KONTRAK PENGADAAN BARANG/JASA<br>TAHUN ANGGARAN {{$year}} </div>
    <div style="text-align: center;font-size:12px">Nomor : {{$kontrak['no_kontrak']}}</div>
    <br/>
    <div style="text-align: justify;font-size:12px">Pada hari ini {{ $kontrak['tgl_kontrak'] }}, kami yang bertandatangan dibawah ini:</div>
    <table>
        <tr>
            <td width="5%" style="text-align: justify;font-size:12px">I.</td>
            <td width="35%" style="text-align: justify;font-size:12px">{{$skppbj['nama_penandatangan']}}</td>
        </tr>
        <tr>
            <td></td>
            <td style="text-align: justify;font-size:12px">{{$skppbj['pangkat_penandatangan']}} NIP. {{$skppbj['nip_penandatangan']}} Selaku Pembuat Komitmen, dalam hal ini bertindak untuk dan atas nama Biro Perlengkapan Kejaksaan
                Agung Republik Indonesia, Jalan Sultan Hasanuddin No. 1 Kebayoran Baru Jakarta Selatan untuk selanjutnya disebut PIHAK PERTAMA,
            </td>
        </tr>
        <tr>
            <td width="5%" style="text-align: justify;font-size:12px">II.</td>
            <td width="35%" style="text-align: justify;font-size:12px">{{$spk['nama_penyedia']}}</td>
        </tr>
        <tr>
            <td></td>
            <td style="text-align: justify;font-size:12px">Direktur {{$pemenang}} yang beralamat {{$ringkasan['alamat_penyedia']}} NPWP {{$ringkasan['npwp']}} dalam hal ini bertindak untuk dan atas nama Perusahaan tersebut untuk selanjutnya disebut PIHAK KEDUA,</td>
        </tr>
    </table>
    <div style="text-align: justify;font-size:12px">Berdasarkan :</div>
    <table>
        <tr>
            <td width="5%" style="text-align: justify;font-size:12px">1.</td>
            <td style="text-align: justify;font-size:12px">Peraturan Pemerintah Nomor : PP Nomor 50 Tahun 2018 tentang Perubahan atas Peraturan Pemerintah Nomor 45 Tahun 2013 tentang Tata Cara Pelaksanaan APBN;</td>
        </tr>
        <tr>
            <td width="5%" style="text-align: justify;font-size:12px">2.</td>
            <td style="text-align: justify;font-size:12px">Peraturan Presiden Republik Indonesia Nomor 12 Tahun 2021 tentang Perubahan Peraturan Presiden Republik Indonesia Nomor 16 Tahun 2018 tentang Pengadaan Barang/Jasa Pemerintah;</td>
        </tr>
        <tr>
            <td width="5%" style="text-align: justify;font-size:12px">3.</td>
            <td style="text-align: justify;font-size:12px">Daftar Isian Pelaksanaan Anggaran (DIPA) Kejaksaan Agung Republik Indonesia Tahun 2022 Nomor : {{$ringkasan['no_dipa']}} tanggal {{$ringkasan['tgl_dipa']}};</td>
        </tr>
        <tr>
            <td width="5%" style="text-align: justify;font-size:12px">4.</td>
            <td style="text-align: justify;font-size:12px">Surat Penawaran Harga {{$pemenang}} tanggal</td>
        </tr>
        <tr>
            <td width="5%" style="text-align: justify;font-size:12px">5.</td>
            <td style="text-align: justify;font-size:12px">Surat Keputusan Penetapan Penyedia Barang/Jasa {{$pemenang}} tanggal {{$skppbj['tgl_skppbj']}} sebagai Pelaksana Pekerjaan {{$model['nama_pengadaan']}}.</td>
        </tr>
    </table>
    <div style="text-align: justify;font-size:12px">Pihak Pertama dan Kedua telah sepakat membuat perjanjian dalam rangka {{$model['nama_pengadaan']}} yang diuraikan dalam Pasal-Pasal seperti tersebut dibawah ini :</div>
    <div style="text-align: center;font-size:12px">Pasal 1</div>
    <div style="text-align: center;font-size:12px">Macam Pekerjaan</div><br>
    <div style="text-align: justify;font-size:12px">PIHAK PERTAMA dalam kedudukannya seperti tersebut diatas, memberi tugas kepada PIHAK KEDUA, untuk Pekerjaan {{$model['nama_pengadaan']}} yang macam, jenis, spesifikasi serta jumlahnya sebagaimana tercantum dalam Lampiran Kontrak.</div>
    <br>
    <div style="text-align: center;font-size:12px">Pasal 2</div>
    <div style="text-align: center;font-size:12px">Harga Pekerjaan</div><br>
    <div style="text-align: justify;font-size:12px">Harga Pekerjaan {{$model['nama_pengadaan']}} tersebut dalam Pasal 1 adalah sebesar Rp.  {{ number_format($nilai_spk,0,'','.') }} ({{ Riskihajar\Terbilang\Facades\Terbilang::make($nilai_spk)}}) sudah termasuk pajak.</div>
    <br>
    <div style="text-align: center;font-size:12px">Pasal 3</div>
    <div style="text-align: center;font-size:12px">Waktu Penyelesaian</div><br>
    @php
        $differenceInDays = \Carbon\Carbon::parse(strtotime($spk['tgl_mulai']))->diffInDays(\Carbon\Carbon::parse(strtotime($spk['tgl_selesai'])));
    @endphp
    <div style="text-align: justify;font-size:12px">Pekerjaan {{$model['nama_pengadaan']}} tersebut dalam Pasal 1 selama {{$differenceInDays}} hari kerja mulai dari tanggal {{$spk['tgl_mulai']}} sampai dengan {{$spk['tgl_selesai']}}, harus sudah selesai dilaksanakan oleh PIHAK KEDUA dan selanjutnya diserahkan kepada PIHAK PERTAMA, seluruh barang tersebut pada Pasal 1 sudah diterima di Gudang Kejaksaan Agung RI.</div>
    <br>
    <div style="text-align: center;font-size:12px">Pasal 4</div>
    <div style="text-align: center;font-size:12px">Cara Pembayaran</div><br>
    <div style="text-align: justify;font-size:12px">
        Pembayaran akan dilakukan dengan SPP atas Daftar Isian Pelaksana Anggaran (DIPA) Kejaksaan Agung Republik Indonesia Tahun {{$year}}
        Nomor : {{$ringkasan['no_dipa']}} tanggal {{$ringkasan['tgl_dipa']}}, {{$ringkasan['cara_pembayaran']}} setelah pekerjaan selesai yang dinyatakan dengan Berita Acara
        Serah Terima Barang di Gudang Kejaksaan Republik Indonesia, ditransfer melalui rekening {{$pemenang}} Nomor Rekening : {{$ringkasan['no_rek']}} pada {{$ringkasan['nama_bank']}}
        {{$ringkasan['kantor_bank']}}.
    </div>
    <br>
    <div style="text-align: center;font-size:12px">Pasal 5</div>
    <div style="text-align: center;font-size:12px">Syarat Denda</div><br>
    <div style="text-align: justify;font-size:12px">
        Apabila terjadi keterlambatan penyerahan Barang/Pekerjaan sebagaimana tersebut pada Pasal 1, maka PIHAK KEDUA dikenakan Denda Keterlambatan sebesar 1/1000 (satu per seribu)
        dari harga Pekerjaan untuk setiap hari keterlambatan, dengan denda setinggi-tingginya 5% dari seluruh harga Pekerjaan.
    </div>
    <br>
    <div style="text-align: center;font-size:12px">Pasal 6</div>
    <div style="text-align: center;font-size:12px">Force Majeure</div><br>
    <div style="text-align: justify;font-size:12px">
        Yang dimaksud dengan Force Majeure dalam perjanjian ini adalah diluar kemampuan PIHAK KEDUA yang mengakibatkan tertundanya waktu penyerahan yang disebabkan karena bencana alam,
        kebakaran, huru hara dan perubahan Peraturan Pemerintah dibidang moneter yang ditetapkan dan tercantum dalam Peraturan Pemerintah. Apabila dalam melaksanakan Pekerjaan tersebut pada Pasal 1,
        terjadi hal-hal yang diluar kemampuan PIHAK KEDUA dan dapat dianggap sebagai Force Majeure maka PIHAK KEDUA dapat mengajukan/meminta pertimbangan dari PIHAK PERTAMA tentang perpanjangan waktu
        penyerahan apabila terjadi Force Majeure yang berkaitan dengan pelaksanaan anggaran/penyediaan dana untuk {{$spk['nama_penyedia']}} maka PIHAK KEDUA tidak akan mendapat ganti rugi.
    </div>
    <br>
    <div style="text-align: center;font-size:12px">Pasal 7</div>
    <div style="text-align: center;font-size:12px">Penyelesaian</div><br>
    <div style="text-align: justify;font-size:12px">
        Segala kesalahpahaman atau perselisihan yang mungkin timbul, diselesaikan dengan cara musyawarah dan apabila tidak terjadi kata mufakat, maka penyelesaiannya melalui Pnegadilan Negeri Jakarta Selatan.
    </div>
    <br>
    <div style="text-align: center;font-size:12px">Pasal 8</div>
    <div style="text-align: center;font-size:12px">Penutup</div><br>
    <div style="text-align: justify;font-size:12px">
        Surat perjanjian ini dibuat dengan ditandatangani oleh Kedua Belah Pihak pada hari dan tanggal tersebut diatas, asli dan lembar kedua dibubuhi materai secukupnya, dibuat dalam rangkap 3 (tiga) dengan ketentuan
        semuanya mempunyai kekuatan hukum yang sama.
    </div>
    <br>
    <table  width="100%">
        <tr>
            <td width="50%" style="border: 0px;border-collapse: collapse;text-align: center;font-size:12px">
                PIHAK KEDUA<br/>
                {{$pemenang}}<br/><br/><br/><br/><br/>
                <u>{{$spk['nama_penyedia']}}</u><br/>
                <span style="font-size:12px">
                DIREKTUR
                </span>
            </td>
            <td width="50%" style="border: 0px;border-collapse: collapse;text-align: center;font-size:12px">
                PIHAK PERTAMA<br/><br/><br/><br/><br/>
                {{$skppbj['nama_penandatangan']}}<br/>
                <span style="text-decoration:overline">
                {{'NIP. '.$skppbj['nip_penandatangan']}}
                </span>
            </td>

        </tr>
    </table>
</body>
</html>
