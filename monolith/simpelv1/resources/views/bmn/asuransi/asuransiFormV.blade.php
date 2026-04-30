@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }} Aset Asuransi</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body" id="div-aset">
                    <div class="row">
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Nama Satker *</label>
                                <input class="form-control" id="nm_satker" name="nm_satker" required
                                    value="{{ $model['nm_satker'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Kode Barang *</label>
                                <input class="form-control" id="kode_barang" name="kode_barang" required
                                    value="{{ $model['kode_barang'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-6">
                            <div class="mb-3">
                                <label for="name" class="form-label">Nama Barang *</label>
                                <input type="text" class="form-control" id="nm_barang" name="nm_barang" required
                                    value="{{ $model['nm_barang'] ?? '' }}">
                            </div>
                        </div>
                    </div>

                    <div class="row">
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="nip" class="form-label">NUP</label>
                                <input type="number" class="form-control" id="nup" name="nup"
                                    value="{{ $model['nup'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-3">
                            <div class="mb-3">
                                <label for="name" class="form-label">Kondisi</label>
                                <input class="form-control" id="kondisi" name="kondisi" required
                                    value="{{ $model['kondisi'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-3">
                            <div class="mb-3">
                                <label for="name" class="form-label">Jenis Dokumen</label>
                                <input type="text" class="form-control" id="jenis_dokumen" name="jenis_dokumen"
                                    value="{{ $model['jenis_dokumen'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-3">
                            <div class="mb-3">
                                <label for="name" class="form-label">Merk/Tipe</label>
                                <input type="text" class="form-control" id="merk" name="merk"
                                    value="{{ $model['merk'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class=" col-lg-3">
                            <div class="mb-3">
                                <label for="name" class="form-label">Tgl Rekam Pertama</label>
                                <input type="date" class="form-control" id="tgl_rekam_pertama" name="tgl_rekam_pertama"
                                    value="{{ $model['tgl_rekam_pertama'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-3">
                            <div class="mb-3">
                                <label for="name" class="form-label">Tgl Perolehan</label>
                                <input type="date" class="form-control" id="tgl_perolehan" name="tgl_perolehan"
                                    value="{{ $model['tgl_perolehan'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class=" col-lg-3">
                            <div class="mb-3">
                                <label for="name" class="form-label">Nilai Perolehan Pertama</label>
                                <input type="text" class="form-control angka" id="nilai_perolehan_pertama"
                                    name="nilai_perolehan_pertama" value="{{ $model['nilai_perolehan_pertama'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-3">
                            <div class="mb-3">
                                <label for="name" class="form-label">Nilai Mutasi</label>
                                <input type="text" class="form-control angka" id="nilai_mutasi" name="nilai_mutasi"
                                    value="{{ $model['nilai_mutasi'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-3">
                            <div class="mb-3">
                                <label for="name" class="form-label">Nilai Perolehan</label>
                                <input type="text" class="form-control angka" id="nilai_perolehan"
                                    name="nilai_perolehan" value="{{ (float) $model['nilai_perolehan'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-3">
                            <div class="mb-3">
                                <label for="name" class="form-label">Nilai Penyusutan</label>
                                <input type="text" class="form-control angka" id="nilai_penyusutan"
                                    name="nilai_penyusutan" value="{{ $model['nilai_penyusutan'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class=" col-lg-3">
                            <div class="mb-3">
                                <label for="name" class="form-label">Nilai Buku</label>
                                <input type="text" class="form-control" id="nilai_buku" name="nilai_buku"
                                    value="{{ $model['nilai_buku'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-3">
                            <div class="mb-3">
                                <label for="name" class="form-label">Kuantitas</label>
                                <input type="text" class="form-control" id="kuantitas" name="kuantitas"
                                    value="{{ $model['kuantitas'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-3">
                            <div class="mb-3">
                                <label for="name" class="form-label">Luas Bangunan</label>
                                <input type="text" class="form-control" id="luas_bangunan" name="luas_bangunan"
                                    value="{{ $model['luas_bangunan'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-3">
                            <div class="mb-3">
                                <label for="name" class="form-label">Luas Dasar Bangunan</label>
                                <input type="text" class="form-control" id="luas_dasar_bangunan"
                                    name="luas_dasar_bangunan" value="{{ $model['luas_dasar_bangunan'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class=" col-lg-12">
                            <div class="mb-3">
                                <label for="name" class="form-label">Nama KPNKNL</label>
                                <input type="text" class="form-control" id="nama_kpknl" name="nama_kpknl"
                                    value="{{ $model['nama_kpknl'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-12">
                            <div class="mb-3">
                                <label for="name" class="form-label">Alamat</label>
                                <input type="text" class="form-control" id="jalan" name="jalan"
                                    value="{{ $model['jalan'] ?? '' }}">
                            </div>
                        </div>
                    </div>

                </div>
            </div>
        </div>
    </div>

    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0"> Asuransi</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body" id="div-asuransi">

                    <form action="{{ $controller }}" method="POST" class="ajaxForm">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $asuransi['id'] ?? '' }}">
                        @endif

                        <input type="hidden" name="id_asset" value="{{ $model['id'] }}">
                        <input type="hidden" name="kode_barang" value="{{ $model['kode_barang'] }}">
                        <input type="hidden" name="nup" value="{{ $model['nup'] }}">
                        <div class="row">
                            <div class=" col-lg-4">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nomor Polis</label>
                                    <input type="text" class="form-control" id="polis_no" name="polis_no"
                                        value="{{ $asuransi['polis_no'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    @include('components.datepicker', [
                                        'value' => $asuransi['polis_tgl'] ?? '',
                                        'label' => 'Tanggal Polis',
                                        'name' => 'polis_tgl',
                                    ])
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-4">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nilai Premi</label>
                                    <input type="text" class="form-control angka" id="polis_premi" name="polis_premi"
                                        value="{{ $asuransi['polis_premi'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Lampiran</label>
                                    <input type="file" class="form-control" id="filename" name="filename"
                                        value="{{ $asuransi['filename'] ?? '' }}">
                                    @if (isset($asuransi['filename']))
                                        <a href="{{ asset($asuransi['filename']) }}" download terget="_blank">
                                            <i class="ri-download-cloud-line"></i> Download File
                                        </a>
                                    @endif
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2">
                                    <a href="{{ url($controller) }}" class="btn btn-outline-primary">Kembali</a>
                                    @if ($canChange)
                                        <button type="submit" class="btn btn-primary">
                                            {{ $isNew ? 'Simpan' : 'Ubah' }}
                                        </button>
                                    @endif
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>
@endsection

@section('js')
    <script>
        $(function() {
            $('#div-aset input').attr('disabled', true);
            $('#div-asuransi input').attr('disabled', `{{ $canChange }}` == 0);

            $('.angka').autoNumeric('init', {
                aSep: '.',
                aDec: ',',
                mDec: '0'
            });
        })
    </script>
@endsection
