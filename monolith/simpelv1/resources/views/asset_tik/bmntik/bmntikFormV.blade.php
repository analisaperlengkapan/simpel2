@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }} Asset TIK</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/asset/tik" method="POST" class="ajaxForm">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Kode Satker *</label>
                                    <select class="form-control" data-choices data-choices-sorting-false name="kdsatker_keu" id="kdsatker_keu">
                                        <option value="">Pilih Satker</option>
                                        {!! $satkerOptions !!}
                                    </select>
                                </div>
                            </div>
                            <!-- <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nama Satker *</label>
                                    <input type="text" class="form-control" id="nm_satker" name="nm_satker" required value="{{ $model['nm_satker'] ?? '' }}">
                                </div>
                            </div> -->
                        </div>
                        <div class="row">
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Kode Barang *</label>
                                    <input class="form-control" id="kode_barang" name="kode_barang" required value="{{ $model['kode_barang'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nama Barang *</label>
                                    <input type="text" class="form-control" id="nm_barang" name="nm_barang" required value="{{ $model['nm_barang'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">NUP</label>
                                    <input type="number" class="form-control" id="nup" name="nup" value="{{ $model['nup'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Kondisi</label>
                                    <select class="form-control" data-choices data-choices-search-false name="kondisi" id="kondisi">
                                        <option value="">Pilih Kondisi</option>
                                        {!! $kondisiOptions !!}
                                    </select>
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Merk/Tipe</label>
                                    <input type="text" class="form-control" id="merk" name="merk" value="{{ $model['merk'] ?? '' }}">
                                </div>
                            </div>
                        <div class="row">
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Tgl Rekam Pertama</label>
                                    <input type="date" class="form-control" id="tgl_rekam_pertama" name="tgl_rekam_pertama" value="{{ $model['tgl_rekam_pertama'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Tgl Perolehan</label>
                                    <input type="date" class="form-control" id="tgl_perolehan" name="tgl_perolehan" value="{{ $model['tgl_perolehan'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nilai Perolehan Pertama</label>
                                    <input type="number" class="form-control" id="nilai_perolehan_pertama" name="nilai_perolehan_pertama" value="{{ $model['nilai_perolehan_pertama'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nilai Mutasi</label>
                                    <input type="number" class="form-control" id="nilai_mutasi" name="nilai_mutasi" value="{{ $model['nilai_mutasi'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nilai Perolehan</label>
                                    <input type="number" class="form-control" id="nilai_perolehan" name="nilai_perolehan" value="{{ $model['nilai_perolehan'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nilai Penyusutan</label>
                                    <input type="number" class="form-control" id="nilai_penyusutan" name="nilai_penyusutan" value="{{ $model['nilai_penyusutan'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nilai Buku</label>
                                    <input type="number" class="form-control" id="nilai_buku" name="nilai_buku" value="{{ $model['nilai_buku'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Kuantitas</label>
                                    <input type="number" class="form-control" id="kuantitas" name="kuantitas" value="{{ $model['kuantitas'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Jumlah Foto</label>
                                    <input type="number" class="form-control" id="jml_foto" name="jml_foto" value="{{ $model['jml_foto'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Status Penggunaan</label>
                                    <input type="text" class="form-control" id="status_penggunaan" name="status_penggunaan" value="{{ $model['status_penggunaan'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Status Pengelolaan</label>
                                    <input type="text" class="form-control" id="status_pengelolaan" name="status_pengelolaan" value="{{ $model['status_pengelolaan'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">No. PSP</label>
                                    <input type="text" class="form-control" id="no_psp" name="no_psp" value="{{ $model['no_psp'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Tanggal PSP</label>
                                    <input type="date" class="form-control" id="tgl_psp" name="tgl_psp" value="{{ $model['tgl_psp'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <hr/>
                                <div class="hstack gap-2">
                                    <a href="{{ url('asset/tik') }}" class="btn btn-outline-primary">Kembali</a>
                                    @if(!$readOnly)
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
            var isReadOnly = '{{ $readOnly }}';
            if(isReadOnly){
                $('input, select').prop('readonly', true);
            }
        })
    </script>
@endsection
