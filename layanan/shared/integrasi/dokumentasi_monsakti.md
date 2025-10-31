MonSAKTI
Pembuka
Administrasi
Penganggaran
Pembayaran
Bendahara
Komitmen
Aset
Persediaan
Pelaporan
bukuBesar
neracaSawal
faDetail
Dokumentasi API Aplikasi Monsakti
Untuk Kementerian/Lembaga
Version: 1.4
Update: 5 Mei 2023
Jika anda memiliki pertanyaan atau membutuhkan bantuan, silakan bertanya pada hai DJPb.

Overview
﻿﻿﻿Pertukaran data harus dilakukan oleh unit pengelola TIK Kementerian Negara/Lembaga (PMK SURAT 171/PMK/05/2021 pasal 85 ayat 2). Dalam hal tidak terdapat unit dimaksud, maka unit operasional di Kementerian/Lembaga dapat mengajukan interkoneksi dengan syarat yang sama dengan unit pengelola TIK sebagai berikut:

Memiliki sumber daya yang dibutuhkan untuk menampung data yaitu server database, database administrator (DBA), dan manajemen pengelolaan data
Memiliki komitmen untuk menjaga keamanan data baik dalam menampung maupun menyebarkan data
Membuat surat permintaan API dengan format yang sudah disediakan (contoh surat dapat dilihat pada tautan berikut: Surat Permintaan API dan Surat Pernyataan Keamanan Server Interkoneksi) dengan tujuan Direktorat SITP, DJPB, Kemenkeu yang dapat disampaikan melalui hai.kemenkeu.go.id atau dengan mengirim surat elektronik ke sitp.perbendaharaan@kemenkeu.go.id
Interkoneksi disediakan untuk pengguna yang berhak berdasarkan kelompok modul yang diminta. Kelompok modul yang tersedia saat ini yaitu:

Modul Admin (ADM)
Modul Anggaran (ANG)
Modul Komitmen (KOM)
Modul Pembayaran (PEM)
Modul Bendahara (BEN)
Modul Aset Tetap (AST)
Modul Persediaan (PER)
Modul Pelaporan (GLP)
Kelompok modul dapat diminta sebagian atau semuanya, tergantung kebutuhan data yang akan dipakai di sistem mitra.

Setiap kelompok modul yang diminta akan memiliki tiga bagian yaitu:

API Key Token
API Endpoint
Reset Token
Graphical user interface

Description automatically generated


Token
﻿﻿Pastikan token yang digunakan merupakan token untuk tahun anggaran yang akan diminta, karena masing-masing token memiliki tahun anggarannya sendiri. Otentifikasi yang kami gunakan adalah "Bearer Token".

Token hanya dapat dipakai untuk 1x hit endpoint, sedangkan token berikutnya ada di response data. Jika muncul response “Token Expired” namun pengguna belum/tidak menyimpan token pada response data sebelumnya, maka pengguna dapat menggunakan endpoint resetToken untuk mendapatkan token baru.

Graphical user interface, text, application

Description automatically generated





API Endpoint
API Endpoint merupakan pranala dengan struktur sebagai berikut: https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/variable1/kelompokModul/tipeData/variable2/variable3/variable4

Penjelasan struktur:

host: https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/

variable:

variable1, terdapat dua fungsi untuk variable1 yaitu ‘API’ untuk response data dan ‘resetToken’. Jika token dan endpoint yang dimasukkan benar, variable ‘API’ akan berisi:
Token baru untuk mengeksekusi data berikutnya
Response data,
sedangkan variable ‘resetToken’ akan berisi token baru untuk mengeksekusi data berikutnya.

kelompokModul menggunakan kode 3 digit kelompok modul.
tipeData merupakan nama tipe data yang dapat diminta pada masing-masing kelompok modul, contoh: dataAng.
variable2 merupakan kode KL atau kode unit pengguna, misal: KL015
variable3, variable4 merupakan parameter yang digunakan untuk request lebih detail dari variable2.
Reset Token
Reset token dapat dieksekusi menggunakan:

token yang sudah expired, atau
token yang diserahkan pertama kali ke PIC pada endpoint: https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/KelompokModul/TipeData/Variable2

Ini endpoint reset tokennya per modul:
RESET LINK: https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/ADM/tipedata/KL006

RESET LINK: https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/ANG/tipedata/KL006

RESET LINK: https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/AST/tipedata/KL006

RESET LINK: https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/BEN/tipedata/KL006

RESET LINK: https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/GLP/tipedata/KL006

RESET LINK: https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/KOM/tipedata/KL006

RESET LINK: https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/PEM/tipedata/KL006

RESET LINK: https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/resetToken/PER/tipedata/KL006





Graphical user interface, text, application, email


Description automatically generated


Tipe Data
Apabila mendapat pesan "Anda tidak memiliki akses terhadap informasi ini.", pastikan token yang digunakan sudah sesuai untuk KL dan Modul yang dimintakan pada link API tersebut. Selain itu juga pastikan nama endpoint dan variabel nya sesuai dengan ketentuan termasuk huruf besar dan huruf kecilnya, misalkan dataAng (bukan dataang), refUraian/program (bukan refuraian/Program).



Penjelasan perumpamaan:

xxx = 3 digit kode kementerian
000000 = 6 digit kode satker
Tipedata yang tersedia:
No.	Nama	Modul	Penjelasan
1.
refUraian
Administrasi	Uraian kode program s.d. komponen
2.
refAdmin
Administrasi	Referensi data satker
3.
pejabat
Administrasi	Data pejabat pada satker
4.
refBank
Administrasi	Referensi bank pada supplierBank
5.
refJnsSPP
Administrasi	Referensi jenis SPP
6.
refAset
Administrasi	Referensi pengkodean pada modul Aset dan Persediaan
7.
dataAng
Penganggaran	Data transaksi penganggaran
8.
refSts
Penganggaran	Referensi status history per satker
9.
pendapatan
Penganggaran	Rencana anggaran pendapatan
10.
realisasi
Pembayaran	Realisasi 16 segmen coa
11.
sppHeader
Pembayaran	Data realisasi spp, spm dan sp2d
12.
sppPengeluaran
Pembayaran	Rincian 16 segmen coa pengeluaran dari sppHeader
13.
sppPotongan
Pembayaran	Rincian 16 segmen coa potongan pada sppHeader
14.
penerimaSPM
Pembayaran	Daftar penerima pada sppHeader
15.
kasTunai
Bendahara	Transaksi bendahara dengan jenis pemindahan kas tunai
16.
kasBank
Bendahara	Transaksi bendahara dengan jenis pemindahan non tunai
17.
spby
Bendahara	Transaksi Surat Perintah Bayar
18.
kuitansi
Bendahara	Kuitansi bukti pembayaran yang dilakukan oleh bendahara
19.
drpp
Bendahara	Daftar Rincian Permintaan Pembayaran yang berisi kumpulan kuitansi yang akan di-SPJ-kan
20.
pungutPajak
Bendahara	Pungutan Pajak atas transaksi bendahara
21.
setorPajak
Bendahara	Setoran Pajak atas transaksi bendahara
22.
pnbp
Bendahara	Setoran Penerimaan Negara Bukan Pajak yang dicatat Bendahara
23.
tup
Bendahara	Data pengajuan TUP satker
24.
pengembalian
Bendahara	Pengembalian belanja pada modul bendahara (SSPB)
25.
capaianRO
Komitmen	Data realisasi kinerja satker
26.
kontrakHeader
Komitmen	Data kontrak
27.
kontrakLine
Komitmen	Data kontrak per line
28.
kontrakTermin
Komitmen	Data kontrak per termin
29.
kontrakCOA
Komitmen	Rincian 16 segmen coa pada kontrak termin
30.
BASTKontrakHeader
Komitmen	Data BAST kontraktual
31.
BASTKontrakDetailBarang
Komitmen	Detail barang pada BAST kontraktual
32.
BASTKontrakCOA
Komitmen	Rincian 16 segmen coa pada BAST kontraktual
33.
BASTNonKontrakHeader
Komitmen	Data BAST non kontraktual
34.
BASTNonKontrakDetailBarang
Komitmen	Detail barang pada BAST non kontraktual
35.
BASTNonKontrakCOA
Komitmen	Rincian 16 segmen coa pada BAST non kontraktual
36.
supplierHeader
Komitmen	Data supplier
37.
supplierAddress
Komitmen	Rincian data supplier per address
38.
supplierBank
Komitmen	Rincian data supplier per rekening
39.
asetTrx
Aset	Data transaksi pada seluruh aset
40.
persediaTrx
Persediaan	Data transaksi pada seluruh persediaan
41.
bukuBesar
Pelaporan	Data jurnal transaksi
42.
neracaSawal
Pelaporan	Saldo awal neraca
43.
faDetail
Pelaporan	Data seluruh transaksi pada seluruh modul yang berdampak pada perubahan angka di FA
* = API template masih dalam tahap pengembangan, dapat berubah dan bertambah sewaktu-waktu.
** Modul baru sebagian disediakan, akan kami perbarui

Administrasi
Dokumentasi endpoint kelompok modul Administrasi

refUraian
﻿Keterangan:

Referensi uraian dari kode-kode penganggaran yang ada pada endpoint dataAng.

Masing-masing jenis uraian dapat menerima variabel masing-masing kode yang ingin diambil sebagaimana penjelasan cara pengambilan dibawah. Namun kode tersebut dapat dikosongkan untuk mengambil semua referensi uraian pada jenis uraian yang diminta, hanya saja perlu diingat pada jenis uraian output, suboutput dan komponen data cukup banyak, sehingga apabila data terlalu besar akan mengeluarkan pesan error " Fatal error: Allowed memory size of...".



Jenis-jenis uraian yang dapat diambil:

program, kegiatan, output, suboutput, komponen, akun.



Variabel yang diterima untuk masing-masing level:

program = KODE_PROGRAM

