@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }}</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/sdm/sdmpengelola" method="POST" class="ajaxForm" enctype="multipart/form-data">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="peg_nip_baru" name="peg_nip_baru" value="{{ $model['peg_nip_baru'] }}">
                        @endif
                        <div class="row">
                            {{--
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Kode Satker *</label>
                                    <select class="form-control" data-choices data-choices-sorting-false name="kdsatker_keu" id="kdsatker_keu">
                                        <option value="">Pilih Satker</option>
                                        {!! $satkerOptions !!}
                                    </select>
                                </div>
                            </div>
                            --}}

                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Satker </label>
                                    <input type="text" class="form-control" id="satker" name="satker" value="{{ $model['inst_nama'] ?? '' }}" disabled>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nama</label>
                                    <input type="hidden" id="isNew" name="isNew" value="{{ $isNew }}">
                                    <input type="text" class="form-control" id="" name="" value="{{ $model['nama'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">NIP</label>
                                    <input type="text" class="form-control" id="" name="" value="{{ $model['peg_nip_baru'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Pangkat</label>
                                    <input type="text" class="form-control" id="" name="" value="{{ $model['pangkat'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row" >
                            <div class="col-lg-8">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Jabatan</label>
                                    <input type="text" class="form-control" id="" name="" value="{{ $model['jabatan'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Unit Kerja</label>
                                    <input type="text" class="form-control" id="" name="" value="{{ $model['unitkerja_nama'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <hr/>

                        <!-- <div class="row" >
                            <div class=" col-lg-2">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Tanggal Sertifikat *</label>
                                    <input type="date" class="form-control" id="" name="" value="{{ $model['tgl_sertifikat_pengadaan'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">File Sertifikat *</label>
                                    @if(isset($model['file_sertifikat_pengadaan']))
                                    <a href="{{ url($model['file_sertifikat_pengadaan']) }}" download terget="_blank">
                                        <i class="ri-download-cloud-line"></i> Download File
                                    </a>
                                    @else
                                        <br/><i>Tidak Ada File Sertifikat</i>
                                    @endif
                                </div>
                            </div>
                        </div> -->
                        
                        <!-- <hr/> -->
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2">
                                    <a href="{{ url('sdm/sdmpengelola') }}" class="btn btn-outline-primary">Kembali</a>
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
