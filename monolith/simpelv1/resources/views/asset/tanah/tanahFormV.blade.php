@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }} Asset Tanah</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/asset/tanah" method="POST" class="ajaxForm">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Nama Satker *</label>
                                    <input class="form-control" id="nm_satker" name="nm_satker" required value="{{ $model['nama_satker'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Kode Barang *</label>
                                    <input class="form-control" id="kode_barang" name="kode_barang" required value="{{ $model['kode_barang'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-9">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nama Barang *</label>
                                    <input type="text" class="form-control" id="nm_barang" name="nm_barang" required value="{{ $model['nama_barang'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Kelompok *</label>
                                    <input class="form-control" id="kode_barang" name="kelompok" required value="{{ $model['kelompok'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-9">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Sub Kelompok *</label>
                                    <input type="text" class="form-control" id="sub_kelompok" name="sub_kelompok" required value="{{ $model['sub_kelompok'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">NUP</label>
                                    <input type="text" class="form-control" id="nup" name="nup" value="{{ $model['nup'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Kondisi</label>
                                    <input class="form-control" id="kondisi" name="kondisi" required value="{{ $model['kondisi'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Jenis Dokumen</label>
                                    <input class="form-control" id="jenis_dokumen" name="jenis_dokumen" required value="{{ $model['jenis_dokumen'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Kepemilikan</label>
                                    <input type="text" class="form-control" id="kepemilikan" name="kepemilikan" value="{{ $model['kepemilikan'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Jenis Sertifikat</label>
                                    <input type="text" class="form-control" id="jenis_sertifikat" name="jenis_sertifikat" value="{{ $model['jenis_sertifikat'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-9">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Merk/Tipe</label>
                                    <input type="text" class="form-control" id="merk" name="merk" value="{{ $model['merk'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Status Penggunaan</label>
                                    <input type="text" class="form-control" id="status_penggunaan" name="status_penggunaan" value="{{ $model['status_penggunaan'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Status Pengelolaan</label>
                                    <input type="text" class="form-control" id="status_pengelolaan" name="status_pengelolaan" value="{{ $model['status_pengelolaan'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nilai Perolehan Pertama</label>
                                    <input type="text" class="form-control angka" id="nilai_perolehan_pertama" name="nilai_perolehan_pertama" value="{{ $model['nilai_perolehan_pertama'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nilai Mutasi</label>
                                    <input type="text" class="form-control angka" id="nilai_mutasi" name="nilai_mutasi" value="{{ $model['nilai_mutasi'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nilai Perolehan</label>
                                    <input type="text" class="form-control angka" id="nilai_perolehan" name="nilai_perolehan" value="{{ $model['nilai_perolehan'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nilai Penyusutan</label>
                                    <input type="text" class="form-control angka" id="nilai_penyusutan" name="nilai_penyusutan" value="{{ $model['nilai_penyusutan'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nilai Buku</label>
                                    <input type="text" class="form-control angka" id="nilai_buku" name="nilai_buku" value="{{ $model['nilai_buku'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Kuantitas(m2)</label>
                                    <input type="text" class="form-control angka" id="kuantitas" name="kuantitas" value="{{ $model['kuantitas'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Luas Tanah Seluruhnya</label>
                                    <input type="text" class="form-control angka" id="luas_tanah_total" name="luas_tanah_total" value="{{ $model['luas_tanah_seluruhnya'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Luas Tanah Untuk Bangunan</label>
                                    <input type="text" class="form-control angka" id="luas_tanah_bangunan" name="luas_tanah_bangunan" value="{{ $model['luas_tanah_untuk_bangunan'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Luas tanah Untuk Sarana Lingkungan</label>
                                    <input type="text" class="form-control angka" id="luas_tanah_sarana" name="luas_tanah_sarana" value="{{ $model['luas_tanah_sarana_lingkungan'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Luas Lahan Kosong</label>
                                    <input type="text" class="form-control angka" id="luas_lahan_kosong" name="luas_lahan_kosong" value="{{ $model['luas_tanah_kosong'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Jumlah Foto</label>
                                    <input type="text" class="form-control angka" id="jml_foto" name="jml_foto" value="{{ $model['jumlah_foto'] ?? '' }}">
                                </div>
                            </div>
                            
                        </div>
                        <div class="row">
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">No. PSP</label>
                                    <input type="text" class="form-control" id="no_psp" name="no_psp" value="{{ $model['no_psp'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <!-- <label for="name" class="form-label">Tanggal PSP</label>
                                    <input type="date" class="form-control" id="tgl_psp" name="tgl_psp" value="{{ $model['tgl_psp'] ?? '' }}"> -->
                                    @include('components.datepicker', [
                                        'value' => $model['tgl_psp'] ?? '',
                                        'label' => 'Tanggal PSP',
                                        'name' => 'tgl_psp',
                                        'withDisabled' => true,
                                    ])
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <!-- <label for="name" class="form-label">Tgl Rekam Pertama</label> -->
                                    <!-- <input type="date" data-date-format="DD-MM-YYYY" class="form-control" id="tgl_rekam_pertama" name="tgl_rekam_pertama" value="{{ $model['tgl_rekam_pertama'] ?? '' }}"> -->
                                    @include('components.datepicker', [
                                        'value' => $model['tgl_rekam_pertama'] ?? '',
                                        'label' => 'Tanggal Rekam',
                                        'name' => 'tgl_rekam_pertama',
                                        'withDisabled' => true,
                                    ])
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <!-- <label for="name" class="form-label">Tgl Perolehan</label> -->
                                    <!-- <input type="date" data-date-format="DD-MM-YYYY" class="form-control" id="tgl_perolehan" name="tgl_perolehan" value="{{ $model['tgl_perolehan'] ?? '' }}"> -->
                                    @include('components.datepicker', [
                                        'value' => $model['tanggal_perolehan'] ?? '',
                                        'label' => 'Tanggal Perolehan',
                                        'name' => 'tanggal_perolehan',
                                        'withDisabled' => true,
                                    ])
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-12">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Alamat</label>
                                    <input type="text" class="form-control" id="alamat" name="alamat" value="{{ $model['alamat'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">RT/RW</label>
                                    <input type="text" class="form-control" id="rt_rw" name="rt_rw" value="{{ $model['rt_rw'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Kelurahan/Desa</label>
                                    <input type="text" class="form-control" id="kelurahan" name="kelurahan" value="{{ $model['kelurahan_desa'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Kecamatan</label>
                                    <input type="text" class="form-control" id="kecamatan" name="kecamatan" value="{{ $model['kecamatan'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Kota/Kabupaten</label>
                                    <input type="text" class="form-control" id="kabkota" name="kabkota" value="{{ $model['uraian_kab_kota'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Kode Kab/Kota</label>
                                    <input type="text" class="form-control" id="kode_kabkota" name="kode_kabkota" value="{{ $model['kode_kab_kota'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Provinsi</label>
                                    <input type="text" class="form-control" id="provinsi" name="provinsi" value="{{ $model['uraian_provinsi'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Kode Provinsi</label>
                                    <input type="text" class="form-control" id="kode_prov" name="kode_prov" value="{{ $model['kode_provinsi'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Kode Pos</label>
                                    <input type="text" class="form-control" id="kode_pos" name="kode_pos" value="{{ $model['kode_pos'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Jumlah KIB</label>
                                    <input type="text" class="form-control angka" id="jml_kib" name="jml_kib" value="{{ $model['jumlah_kib'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">SBSK</label>
                                    <input type="text" class="form-control angka" id="sbsk" name="sbsk" value="{{ $model['sbsk'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">OPTIMALISASI</label>
                                    <input type="text" class="form-control angka" id="optimalisasi" name="optimalisasi" value="{{ $model['optimalisasi'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Status SBSN</label>
                                    <input type="text" class="form-control" id="status_sbsn" name="status_sbsn" value="{{ $model['status_sbsn'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>

    @include('monsakti.transaksi-aset.gridV', ['model' => $model])

    @include('monsakti.transaksi-aset.mapsV', ['model' => $model, 'jenis_aset' => 'asset_tanah'])

    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-body">
                    <div class="hstack gap-2">
                        <a href="{{ url('asset/tanah') }}" class="btn btn-outline-primary">Kembali</a>
                        @if(!$readOnly)
                        <button type="submit" class="btn btn-primary">
                            {{ $isNew ? 'Simpan' : 'Ubah' }}
                        </button>
                        @endif
                    </div>
                </div>
            </div>
        </div>
    </div>
@endsection

@section('js')
    <script>
        $(function() {
            var isReadOnly = '{{ $readOnly }}';
            if(isReadOnly){
                $('input, select').prop('readonly', true);
            }

            $('.angka').autoNumeric('init', {
                aSep : '.',
                aDec: ',',
                mDec: '0'
            });
        });

    </script>
@endsection
