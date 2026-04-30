@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }} Daftar Langganan Jasa Khusus TIK</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/asset-tik/langganan" method="POST" class="ajaxForm">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <!-- <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Kode Satker *</label>
                                    <select class="form-control" data-choices data-choices-sorting-false name="kdsatker_keu" id="kdsatker_keu">
                                        <option value="">Pilih Satker</option>
                                        {!! $satkerOptions !!}
                                    </select>
                                </div>
                            </div> -->
                            <!-- <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nama Satker *</label>
                                    <input type="text" class="form-control" id="nm_satker" name="nm_satker" required value="{{ $model['nm_satker'] ?? '' }}">
                                </div>
                            </div> -->
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Satker </label>
                                    <input type="text" class="form-control" id="satker" name="satker" value="{{ $model['inst_nama'] ?? '' }}" disabled>
                                    <input type="hidden" class="form-control" id="kdsatker_keu" name="kdsatker_keu" value="{{ $model['kdsatker_keu'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nama Layanan *</label>
                                    <input type="text" class="form-control" id="nm_layanan" name="nm_layanan" required value="{{ $model['nm_layanan'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nama Penyedia *</label>
                                    <input type="text" class="form-control" id="nm_perusahaan" name="nm_perusahaan" required value="{{ $model['nm_perusahaan'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Tgl Mulai Langganan *</label>
                                    <input type="date" class="form-control" id="tgl_mulai" required name="tgl_mulai" value="{{ $model['tgl_mulai'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Tgl Selesai Langganan *</label>
                                    <input type="date" class="form-control" id="tgl_selesai" required name="tgl_selesai" value="{{ $model['tgl_selesai'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Tipe Beli *</label>
                                    <select class="form-control" data-choices data-choices-sorting-false name="tipe_beli" id="tipe_beli">
                                        <option value="">Pilih Tipe Beli</option>
                                        {!! $tipebeliOptions !!}
                                    </select>
                                </div>
                            </div>
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nilai</label>
                                    <input type="text" class="form-control angka" id="nilai" name="nilai" required value="{{ $model['nilai'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-12">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Deskripsi</label>
                                    <input type="text" class="form-control" id="deskripsi" name="deskripsi" required value="{{ $model['deskripsi'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row" >
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">File Invoice *</label>
                                    <input type="file" class="form-control" id="file_invoice" name="file_invoice" value="{{ $model['file_invoice'] ?? '' }}">
                                    @if(isset($model['file_invoice']))
                                    <a href="{{ url($model['file_invoice']) }}" download terget="_blank">
                                        <i class="ri-download-cloud-line"></i> Download File
                                    </a>
                                    @endif
                                </div>
                            </div>
                        </div>
                        
                        <div class="row">
                            <div class="col-lg-12">
                                <hr/>
                                <div class="hstack gap-2">
                                    <a href="{{ url('asset-tik/langganan') }}" class="btn btn-outline-primary">Kembali</a>
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

            $('.angka').autoNumeric('init', {
                aSep : '.',
                aDec: ',',
                mDec: '0'
            });
        })
    </script>
@endsection