akun = KODE_AKUN

kegiatan = KODE_KEGIATAN

output = KODE_KEGIATAN.KODE_OUTPUT

suboutput = KODE_KEGIATAN.KODE_OUTPUT.KODE_SUBOUTPUT

komponen = KODE_KEGIATAN.KODE_OUTPUT.KODE_SUBOUTPUT.KODE_KOMPONEN



Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/ADM/refUraian /KLxxx/komponen/6619.EBA.960.057

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/ADM/refUraian /KLxxx/program

refAdmin
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Opsional	Kode Satker
Keterangan:

Berisi data referensi satker pada masing-masing kementerian.



Contoh link pengambilan data:


https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/ADM/refAdmin /KLxxx/000000



Elemen data:

COLUMN_NAME	DATA_TYPE
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KODE_UNIT	VARCHAR2(255 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
DESKRIPSI	VARCHAR2(4000 CHAR)
KODE_KAB_KOTA	VARCHAR2(255 CHAR)
URAIAN_KODE_KAB_KOTA	VARCHAR2(4000 CHAR)
KODE_KEWENANGAN	VARCHAR2(255 CHAR)
URAIAN_KEWENANGAN	VARCHAR2(4000 CHAR)
KODE_KPPN	VARCHAR2(255 CHAR)
NAMA_KPPN	VARCHAR2(4000 CHAR)
pejabat
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Opsional	Kode satker
2.	NIP	Opsional	Nomor Induk Pegawai
﻿Keterangan:

Untuk mengambil data pejabat yang telah di input pada menu Pejabat pada aplikasi SAKTI.



Contoh link  pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/ADM/pejabat/KLxxx/000000


Elemen data:
COLUMN_NAME	DATA_TYPE
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KODE_UNIT	VARCHAR2(2 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
NAMA	VARCHAR2(55 CHAR)
NIP	VARCHAR2(18 CHAR)
TELPON	VARCHAR2(15 CHAR)
EMAIL	VARCHAR2(45 CHAR)
JABATAN	VARCHAR2(4000 CHAR)
refBank
Keterangan:

Digunakan untuk penarikan data pada endpoint supplierBank dalam kasus data yang ditarik terlalu besar, silahkan gunakan referensi bank pada endpoint ini untuk memperkecil data yang ditarik pada endpoint supplierBank.



Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/ADM/refBank/KLxxx/

refJnsSPP
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KODE	Opsional	Kode jenis SPP yang ada pada endpoint realisasi kolom KD_JNS_SPP
Keterangan:

Referensi uraian dari kode jenis spp (KD_JNS_SPP) pada endpoint realisasi.



Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/ADM/refJnsSPP/KLxxx/

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/ADM/refJnsSPP/KLxxx/211




Elemen data:

COLUMN_NAME	DATA_TYPE
KODE	VARCHAR2(255 CHAR)
DESKRIPSI	VARCHAR2(4000 CHAR)
refAset
Keterangan:

Referensi uraian dari kode-kode yang ada pada asetTrx.



Jenis-jenis uraian yang dapat diambil:

KDTRX, KDGOL, KDBID, KDKEL, KDSKEL, KDBRG



Masing-masing jenis uraian dapat menerima variabel masing-masing kode yang ingin diambil sebagaimana penjelasan cara pengambilan dibawah. Namun kode tersebut dapat dikosongkan untuk mengambil semua referensi uraian pada jenis uraian yang diminta, hanya saja perlu diingat pada jenis kode barang data cukup banyak, sehingga apabila data terlalu besar akan mengeluarkan pesan error " Fatal error: Allowed memory size of...".



Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/ADM/refAset/KLxxx/KDTRX/100

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/ADM/refAset/KLxxx/KDBRG/3010101001

Penganggaran
Dokumentasi endpoint kelompok modul Penganggaran

dataAng
Variabel yang diterima:

No.	Nama.	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode Satker
2.	KODE_STS_HISTORY
Wajib	Kode status history, referensi ada pada endpoint refSts
Keterangan:

Berisi data transaksi penganggaran dari program sampai detail item berserta informasi RPD bulanan (Halaman III DIPA). Kami memberikan opsi apakah data yang ada pada endpoint ini hanya untuk pagu yang sedang aktif, atau seluruh data pada status history. Sejak API versi 0.8, KDSATKER wajib diisi, dan sejak API versi 1.3, KODE_STS_HISTORY wajb diisi, silahkan untuk melihat status history pada masing-masing satker pada endpoint refSts.



Referensi KODE_STS_HISTORY:

Kode	Jenis Revisi	Keterangan
D00	RKAKL_AWAL	Dikhususkan untuk pagu indikatif. Segala perubahan akan mereplace pada status history yang sama
D01 dan seterusnya	RKAKL_AWAL	Tahap Pagu Alokasi
E00 dan seterusnya	RKAKL_AWAL	Tahap Alokasi Anggaran
B00	DIPA_AWAL
B01 dan seterusnya	DIPA_REVISI
A01 dan seterusnya	USULAN_DIPA_REVISI
C01 dan seterusnya	SATKER_REVISI	Revisi KPA atau Revisi POK
﻿Tips:


Untuk mendapatkan total pagu yang akurat, silahkan menjumlah dengan ketentuan header1 dan header2 = 0. Untuk menyandingkan dengan data pada endpoint realisasi sampai dengan item gunakan kolom CONS_ITEM.



Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/ANG/dataAng/KLxxx/000000/B00





Elemen data:

COLUMN_NAME	DATA_TYPE
KDSATKER	VARCHAR2(255 CHAR)
KODE_PROGRAM	VARCHAR2(255 CHAR)
KODE_KEGIATAN	VARCHAR2(4 CHAR)
KODE_OUTPUT	VARCHAR2(255 CHAR)
KDIB	VARCHAR2(255 CHAR)
VOLUME_OUTPUT	FLOAT
KODE_SUBOUTPUT	VARCHAR2(255 CHAR)
VOLUME_SUBOUTPUT	FLOAT
KODE_KOMPONEN	VARCHAR2(255 CHAR)
KODE_SUBKOMPONEN	VARCHAR2(255 CHAR)
URAIAN_SUBKOMPONEN	VARCHAR2(255 CHAR)
KODE_AKUN	VARCHAR2(255 CHAR)
KODE_JENIS_BEBAN	VARCHAR2(255 CHAR)
KODE_CARA_TARIK	VARCHAR2(255 CHAR)
KODE_JENIS_BANTUAN	VARCHAR2(255 CHAR)
KODE_REGISTER	VARCHAR2(255 CHAR)
HEADER1	NUMBER(1,0)
HEADER2	NUMBER(1,0)
KODE_ITEM	VARCHAR2(255 CHAR)
NOMOR_ITEM	NUMBER(10,0)
CONS_ITEM	NUMBER
URAIAN_ITEM	VARCHAR2(255 CHAR)
SUMBER_DANA	VARCHAR2(255 CHAR)
VOL_KEG_1	FLOAT
SAT_KEG_1	VARCHAR2(255 CHAR)
VOL_KEG_2	FLOAT
SAT_KEG_2	VARCHAR2(255 CHAR)
VOL_KEG_3	FLOAT
SAT_KEG_3	VARCHAR2(255 CHAR)
VOL_KEG_4	FLOAT
SAT_KEG_4	VARCHAR2(255 CHAR)
VOLKEG	FLOAT
SATKEG	VARCHAR2(255 CHAR)
HARGASAT	NUMBER(19,2)
TOTAL	NUMBER(19,2)
KODE_BLOKIR	VARCHAR2(255 CHAR)
NILAI_BLOKIR	NUMBER(19,2)
KODE_STS_HISTORY	VARCHAR2(255 CHAR)
POK_NILAI_1	NUMBER(19,2)
POK_NILAI_2	NUMBER(19,2)
POK_NILAI_3	NUMBER(19,2)
POK_NILAI_4	NUMBER(19,2)
POK_NILAI_5	NUMBER(19,2)
POK_NILAI_6	NUMBER(19,2)
POK_NILAI_7	NUMBER(19,2)
POK_NILAI_8	NUMBER(19,2)
POK_NILAI_9	NUMBER(19,2)
POK_NILAI_10	NUMBER(19,2)
POK_NILAI_11	NUMBER(19,2)
POK_NILAI_12	NUMBER(19,2)
refSts
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Opsional	Kode satker
Keterangan:

Merupakan referensi masing-masing satker memiliki status history apa saja. Dapat digunakan sebagai trigger untuk pengambilan dataAng, sehingga hanya mengambil data terbaru saja tidak perlu memperbarui semua data anggaran. Dapat digunakan untuk crosscheck penjumlahan kolom TOTAL pada dataAng per satker per status history apakah sudah benar atau tidak.

Tips:

Untuk mengambil status history terakhir adalah dengan melakukan mengambil id terbesar dari masing-masing satker dengan KODE_STS_HISTORY B atau C apabila FLAG_UPDATE_COA = 1.

Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/ANG/refSts/KLxxx/000000

pendapatan
﻿﻿Variabel yang diterima:

No.
Nama
Sifat
Keterangan
1.
KDSATKER
Wajib
Kode satker
2.
KODE_STS_HISTORY
Opsional
Kode status history
Keterangan:

Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/ANG/pendapatan/KLxxx/000000/B00

Pembayaran
Dokumentasi endpoint kelompok modul Pembayaran

realisasi
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Opsional	Kode satker
2.	KD_JNS_SPP	Opsional	Kode jenis SPP, referensi ada pada endpoint refJnsSPP
3.	NO_SPP	Opsional	Nomor SPP
Keterangan:

Berisi data realisasi SPP dan yang dipersamakan dengan menggunakan 16 segmen kode coa. Kami menyarankan untuk menggunakan sppHeader dan endpoint turunannya untuk mendapatkan informasi data pada modul pembayaran yang lebih efisien dan lebih lengkap.



Penjelasan terkait 16 segmen coa:

No	Kode segmen	Digit	Pengertian Kode	Penjelasan
1	KDSATKER	6	Kode Satker	Referensi ada pada refAdmin
2	KDKPPN	3	Kode KPPN	Referensi ada pada refAdmin
3	AKUN	6	Kode Akun	Referensi ada pada refUraian/akun
4	BAUNITKODE_PROGRAM	7	Kode Unit Eselon 1 dan Kode Program	Referensi ada pada refAdmin dan refUraian/program
5	KODE_KEGIATANKODE_OUTPUT	7	Kode Kegiatan dan Kode Output/KRO	Referensi ada pada refUraian/kegiatan dan refUraian/output
6	SUMBERDANA	10	Sumber Dana	Referensi pada tabel dibawah
7	REKBANK	5	Kode Bank
8	KODE_KEWENANGAN	1	Kode Kewenangan	Referensi ada pada refAdmin
9	KODE_KAB_KOTA	4	Kode Lokasi	Referensi ada pada refAdmin
10	TIPEANGGARAN	1	Tipe Anggaran	1=RKAKL, 2=DIPA, 3=AFP
11	INTRACO	6	(Belum digunakan)
12	CADANGAN	6	(Belum digunakan)
13	KODE_SUBOUTPUT	3	Kode sub output/RO	Referensi ada pada refUraian/suboutput
14	KODE_KOMPONEN	3	Kode komponen	Referensi ada pada refUraian/komponen
15	KODE_ SUBKOMPONEN	2	Kode sub komponen	Uraian ada pada dataAng, gunakan huruf belakangnya saja
16	KODE_ITEM	6	Kode item	Uraian ada pada dataAng. Kode ini sama dengan CONS_ITEM pada dataAng
Penjelasan Status Data:

No	Deskripsi	Membebani FA
1	Baru	Tidak
2	Cetak SPP	Tidak
3	Batal SPP	Tidak
4	Setuju SPP	Ya
5	ADK SPP	Ya
6	Batal ADK SPP	Ya
7	Upload NTT	Ya
8	Cetak SPM	Ya
9	Batal SPM	Ya
10	Setuju SPM	Ya
11	ADK SPM	Ya
12	Batal ADK SPM	Ya
13	Upload SP2D	Ya
14	Permohonan Pembatalan ADK SPM	Ya
15	SP2D Void	Ya
16	Voided	Tidak
Penjelasan referensi sumber dana:

Kode	Deskripsi	Singkatan
A	RUPIAH MURNI	RM
B	PINJAMAN LUAR NEGERI	PLN
C	RUPIAH MURNI PENDAMPING	RMP
D	PNBP	PNP
E	PINJAMAN DALAM NEGERI	PDN
F	BADAN LAYANAN UMUM	BLU
G	STIMULUS	STM
H	HIBAH DALAM NEGERI	HDN
I	HIBAH LUAR NEGERI	HLN
J	HIBAH LANGSUNG DALAM NEGERI	HLD
K	HIBAH LANGSUNG LUAR NEGERI	HLL
L	HIBAH LANGSUNG BARANG DALAM NEGERI	HLBD
M	HIBAH LANGSUNG BARANG LUAR NEGERI	HLBL
N	HIBAH LANGSUNG JASA DALAM NEGERI	HLJD
O	HIBAH LANGSUNG JASA LUAR NEGERI	HLJL
P	HIBAH LANGSUNG SURAT BERHARGA DALAM NEGERI	HLSD
Q	HIBAH LANGSUNG SURAT BERHARGA LUAR NEGERI	HLSL
R	LUNCURAN	-
S	SALDO AWAL BLU	-
Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/PEM/realisasi /KLxxx/000000/231/00005T



Elemen data:

COLUMN_NAME	DATA_TYPE
KDSATKER	VARCHAR2(6 CHAR)
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KD_JNS_SPP	VARCHAR2(3 CHAR)
NO_SPP	VARCHAR2(6 CHAR)
TGL_SPP	DATE
NO_SPM	VARCHAR2(50 CHAR)
TGL_SPM	DATE
NO_SP2D	VARCHAR2(50 CHAR)
TGL_SP2D	DATE
URAIAN	VARCHAR2(255 CHAR)
KODE_COA	VARCHAR2(100 CHAR)
KODE_PROGRAM	VARCHAR2(2 CHAR)
KODE_KEGIATAN	VARCHAR2(4 CHAR)
KODE_OUTPUT	VARCHAR2(3 CHAR)
KODE_SUBOUTPUT	VARCHAR2(5 CHAR)
KODE_KOMPONEN	VARCHAR2(5 CHAR)
KODE_SUBKOMPONEN	VARCHAR2(5 CHAR)
KODE_AKUN	VARCHAR2(6 CHAR)
KODE_ITEM	VARCHAR2(22 CHAR)
MATA_UANG	VARCHAR2(3 CHAR)
KURS	NUMBER(19,4)
NILAI_VALAS	NUMBER(21,2)
NILAI_RUPIAH	NUMBER(21,2)
STATUS_DATA	VARCHAR2(4000 CHAR)
sppHeader
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Opsional	Kode satker
2.	KD_JNS_SPP	Opsional	Kode jenis SPP, referensi ada pada endpoint refJnsSPP
3.	NO_SPP	Opsional	Nomor SPP
Keterangan:

Berisi data realisasi SPP dan yang dipersamakan dengan menggunakan 16 segmen kode coa. Endpoint ini hanya berisikan 1 row per 1 SPP. Untuk mendapatkan distribusi COA silahkan gunakan sppPengeluaran, sppPotongan dan penerimaSPM dengan melakukan join data menggunakan kolom ID_SPP.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/PEM/sppHeader/KLxxx/000000/231/00005T


Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(6 CHAR)
KD_KPPN	VARCHAR2(3 CHAR)
THN_ANG	VARCHAR2(4 CHAR)
ID_SPP	NUMBER(19,0)	Primary Key
ID_SUPPLIER	NUMBER(19,0)
ID_BAST	VARCHAR2(1000 BYTE)
STS_DATA	VARCHAR2(50 CHAR)	Referensi ada pada endpointrealisasi (4.1)
KD_JNS_SPP	VARCHAR2(3 CHAR)	Referensi ada pada endpointrefJnsSPP
NO_SPP	VARCHAR2(6 CHAR)
NO_SPP2	VARCHAR2(15 CHAR)	Keterangan satker/tahun
TGL_SPP	DATE
ID_SPP_YG_DIKOREKSI	NUMBER	Berisi id spp yang dikoreksi
JNS_SPP_KOREKSI	VARCHAR2(255 CHAR)	Jika SPP Koreksi
TGL_SPP_KOREKSI	TIMESTAMP(6)	Jika SPP Koreksi
TGL_SPM_KOREKSI	TIMESTAMP(6)	Jika SPP Koreksi
TGL_SP2D_KOREKSI	DATE	Jika SPP Koreksi
KOREKSI_FLAG	NUMBER(1,0)	0 atau null /1(yang dikoreksi)
NO_SPM	VARCHAR2(50 CHAR)
TGL_SPM	DATE
TGL_ADK_SPM	DATE
NILAI_SPM	NUMBER(19,0)
NO_SP2D	VARCHAR2(50 CHAR)
TGL_SP2D	DATE
NILAI_SP2D	NUMBER(19,2)
NO_SP2B	VARCHAR2(255 CHAR)	Pengesahan
TGL_SP2B	TIMESTAMP(6)
NILAI_SP2B	NUMBER(19,2)
NO_SP3HL_BJS	VARCHAR2(50 CHAR)
TGL_SP3HL_BJS	DATE
NO_GAJI	VARCHAR2(900 BYTE)
BULAN_GAJI	NUMBER(10,0)
NO_REKSUS	VARCHAR2(50 CHAR)
ID_JADWAL_BYR_KONTRAK	NUMBER(19,0)
ID_KONTRAK	NUMBER(19,0)
NO_KONTRAK	VARCHAR2(255 CHAR)
NILAI_KONTRAK_PDN	NUMBER(19,2)
NILAI_KONTRAK_PDP	NUMBER(19,2)
NILAI_KONTRAK_PLN	NUMBER(19,2)
NO_APLIKASI	VARCHAR2(50 CHAR)	Nomor invoice SPD-PL
TGL_APLIKASI	DATE	Tanggal SPD-PL
NILAI_APLIKASI	NUMBER(19,2)	Nilai
NO_REGISTER	VARCHAR2(50 CHAR)	Nomor register pengesahan
TGL_REGISTER	DATE
NO_PENGESAHAN	VARCHAR2(50 CHAR)	Nomor dokumen pengesahan
TGL_PENGESAHAN	DATE
JML_PENGELUARAN	NUMBER(19,2)	Jumlah kotor SPP/SPM
JML_POTONGAN	NUMBER(19,2)	Jumlah potongan SPP/SPM
JML_PEMBAYARAN	NUMBER(19,2)	Jumlah bersih SPP/SPM
KD_VALAS	VARCHAR2(3 CHAR)	Kode valas(mata uang)
TIPE_KURS	VARCHAR2(1 CHAR)	Referensi (BI tengah, user)
TGL_KURS	DATE
NILAI_TUKAR	NUMBER(19,2)	Nilai tukar spp/spm
NILAI_TUKAR_SP2D	NUMBER(19,4)	Nilai tukar sp2d
NIP_PPK	VARCHAR2(255 CHAR)
NAMA_PPK	VARCHAR2(255 CHAR)
NIP_PPSPM	VARCHAR2(255 CHAR)
NAMA_PPSPM	VARCHAR2(255 CHAR)
URAIAN	VARCHAR2(255 CHAR)
NPWP2	VARCHAR2(50 CHAR)	NPWP bendahara
KODE_SUMBER_DANA	VARCHAR2(1 CHAR)	Referensi ada pada endpointrealisasi (4.1)
NO_NOD	VARCHAR2(255 CHAR)	Pinjaman Luar Negeri
AMOUNT_NOD	NUMBER(19,2)	Pinjaman Luar Negeri
KURS_NOD	NUMBER(19,2)	Pinjaman Luar Negeri
TGL_NOD	TIMESTAMP(6)	Pinjaman Luar Negeri
NO_WA	VARCHAR2(255 CHAR)	Pinjaman Luar Negeri
TGL_WA	TIMESTAMP(6)	Pinjaman Luar Negeri
PEMBAYARAN_PENDAMPING	NUMBER(19,2)	Pinjaman Luar Negeri
PORSI_SETOR_SAAT_INI	NUMBER(19,2)	Pinjaman Luar Negeri
PORSI_TDK_SETOR_SAAT_INI	NUMBER(19,2)	Pinjaman Luar Negeri
STATUS_APD	VARCHAR2(255 CHAR)	pinjaman
NILAI_KONTRAK_APD	NUMBER(19,2)
PERIODE_TRIWULAN	VARCHAR2(3 CHAR)	Triwulan pengesahan
SALDO_AWAL	NUMBER(19,2)	Saldo awal pengesahan
BELANJA	NUMBER(19,2)	Realisasi pengeluaran
PENDAPATAN	NUMBER(19,2)	Realisasi pendapatan
SALDO_AKHIR	NUMBER(19,2)	Saldo akhir pengesahan
sppPengeluaran
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	ID_SPP	Wajib	ID SPP
Keterangan:

Merupakan anak dari sppHeader, di join menggunakan ID_SPP. Berisi data distribusi coa 16 segmen untuk nilai pengeluaran SPP tersebut.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/PEM/sppPengeluaran/KLxxx/0000001


Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(6 CHAR)
ID_SPP	NUMBER(19,0)
KODE_PROGRAM	VARCHAR2(2 CHAR)
KODE_KEGIATAN	VARCHAR2(4 CHAR)
KODE_OUTPUT	VARCHAR2(3 CHAR)
KODE_AKUN	VARCHAR2(6 CHAR)
KODE_SUBOUTPUT	VARCHAR2(5 CHAR)
KODE_KOMPONEN	VARCHAR2(5 CHAR)
KODE_SUBKOMPONEN	VARCHAR2(5 CHAR)
KODE_ITEM	VARCHAR2(22 CHAR)
KD_CTARIK	VARCHAR2(1 CHAR)
KD_REGISTER	VARCHAR2(8 CHAR)
KODE_COA	VARCHAR2(255 CHAR)
KODE_VALAS	VARCHAR2(255 CHAR)
NILAI_AKUN_PENGELUARAN	NUMBER	Nilai realisasi belanja
NILAI_TUKAR	NUMBER(19,2)	Nilai tukar (kurs) saat spp/spm
NILAI_TUKAR_SP2D	NUMBER(19,4)	Nilai tukar (kurs) sp2d
TGL_KUR_SP2D	DATE
NILAI_VALAS	NUMBER	Nilai spp/spm(nilai akun*nilai tukar)
NILAI_PEMBAYARAN_VALAS_SP2D	NUMBER(19,2)	Nilai sp2d(nilai akun*nilai tukar sp2d)
sppPotongan
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	ID_SPP	Wajib	ID SPP
Keterangan:

Merupakan anak dari sppHeader, di join menggunakan ID_SPP. Berisi data distribusi coa 16 segmen untuk nilai pengeluaran SPP tersebut.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/PEM/sppPotongan/KLxxx/0000001


Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(6 CHAR)
ID_SPP	NUMBER(19,0)
KODE_PROGRAM	VARCHAR2(2 CHAR)
KODE_KEGIATAN	VARCHAR2(4 CHAR)
KODE_OUTPUT	VARCHAR2(3 CHAR)
KODE_AKUN	VARCHAR2(6 CHAR)
KODE_SUBOUTPUT	VARCHAR2(5 CHAR)
KODE_KOMPONEN	VARCHAR2(5 CHAR)
KODE_SUBKOMPONEN	VARCHAR2(5 CHAR)
KODE_ITEM	VARCHAR2(22 CHAR)
KD_CTARIK	VARCHAR2(1 CHAR)
KD_REGISTER	VARCHAR2(8 CHAR)
KODE_COA	VARCHAR2(255 CHAR)
KODE_VALAS	VARCHAR2(255 CHAR)
NILAI_AKUN_POT	NUMBER	Nilai potongan
NILAI_TUKAR	NUMBER(19,2)	Nilai tukar (kurs) saat spp/spm
NILAI_TUKAR_SP2D	NUMBER(19,4)	Nilai tukar (kurs) sp2d
NILAI_VALAS	NUMBER	Nilai spp/spm(nilai akun*nilai tukar)
NILAI_PEMBAYARAN_VALAS_SP2D	NUMBER(19,2)	Nilai sp2d(nilai akun*nilai tukar sp2d)
penerimaSPM
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	ID_SPP	Wajib	ID SPP
Keterangan:

Merupakan anak dari sppHeader, di join menggunakan ID_SPP. Berisi data penerima / supplier dari SPP tersebut.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/PEM/penerimaSPM/KLxxx/0000001


Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KDSATKER	VARCHAR2(6 CHAR)
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
ID_SPP	NUMBER(19,0)
ID_SUPPLIER	NUMBER(19,0)
ID_SUPPLIER_ADDRESS	NUMBER(19,0)
ID_SUPPLIER_BANK	NUMBER(19,0)
KODE_TIPE_SUPPLIER	VARCHAR2(255 CHAR)
NAMA_SITE	VARCHAR2(255 CHAR)
NAMA	VARCHAR2(255 CHAR)
ALAMAT	VARCHAR2(255 CHAR)
NAMA_BANK	VARCHAR2(255 CHAR)
NAMA_PEGAWAI	VARCHAR2(255 CHAR)
NILAI	NUMBER(19,2)
NPWP	VARCHAR2(255 CHAR)
NRS	VARCHAR2(255 CHAR)	Nomor register supplier
Bendahara
Dokumentasi endpoint kelompok modul Bendahara

kasTunai
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
Keterangan:

Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/BEN/kasTunai/KLxxx/000000



Elemen data:

COLUMN_NAME	DATA_TYPE
ID	NUMBER(19,0)
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
KODE_UNIT_TEKNIS	VARCHAR2(10 CHAR)
THANG	VARCHAR2(255 CHAR)
KATEGORI_KAS	VARCHAR2(3 CHAR)
JUMLAH	NUMBER(19,2)
NO_REFERENSI	VARCHAR2(50 CHAR)
TGL_TRANSAKSI	DATE
KODE_SUMBER_DANA	VARCHAR2(1 CHAR)
URAIAN	VARCHAR2(2000 CHAR)
kasBank
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
Keterangan:

Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/BEN/kasBank/KLxxx/000000


Elemen data:

COLUMN_NAME	DATA_TYPE
ID	NUMBER(19,0)
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
KODE_UNIT_TEKNIS	VARCHAR2(10 CHAR)
KATEGORI_KAS	VARCHAR2(3 CHAR)
JUMLAH	NUMBER(19,2)
NO_REFERENSI	VARCHAR2(50 CHAR)
TGL_TRANSAKSI	DATE
URAIAN	VARCHAR2(2000 CHAR)
KODE_SUMBER_DANA	VARCHAR2(1 CHAR)
THANG	VARCHAR2(255 CHAR)
ID_REK_BANK	NUMBER(19,0)
NAMA_BANK	VARCHAR2(100 CHAR)
spby
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
Keterangan:
Data ini berisi seluruh transaksi Surat Perintah Bayar (SPBy) yang telah diinput pada Aplikasi SAKTI.﻿

﻿
Contoh link pengambilan data:﻿

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/BEN/spby/KLxxx/000000

﻿
Elemen data:

COLUMN_NAME	DATA_TYPE
ID	NUMBER(19,0)
ID_BAST	VARCHAR2(255 CHAR)
WAJIB_PAJAK_ID	NUMBER(19,0)
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
THANG	VARCHAR2(255 CHAR)
JENIS_PERINTAH_BAYAR	VARCHAR2(1 CHAR)
NO_PERINTAH_BAYAR	VARCHAR2(50 CHAR)
REF_KWITANSI_SUPPLIER	VARCHAR2(50 CHAR)
NO_UANG_MUKA	VARCHAR2(50 CHAR)
IS_RETURNED	NUMBER(1,0)
TGL_PERINTAH_BAYAR	DATE
CARA_TARIK	VARCHAR2(255 CHAR)
NO_REGISTER	VARCHAR2(255 CHAR)
STATUS_KWITANSI	NUMBER(10,0)
STATUS_PUNGUTAN	NUMBER(10,0)
UM_RETURNED	NUMBER(1,0)
STATUS_SPP	NUMBER(10,0)
STATUS_SPTB	NUMBER(10,0)
KODE_UNIT_TEKNIS	VARCHAR2(10 CHAR)
URAIAN_PERINTAH_BAYAR	VARCHAR2(255 CHAR)
URAIAN	VARCHAR2(225 CHAR)
KETERANGAN_VALIDASI	VARCHAR2(255 CHAR)
ADA_BAST	NUMBER(1,0)
KATEGORI_PENGELUARAN	NUMBER(1,0)
TAHUN_ANGGARAN_HIBAH	VARCHAR2(1 CHAR)
KATEGORI_PENGELUARAN_HIBAH	VARCHAR2(1 CHAR)
KODE_AKUN	VARCHAR2(255 CHAR)
JUMLAH	NUMBER(19,2)
STATUS_VALIDASI	NUMBER(1,0)
KODE_COA	VARCHAR2(255 CHAR)
NAMA_WP	VARCHAR2(100 CHAR)
NPWP_WP	VARCHAR2(20 CHAR)
NIP_PPK	VARCHAR2(255 CHAR)
kuitansi
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
Keterangan:
Berisi seluruh kuitansi bukti pembayaran yang dilakukan oleh bendahara.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/BEN/kuitansi/KLxxx/000000


Elemen data:

COLUMN_NAME	DATA_TYPE
ID	NUMBER(19,0)
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
KODE_UNIT_TEKNIS	VARCHAR2(10 CHAR)
NON_BARANG	NUMBER(1,0)
CARA_BAYAR	VARCHAR2(1 CHAR)
JABATAN_PENERIMA	VARCHAR2(100 CHAR)
JNS_KUITANSI	VARCHAR2(1 CHAR)
JUMLAH	NUMBER(19,2)
KETERANGAN	VARCHAR2(2000 CHAR)
NAMA_PENERIMA	VARCHAR2(100 CHAR)
NO_KWITANSI	VARCHAR2(50 CHAR)
STATUS_BARANG	NUMBER(1,0)
STATUS_KWITANSI	VARCHAR2(1 CHAR)
THANG	VARCHAR2(255 CHAR)
TGL_BAYAR	DATE
TGL_KWITANSI	DATE
PERINTAH_BAYAR_ID	NUMBER(19,0)
KODE_COA	VARCHAR2(255 CHAR)
NO_DRPP_ID	NUMBER(19,0)
NIP_PPK	VARCHAR2(255 CHAR)
ID_REK_BANK	NUMBER(19,0)
drpp
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
Keterangan:
Berisi Daftar Rincian Permintaan Pembayaran (DRPP) yang berisi kumpulan kuitansi yang akan di-SPJ-kan.



Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/BEN/drpp /KLxxx/000000


Elemen data:

COLUMN_NAME	DATA_TYPE
ID	NUMBER(19,0)
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
KODE_UNIT_TEKNIS	VARCHAR2(255 CHAR)
KODE_COA	VARCHAR2(255 CHAR)
JENIS_KUITANSI	VARCHAR2(1 CHAR)
TOTAL_JUMLAH	NUMBER(19,2)
NO_DRPP	VARCHAR2(50 CHAR)
STATUS_DRPP	VARCHAR2(1 CHAR)
THANG	VARCHAR2(255 CHAR)
TGL_DRPP	DATE
TAHUN_ANG_HIBAH	VARCHAR2(255 CHAR)
ID_SPP	NUMBER(19,0)
JUMLAH	NUMBER(19,2)
pungutPajak
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
Keterangan:
Berisi pungutan pajak atas transaksi bendahara.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/BEN/pungutPajak/KLxxx/000000

﻿

Elemen data:﻿

COLUMN_NAME	DATA_TYPE
ID	NUMBER(19,0)
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
THANG	VARCHAR2(255 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
KODE_UNIT_TEKNIS	VARCHAR2(10 CHAR)
DASAR_PUNGUT	VARCHAR2(1 CHAR)
KETERANGAN	VARCHAR2(2000 CHAR)
NO_BUKTI_PUNGUT	VARCHAR2(50 CHAR)
TGL_PUNGUT	DATE
PERINTAH_BAYAR_ID	NUMBER(19,0)
WAJIB_PAJAK_ID	NUMBER(19,0)
NAMA_WAJIB_PAJAK	VARCHAR2(100 CHAR)
NO_REKENING	VARCHAR2(20 BYTE)
KODE_BANK	VARCHAR2(255 BYTE)
NAMA_BANK	VARCHAR2(4000 CHAR)
ALAMAT	VARCHAR2(255 CHAR)
JENIS_PEM_KAS	VARCHAR2(1 BYTE)
KODE_AKUN	VARCHAR2(6 CHAR)
JUMLAH	NUMBER(19,2)
STATUS	VARCHAR2(1 CHAR)
SETORAN_PAJAK_ID	NUMBER(19,0)
setorPajak
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
Keterangan:
Berisi setoran pajak atas transaksi bendahara.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/BEN/setorPajak/KLxxx/000000


Elemen data:

COLUMN_NAME	DATA_TYPE
ID	NUMBER(19,0)
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
KODE_UNIT_TEKNIS	VARCHAR2(10 CHAR)
THANG	VARCHAR2(255 CHAR)
KODE_AKUN	VARCHAR2(6 CHAR)
ALAMAT_OBJECT_PAJAK	VARCHAR2(255 CHAR)
KODE_BANK	VARCHAR2(255 CHAR)
CABANG	VARCHAR2(50 CHAR)
CARA_SETOR	VARCHAR2(1 CHAR)
JENIS_SETORAN	VARCHAR2(50 CHAR)
JUMLAH_SETOR_PAJAK	NUMBER(19,2)
KETERANGAN	VARCHAR2(2000 CHAR)
MASA_PAJAK	VARCHAR2(6 CHAR)
NO_KETETAPAN	VARCHAR2(50 BYTE)
NO_OBJECT_PAJAK	VARCHAR2(50 CHAR)
NO_REF_GL	VARCHAR2(50 CHAR)
NO_SSP	VARCHAR2(50 CHAR)
NTPN	VARCHAR2(20 CHAR)
TGL_SETORAN	DATE
TGL_TERIMA_BANK	DATE
NAMA_WAJIB_PAJAK	VARCHAR2(100 CHAR)
NTB	VARCHAR2(20 BYTE)
TGL_BUKU	DATE
pnbp
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
Keterangan:
Berisi setoran Penerimaan Negara Bukan Pajak (PNBP) yang telah dicatat Bendahara pada Aplikasi SAKTI.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/BEN/pnbp/KLxxx/000000

﻿
Elemen data:

COLUMN_NAME	DATA_TYPE
ID	NUMBER(19,0)
KODE_KEMENTERIAN	VARCHAR2(255 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
KODE_UNIT_TEKNIS	VARCHAR2(50 CHAR)
KODE_PROGRAM	VARCHAR2(255 CHAR)
KODE_KEGIATAN	VARCHAR2(255 CHAR)
KODE_OUTPUT	VARCHAR2(255 CHAR)
KODE_KPPN	VARCHAR2(255 CHAR)
KODE_AKUN	VARCHAR2(255 CHAR)
KODE_BANK	VARCHAR2(255 CHAR)
KODE_SUB_FUNGSI	VARCHAR2(255 CHAR)
THANG	VARCHAR2(255 CHAR)
CABANG	VARCHAR2(50 CHAR)
NTPN	VARCHAR2(20 CHAR)
CARA_SETOR	VARCHAR2(1 CHAR)
JENIS_PENGEMBALIAN	VARCHAR2(2 BYTE)
JENIS_PUNGUT	VARCHAR2(1 CHAR)
JUMLAH	NUMBER(19,2)
KETERANGAN	VARCHAR2(2000 CHAR)
NO_SPN	VARCHAR2(50 BYTE)
TGL_SPN	DATE
NO_SSBP	VARCHAR2(50 CHAR)
STATUS	VARCHAR2(1 CHAR)
TGL_SSBP	DATE
TGL_TERIMA_BANK	DATE
WAJIB_PAJAK_ID	NUMBER(19,0)
FLAG_OWNER	NUMBER(1,0)
NTB	VARCHAR2(20 BYTE)
TGL_BUKU	DATE
tup
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
Keterangan:
Berisi data pengajuan Tambahan Uang Persediaan (TUP) yang diajukan ke KPPN.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/BEN/tup/KLxxx/000000


Elemen data:


pengembalian
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
Keterangan:
Berisikan data pengembalian belanja yang direkam di modul bendahara yang biasa disebut SSPB.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/BEN/pengembalian/KLxxx/000000



Elemen data:


Komitmen
Dokumentasi endpoint kelompok modul Komitmen

capaianRO
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Opsional	Kode satker
2.	KODE_PERIODE	Opsional	Kode periode
Keterangan:

Untuk mengambil data yang ada pada menu realisasi kinerja satker.



Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/KOM/capaianRO/KLxxx/000000/2022-01



Elemen data:

COLUMN_NAME	DATA_TYPE
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KODE_UNIT	VARCHAR2(2 CHAR)
KDSATKER	VARCHAR2(6 CHAR)
SUB_OUTPUT_KODE	VARCHAR2(255 CHAR)
KODE_PERIODE	VARCHAR2(7 CHAR)
STATUS	VARCHAR2(255 CHAR)
RENCANA_SUB_OUTPUT	NUMBER(21,4)
SATUAN_SUB_OUTPUT	VARCHAR2(255 CHAR)
PENAMBAHAN_REALISASI_VOLUME_RO	NUMBER(21,4)
TOTAL_REALISASI_SUB_OUTPUT	NUMBER(21,4)
PENAMBAHAN_PROGRESS_CAPAIAN_RO	FLOAT
TOTAL_PROGRESS_CAPAIAN_RO	FLOAT
BUKTI_DOKUMEN	VARCHAR2(255 CHAR)
REFERENSI_KETERANGAN	VARCHAR2(365 CHAR)
REFERENSI	VARCHAR2(2 CHAR)
KETERANGAN	VARCHAR2(365 CHAR)
RO_STRATEGIS	NUMBER(1,0)
ANGGARAN_BELANJA	NUMBER(19,2)
REALISASI_BELANJA	NUMBER(19,2)
PENGEMBALIAN_BELANJA	NUMBER(19,2)
PERSEN_GAP	FLOAT
REVISI_DIPA_KE	NUMBER(10,0)
kontrakHeader
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
Keterangan:




Contoh link pengambilan data:



https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/KOM/kontrakHeader/KLxxx/000000/



Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)	Berisi Kode BA
KDSATKER	VARCHAR2(255 CHAR)	Berisi Kode Satker
THN_ANG	VARCHAR2(255 CHAR)	Berisi Tahun Anggaran
ID_KONTRAK	NUMBER(19,0)	Berisi Id kontrak(unique primary key)
NO_KONTRAK	VARCHAR2(150 CHAR)	Berisi Nomor kontrak
TANGGAL_KONTRAK	VARCHAR2(17 BYTE)	Berisi
TANGGAL_MULAI_PELAKSANAAN	VARCHAR2(17 BYTE)	Berisi
TANGGAL_SELESAI_PELAKSANAAN	VARCHAR2(17 BYTE)	Berisi
NILAI_KONTRAK	NUMBER(21,4)	Berisi
MATA_UANG	VARCHAR2(255 CHAR)	Berisi
TIPE_KONTRAK	VARCHAR2(4000 CHAR)	Multiyear/Annual
NOMOR_CAN	VARCHAR2(100 CHAR)	Berisi NRK(bukti sudah didaftarkan di SPAN)
JENIS_KONTRAK	VARCHAR2(4000 CHAR)	BA BUN, Kontrak, Release Multi Year
URAIAN_KONTRAK	VARCHAR2(240 CHAR)	Berisi Uraian Kontrak
ID_SUPPLIER	NUMBER(19,0)	Id Supplier header
NAMA_SUPPLIER	VARCHAR2(4000 CHAR)

kontrakLine
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
2.	ID_KONTRAK	Opsional	ID Kontrak
3.	ID_LINE_KONTRAK	Opsional	ID Line Kontrak
Keterangan:



Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/KOM/kontrakLine/KLxxx/000000/00001/00001/



Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
ID_KONTRAK	NUMBER(19,0)	ID Kontrak
ID_LINE_KONTRAK	NUMBER(19,0)	PK
DESKRIPSI_LINE	VARCHAR2(240 CHAR)
CARA_TARIK	VARCHAR2(4000 CHAR)	Cara Tarik sesuai pagu DIPA
NILAI_LINE	NUMBER(19,2)	Nilai
TIPE_LINE	VARCHAR2(30 BYTE)	Belanja/Biaya
URUTAN_LINE	NUMBER(10,0)	Urutan(gen by sys)
kontrakTermin
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
2.	ID_KONTRAK	Opsional	ID Kontrak
3.	ID_LINE_KONTRAK	Opsional	ID Line Kontrak
4.	ID_JADWAL_PEMBAYARAN	Opsional	ID Jadwal Pembayaran
Keterangan:





Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/KOM/kontrakTermin/KLxxx/000000/00001/00001/000001/



Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
ID_KONTRAK	NUMBER(19,0)
ID_LINE_KONTRAK	NUMBER(19,0)
ID_JADWAL_PEMBAYARAN	NUMBER(19,0)	PK
TERMIN_KE	VARCHAR2(50 CHAR)	Sq yang nentuin satker
DESKRIPSI_TERMIN	VARCHAR2(255 CHAR)
TANGGAL_TERMIN	VARCHAR2(17 BYTE)
NILAI_TERMIN	NUMBER(19,2)
kontrakCOA
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
2.	ID_KONTRAK	Opsional	ID Kontrak
3.	ID_LINE_KONTRAK	Opsional	ID Line Kontrak
4.	ID_JADWAL_PEMBAYARAN	Opsional	ID Jadwal Pembayaran
Keterangan:





Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/KOM/kontrakCOA/KLxxx/000000/00001/00001/000001/




Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(6 CHAR)
ID_KONTRAK	NUMBER(19,0)
ID_LINE_KONTRAK	NUMBER(19,0)
ID_JADWAL_PEMBAYARAN	NUMBER(19,0)
KODE_PROGRAM	VARCHAR2(2 CHAR)
KODE_KEGIATAN	VARCHAR2(4 CHAR)
KODE_OUTPUT	VARCHAR2(3 CHAR)
KODE_AKUN	VARCHAR2(6 CHAR)
KODE_SUBOUTPUT	VARCHAR2(3 CHAR)
KODE_KOMPONEN	VARCHAR2(3 CHAR)
KODE_SUBKOMPONEN	VARCHAR2(2 CHAR)
KODE_ITEM	VARCHAR2(6 CHAR)
KODE_COA	VARCHAR2(100 CHAR)
VOL_SUBOUTPUT	NUMBER(21,4)
NILAI_COA_DETAIL	NUMBER(19,2)
BASTKontrakHeader
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
2.	ID_BAST	Opsional	ID BAST
Keterangan:




Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/KOM/BASTKontrakHeader/KLxxx/000000/



Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
THN_ANG	VARCHAR2(255 CHAR)
ID_BAST	NUMBER(19,0)	PK
NO_KONTRAK	VARCHAR2(255 CHAR)
NO_BAST	VARCHAR2(50 CHAR)
TANGGAL_BAST	VARCHAR2(17 BYTE)
KATEGORI_BAST	VARCHAR2(4000 CHAR)	Barang jasa bastbg Escrow realisasi barang, e r jasa
NILAI_BAST	NUMBER
NOMOR_DAN_STATUS_SPP	VARCHAR2(4000 CHAR)	Nomor-status spp
JENIS_SPP	VARCHAR2(4000 CHAR)
BASTKontrakDetailBarang
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
2.	ID_BAST	Opsional	ID BAST
Keterangan:




Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/KOM/BASTKontrakDetailBarang/KLxxx/000000/


Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
ID_BAST	NUMBER(19,0)
KODE_BARANG	VARCHAR2(255 CHAR)	Kode barang subsubkelompok
NAMA_BARANG	VARCHAR2(4000 CHAR)	Deskripsi barang
JUMLAH_BARANG	NUMBER(10,0)	kuantitas
NILAI_TOTAL_BARANG	NUMBER	Harga total
STATUS_PENDETAILAN	CHAR(17 BYTE)	Sudah Didetailkan/ Belum Didetailkan
BASTKontrakCOA
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
2.	ID_BAST	Opsional	ID BAST
Keterangan:


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/KOM/BASTKontrakCOA/KLxxx/000000/



Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
ID_BAST	NUMBER(19,0)
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(6 CHAR)
KODE_PROGRAM	VARCHAR2(2 CHAR)
KODE_KEGIATAN	VARCHAR2(4 CHAR)
KODE_AKUN	VARCHAR2(6 CHAR)
KODE_OUTPUT	VARCHAR2(3 CHAR)
KODE_SUBOUTPUT	VARCHAR2(3 CHAR)
KODE_KOMPONEN	VARCHAR2(3 CHAR)
KODE_SUBKOMPONEN	VARCHAR2(2 CHAR)
KODE_ITEM	VARCHAR2(6 CHAR)
KODE_COA	VARCHAR2(100 CHAR)
VOL_SUBOUTPUT	NUMBER(21,4)
NILAI_COA_DETAIL	NUMBER(19,2)
BASTNonKontrakHeader
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
2.	ID_BAST	Opsional	ID BAST
Keterangan:
Berisi data induk BAST non kontraktual.


Kategori BAST:

BAST Barang TUP
Kuitansi GUP KKP Jasa
BAST Jasa
BAST Barang UP
BAST Barang HIBAH
BAST Barang
BAST Jasa TUP
Kuitansi GUP Valas Jasa
BAST Jasa HIBAH
Kuitansi GUP KKP Barang
Kuitansi TUP KKP Jasa
Kuitansi GUP Valas Barang
Kuitansi TUP KKP Barang
Kuitansi TUP Valas Jasa
Kuitansi TUP Valas Barang
BAST Jasa UP


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/PEM/BASTNonKontrakHeader/KLxxx/000000/


Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
THN_ANG	VARCHAR2(255 CHAR)
ID_BAST	NUMBER(19,0)	PK
NO_DOKUMEN	VARCHAR2(50 CHAR)
TANGGAL_BAST	VARCHAR2(17 BYTE)
KATEGORI_BAST	VARCHAR2(4000 CHAR)
URAIAN_BAST	VARCHAR2(255 CHAR)
NILAI_BAST	NUMBER(19,2)
SUPPLIER_WP_WB	VARCHAR2(4000 CHAR)
NOMOR_DAN_STATUS_SPP_SPBY	VARCHAR2(4000 CHAR)	Nomor spp/spby - status
JENIS_SPP_SPBY	VARCHAR2(4000 CHAR)	Jenis spp(berisi 3 digit kode jenis spp)/spby
BASTNonKontrakDetailBarang
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
2.	ID_BAST	Opsional	ID BAST
Keterangan:
Berisikan informasi detail barang yang diperoleh dari BAST non kontraktual.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/PEM/BASTNonKontrakDetailBarang/KLxxx/000000/


Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
ID_BAST	NUMBER(19,0)
KODE_BARANG	VARCHAR2(255 CHAR)	Kode barang subsubkelompok, dapat digunakan untuk digabungkan ke modul Aset
NAMA_BARANG	VARCHAR2(4000 CHAR)
JUMLAH_BARANG	NUMBER(10,0)
NILAI_TOTAL_BARANG	NUMBER
STATUS_PENDETAILAN	CHAR(17 BYTE)
BASTNonKontrakCOA
﻿Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
2.	ID_BAST	Opsional	ID BAST
Keterangan:
Berisikan distribusi COA 16 segmen pada BAST non kontraktual.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/KOM/BASTNonKontrakCOA/KLxxx/000000/


Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(6 CHAR)
ID_BAST	NUMBER(19,0)
KODE_PROGRAM	VARCHAR2(2 CHAR)
KODE_KEGIATAN	VARCHAR2(4 CHAR)
KODE_AKUN	VARCHAR2(6 CHAR)
KODE_OUTPUT	VARCHAR2(3 CHAR)
KODE_SUBOUTPUT	VARCHAR2(3 CHAR)
KODE_KOMPONEN	VARCHAR2(3 CHAR)
KODE_SUBKOMPONEN	VARCHAR2(2 CHAR)
KODE_ITEM	VARCHAR2(6 CHAR)
KODE_COA	VARCHAR2(255 CHAR)
VOL_SUB_OUTPUT	NUMBER
NILAI_COA_DETAIL	NUMBER(21,2)
supplierHeader
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	ID_SUPPLIER	Wajib	ID SUPPLIER
2.	ID_SUPPLIER_ADDRESS	Wajib	ID SUPPLIER ADDRESS
3.	NAMA_BANK	Opsional	Nama Bank
Keterangan:

Berisikan informasi induk pada data supplier yang terdaftar pada SAKTI.



Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/KOM/supplierHeader/KLxxx/



Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
ID_SUPPLIER	NUMBER(19,0)
NAMA_SUPPLIER	VARCHAR2(4000 CHAR)
NPWP	VARCHAR2(20 CHAR)
NRS	VARCHAR2(255 CHAR)
STATUS_DATA	VARCHAR2(8 BYTE)	Register(bcsr) ubah(bcsu) inaktif(bcsi)
supplierAddress
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	ID_SUPPLIER	Wajib	ID SUPPLIER
Keterangan:
Berisikan informasi detail alamat pada supplier.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/KOM/supplierAddress/KLxxx/


Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
ID_SUPPLIER	NUMBER(19,0)
ID_SUPPLIER_ADDRESS	NUMBER(19,0)
NAMA_SITE	VARCHAR2(220 CHAR)	Kode tipe supplier_kodepos
KODE_KPPN	VARCHAR2(255 CHAR)
KODE_TIPE_SUPPLIER	VARCHAR2(255 CHAR)	1-8
ALAMAT1	VARCHAR2(255 CHAR)
ALAMAT2	VARCHAR2(255 CHAR)
KODE_NEGARA	VARCHAR2(255 CHAR)
PROVINSI	VARCHAR2(4000 CHAR)
KOTA	VARCHAR2(4000 CHAR)
KECAMATAN	VARCHAR2(4000 CHAR)
KODE_POS	VARCHAR2(4000 CHAR)
NO_TELP	VARCHAR2(15 CHAR)
NO_FAX	VARCHAR2(15 CHAR)
EMAIL	VARCHAR2(50 CHAR)
STATUS_DATA	VARCHAR2(8 BYTE)	Register(bcsr) ubah(bcsu) inaktif(bcsi)
supplierBank
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	ID_SUPPLIER	Wajib	ID SUPPLIER
2.	ID_SUPPLIER_ADDRESS	Wajib	ID SUPPLIER ADDRESS
3.	NAMA_BANK	Opsional	Nama Bank
Keterangan:
Berisikan informasi detail bank pada supplier.



Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/KOM/supplierBank/KLxxx/


Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(255 CHAR)
ID_SUPPLIER	NUMBER(19,0)
ID_SUPPLIER_ADDRESS	NUMBER(19,0)
NAMA_BANK	VARCHAR2(4000 CHAR)
NAMA_CABANG_BANK	VARCHAR2(4000 CHAR)	Deskripsi negara
ALAMAT_BANK	VARCHAR2(150 CHAR)
NAMA_PEMILIK_REKENING	VARCHAR2(100 CHAR)
NO_REKENING	VARCHAR2(100 CHAR)
MATA_UANG	VARCHAR2(255 CHAR)
DETIL_NAMA_CABANG_BANK	VARCHAR2(150 BYTE)
NAMA_PEGAWAI_PEMDA_ PENERUSAN_PINJAMAN	VARCHAR2(100 CHAR)	Tipe 3/6
NPWP	VARCHAR2(15 CHAR)	3/5/6 – mungkin dari header
NIP	VARCHAR2(18 CHAR)	Hanya tipe 3
ALAMAT1	VARCHAR2(255 CHAR)
ALAMAT2	VARCHAR2(255 CHAR)
KOTA	VARCHAR2(4000 CHAR)
PROPINSI	VARCHAR2(4000 CHAR)
KODE_POS	VARCHAR2(5 CHAR)
STATUS_DATA	VARCHAR2(8 BYTE)	Register(bcsr) ubah(bcsu) inaktif(bcsi)
Aset
Dokumentasi endpoint kelompok modul Aset

asetTrx
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
2.	KDGOL	Wajib	Kode golongan
3.	KDBID	Wajib	Kode bidang
4.	KDKEL	Opsional	Kode kelompok
5.	KDSKEL	Opsional	Kode sub kelompok
6.	KDBRG	Opsional	Kode barang
Keterangan:

Berisi data transaksi aset historis untuk masing-masing kode barang.


Tips:

Apabila data terlalu besar akan mengeluarkan pesan error " Fatal error: Allowed memory size of...". Apabila hal tersebut terjadi, silahkan perkecil cakupan data dengan menggunakan kode kelompok, subkelompok, ataupun sampai kode barang apabila diperlukan. Gunakan endpoint refAset untuk mendapatkan list referensi kode barang untuk melakukan looping request.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/AST/asetTrx/KLxxx/000000/2/201/20103/2010307/2010307999


Elemen data:

No	Nama Kolom	Jenis	Deskripsi	Keterangan
1	KODE_KEMENTERIAN	VARCHAR2(3 CHAR)	Kode BA/Dept
2	KDSATKER	VARCHAR2(6 CHAR)	Kode Satker	Referensi ada pada refAdmin
3	KDUAKPB	VARCHAR2(255 CHAR)	Kode UAKPB
4	KDGOL	VARCHAR2(1 CHAR)	Kode Golongan Barang	Referensi ada pada refAset/KDGOL
5	KDBID	VARCHAR2(3 CHAR)	Kode Bidang Barang	Referensi ada pada refAset/KDBID
6	KDKEL	VARCHAR2(5 CHAR)	Kode Kelompok Barang	Referensi ada pada refAset/KDKEL
7	KDSKEL	VARCHAR2(7 CHAR)	Kode Sub Kelompok Barang	Referensi ada pada refAset/KDSKEL
8	KDBRG	VARCHAR2(255 CHAR)	Kode Barang	Referensi ada pada refAset/KDBRG
9	NUP	NUMBER(19,0)	No. Asset
10	KOND	NUMBER(10,0)	Kode Kondisi	1=Baik, 2=rusak Ringan, 3=Rusak Berat
11	KDTRX	VARCHAR2(255 CHAR)	Kode Jenis Transaksi	Referensi ada pada refAset/KDTRX
12	Q_AST	NUMBER(19,0)	Kuantitas Asset	Jumlah aset
13	Q_PRB	NUMBER(19,0)	Kuantitas Perubahan	Penambahan atau pengurangan aset suatu transaksi
14	NA	NUMBER(19,0)	Nilai Asset	Nilai aset bruto
15	NAN	NUMBER(19,0)	Nilai Asset Neraca	Nilai buku atau nilai aset bruto dikurang total penyusutan
16	NP	NUMBER(19,0)	Nilai Perubahan	Penambahan atau pengurangan aset transaksi
17	NPN	NUMBER(19,0)	Nilai Perubahan Neraca	Penambahan atau pengurangan penyusutan suatu transaksi
18	SM	NUMBER(10,0)	Sisa Masa Manfaat	Semesteran
19	MM	NUMBER(10,0)	Masa Manfaat	Tahunan
20	NO_SPPA	VARCHAR2(255 CHAR)	No. SPPA
21	STS	CHAR(1 BYTE)	Status Asset	1=Aktif, 2=Henti guna, 3=Mitra
22	KODE_SATKER_ASAL	VARCHAR2(255 CHAR)	Kode Satker asal	Satker intraco
23	KODE_REGISTER	VARCHAR2(64 BYTE)	Kode register	Identitas unik dari setiap aset
24	KET	VARCHAR2(255 CHAR)	Keterangan
25	NO_DOK	VARCHAR2(255 CHAR)	No. Dasar Mutasi
26	JNS_AST	NUMBER(10,0)	Jenis Asset	1=Intrakomptabel, 2=Ekstrakomptabel
27	PER	NUMBER(2,0)	Periode Transaksi	1 s/d 14
28	MEREK_TIPE	VARCHAR2(255 CHAR)	Merek Tipe
29	CTT	NUMBER(10,0)	Kode Tercatat	1=DBR, 2=DBL, 3=KIB
30	THN_ANG	NUMBER(10,0)	Tahun Anggaran
31	CREATED_DATE	DATE
32	CREATED_BY	VARCHAR2(50 CHAR)
33	TGL_BUKU	DATE	Tgl. Pembukuan
34	TGL_OLEH	DATE	Tgl. Perolehan
35	TGL_AWAL_PAKAI	DATE	Tgl. Awal Pemakaian

Persediaan
Dokumentasi endpoint kelompok modul Persediaan

persediaTrx
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
2.	KDGOL	Opsional	Kode golongan
3.	KDBID	Opsional	Kode bidang
4.	KDKEL	Opsional	Kode kelompok
5.	KDSKEL	Opsional	Kode sub kelompok
6.	KDBRG	Opsional	Kode barang
Keterangan:

Berisi data transaksi persediaan untuk masing-masing kode barang.


Tips:

Kami menyarankan untuk paling tidak mengirimkan variabel sampai dengan kode sub kelompok untuk menghindari data yang terlalu besar. Apabila data terlalu besar akan mengeluarkan pesan error " Fatal error: Allowed memory size of...".


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/PER/persediaTrx/KLxxx/000000/2/201/20103/2010307/2010307999


Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KODE_AKUN	VARCHAR2(255 CHAR)
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)
KDSATKER	VARCHAR2(6 CHAR)
KDGOL	VARCHAR2(1 CHAR)
KDBID	VARCHAR2(3 CHAR)
KDKEL	VARCHAR2(5 CHAR)
KDSKEL	VARCHAR2(7 CHAR)
KODE_KPB	VARCHAR2(255 CHAR)
JENIS_TRANSAKSI	VARCHAR2(255 BYTE)
KDBRG	VARCHAR2(255 CHAR)
KODE_PERSEDIAAN	VARCHAR2(255 CHAR)
NAMA_PERSEDIAAN	VARCHAR2(4000 CHAR)
KUANTITAS	NUMBER
NILAI	NUMBER
TGL_BUKU	VARCHAR2(10 BYTE)
KETERANGAN	VARCHAR2(255 CHAR)
Pelaporan
Dokumentasi endpoint kelompok modul Pelaporan

bukuBesar
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
2.	PERIODE	Wajib	Kode periode (format data: MM)
3.	OPSI_FILTER	Opsional	Dapat berisi ‘BB’ untuk filter hanya buku besar atau dapat berisi ‘kas’ untuk filter hanya jurnal berbasis kas.
Keterangan:

Berisi data seluruh jurnal-jurnal transaksi pada aplikasi SAKTI.


Tips:

Kami menyarankan untuk tidak melakukan penarikan data ulang untuk periode yang sudah tutup permanen.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/GLP/bukuBesar/KLxxx/000000/MM


Elemen data:


COLUMN_NAME	DATA_TYPE	KETERANGAN
KDBAES1	VARCHAR2(5 CHAR)	BerisiKode BA Eselon 1
KDKANWIL	VARCHAR2(3 CHAR)	BerisiKode Kanwil (khusus Kementerian Keuangan)
KDWILAYAH	VARCHAR2(4 CHAR)	BerisiKode Wilayah Provinsi atau Kabupaten/Kota
KDKPPN	VARCHAR2(3 CHAR)	BerisiKode KPPN Mitra Satker yang Bersangkutan
KDSATKER	VARCHAR2(6 CHAR)	BerisiKode Satker
KDFUNGSI	VARCHAR2(2 CHAR)	BerisiKode Fungsi (Biasanya terisi 00)
KDSFUNG	VARCHAR2(2 CHAR)	BerisiKode Sub Fungsi (Biasanya terisi 00)
KDPROGRAM	VARCHAR2(4 CHAR)	BerisiKode Program
KDGIAT	VARCHAR2(4 CHAR)	BerisiKode Kegiatan
KDSGIAT	VARCHAR2(5 CHAR)	BerisiKode Sub Kegiatan (Biasanya terisi 00000)
KDOUTPUT	VARCHAR2(3 CHAR)	BerisiKode Output
KDSOUTPUT	VARCHAR2(3 CHAR)	BerisiKode Sub Output (Biasanya dikosongkan)
KDKEM	VARCHAR2(1 CHAR)	BerisiKode Pengembalian (0 artinya Realisasi, 1 artinya pengembalian)
KDKAS	VARCHAR2(1 CHAR)	BerisiKode Kas
NKAS	VARCHAR2(1 CHAR)	BerisiKode yang membedakan transaksi Kas atau Non Kas (sudah tidak digunakan)
KDVAL	VARCHAR2(1 CHAR)	BerisiKode Valas
KDTRN	VARCHAR2(1 CHAR)	BerisiKode Transaksi
KDMAKMAP	VARCHAR2(6 CHAR)	BerisiKode Perkiraan
KDDK	VARCHAR2(1 CHAR)	BerisiKode Debet atau Kredit
PERKSAI	VARCHAR2(6 CHAR)	BerisiKode Perkiraan Kas pada Posisi Debet
PERKSAI1	VARCHAR2(6 CHAR)	BerisiKode Perkiraan Kas pada Posisi Kredit
PERKKOR	VARCHAR2(6 CHAR)	BerisiKode Perkiraan Akrual pada Posisi Debet
PERKKOR1	VARCHAR2(6 CHAR)	BerisiKode Perkiraan Akrual pada Posisi Kredit
KDKM	VARCHAR2(1 CHAR)	BerisiKode Keluar Masuk (Hanya untuk kasus-kasus tertentu)
KDSDCP	VARCHAR2(3 CHAR)	BerisiKode Sumber Dana dan Cara Penarikan
THNANG	VARCHAR2(4 CHAR)	BerisiTahun Anggaran Transaksi
PERIODE	VARCHAR2(2 CHAR)	BerisiPeriode Transaksi
RPHREAL	NUMBER(24,2)	BerisiJumlah Rupiah suatu Transaksi
TGLKIRIM	DATE	BerisiTanggal Kirim (Sudah Jarang digunakan)
TGLTERIMA	DATE	BerisiTanggal Terima (Sudah Jarang digunakan)
TGLUPDATE	DATE	BerisiTanggal Updating suatu transaksi
FLAGREV	VARCHAR2(1 CHAR)	BerisiKode Pemisah Revisi DIPA atau DIPA Awal
KDBAPEL	VARCHAR2(3 CHAR)	BerisiKode BA Pelaksana (Sudah Jarang digunakan)
KDES1PEL	VARCHAR2(2 CHAR)	BerisiKode Eselon I Pelaksana (Sudah Jarang digunakan)
JNSDOK1	VARCHAR2(10 CHAR)	BerisiKode Jenis Dokumen
NODOK1	VARCHAR2(255 CHAR)	BerisiNomor Dokumen Transaksi
TGLDOK1	DATE	BerisiTanggal Dokumen Transaksi
KDDEKON	VARCHAR2(2 CHAR)	BerisiKode Kewenangan
REGISTER	VARCHAR2(8 CHAR)	BerisiKode Register Apabila Sumber Dana berasal dari BLN
KDJENDOK	VARCHAR2(2 CHAR)	BerisiKode Jenis Dokumen BLN atau HLN
KDKANWILK	VARCHAR2(3 CHAR)	BerisiKode Kanwil DJPb (Sudah Tidak Digunakan)
TGLPOST	DATE	BerisiTanggal Posting Transaksi
REVISIKE	NUMBER(2,0)	BerisiAngka Incremental Revisi DIPA
DIPAKE	NUMBER(3,0)	BerisiAngka Incremental DIPA
KDCRBAY	VARCHAR2(1 CHAR)	BerisiKode Cara Bayar
NOKARWAS	VARCHAR2(5 CHAR)	BerisiNomor Karwas
KDBEBAN	VARCHAR2(1 CHAR)	BerisiKode Beban
KDJNSBAN	VARCHAR2(1 CHAR)	BerisiKode Jenis Bantuan
KDBLU	VARCHAR2(1 CHAR)	BerisiKode Pemisah Transaksi BLU atau Non BLU
NOREGIS	VARCHAR2(40 CHAR)	BerisiNomor Register
KDVALAS	VARCHAR2(3 CHAR)	BerisiKode Valas yang digunakan
NILKURS	NUMBER(20,2)	BerisiNilai Kurs
TGLKURS	DATE	BerisiTanggal Kurs
KDKPKNL	VARCHAR2(5 CHAR)	BerisiKode KPKNL
STAT_REKON	VARCHAR2(1 CHAR)	BerisiStatus Rekon (Sudah Tidak digunakan)
KATEGORI	VARCHAR2(40 CHAR)	BerisiKode Kategori Sumber Dana
TRN_BMN	VARCHAR2(3 CHAR)	BerisiKode Transaksi yang berasal dari SIMAK-BMN
NIL_VALAS	NUMBER(20,2)	BerisiNilai Valas
CAD1	VARCHAR2(10 CHAR)	BerisiKode Wilayah SIMAK-BMN
CAD2	VARCHAR2(10 CHAR)	Berisi10 digit Pertama Kode Lokasi Intraco transaksi TKTM
CAD3	VARCHAR2(10 CHAR)	Berisi10 digit Kedua Kode Lokasi Intraco transaksi TKTM
HAPUS	NUMBER(1,0)	BerisiKode Hapus
ID	NUMBER(10,0)	BerisiID sebuah transaksi (didapat otomatis dari sistem)

neracaSawal
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
Keterangan:

Berisi data saldo awal neraca atau saldo akhir pada tahun lalu. Data akan bersifat statis apabila status laporan keuangan sudah audited sehingga tidak perlu dilakukan updating data secara berkala.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/GLP/neracaSawal/KLxxx/000000/


Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
KDBAES1	VARCHAR2(5 CHAR)	Berisi Kode BA Eselon 1
KDSATKER	VARCHAR2(6 CHAR)	Berisi Kode SAtker
AKUN	VARCHAR2(6 CHAR)	Berisi Kode Akun 6 digit
NILAI	NUMBER	Berisik Nilai pada Akun

faDetail
Variabel yang diterima:

No.	Nama	Sifat	Keterangan
1.	KDSATKER	Wajib	Kode satker
2.	PERIODE	Wajib	Kode periode (format data: MM)
Keterangan:

Berisi data seluruh transaksi yang berdampak pada FA akrual pada SAKTI.


Contoh link pengambilan data:

https://monsakti.kemenkeu.go.id/sitp-monsakti-omspan/webservice/API/GLP/faDetail/KLxxx/000000/08


Elemen data:

COLUMN NAME	DATA TYPE	KETERANGAN
ID	NUMBER(19,0)	Berisi Kode BA Eselon 1
KDSATKER	VARCHAR2(6 CHAR)	Berisi Kode SAtker
KODE_KEMENTERIAN	VARCHAR2(3 CHAR)	Berisi Kode Akun 6 digit
DESKRIPSI_TRANS	VARCHAR2(255 CHAR)	Berisik Nilai pada Akun
JENIS_DOKUMEN	VARCHAR2(3 CHAR)
KODE_COA	VARCHAR2(100 CHAR)
KODE_MATA_UANG_TRANS	VARCHAR2(10 CHAR)
KODE_PERIODE	VARCHAR2(7 CHAR)
KODE_SDATA	VARCHAR2(4 CHAR)
KURS	NUMBER(19,2)
NILAI_RUPIAH	NUMBER(19,2)
NILAI_TRANS_VALAS	NUMBER(19,2)
NO_DOK	VARCHAR2(255 CHAR)
NOMOR_DIPA	VARCHAR2(30 CHAR)
TANGGAL_DIPA	DATE
TGL_DOK	DATE
TGL_JURNAL	DATE
ID_TRN_MODUL	VARCHAR2(255 CHAR)
© 2014 - 2023 Direktorat Jenderal Perbendaharaan
