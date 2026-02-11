@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Klaim</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">

                    <form action="{{ $controller }}" method="POST" class="ajaxForm">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="asuransi_transaksi_id" class="form-label">Polis</label>
                                    <select class="form-control" data-choices data-choices-search-false
                                        name="asuransi_transaksi_id" id="asuransi_transaksi_id">
                                        <option value="" selected>Pilih Polis</option>
                                        {!! $polisOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>

                        <div class="row">
                            <div class=" col-lg-4">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nomor Surat</label>
                                    <input type="text" class="form-control" id="surat_no" name="surat_no"
                                        value="{{ $model['surat_no'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    @include('components.datepicker', [
                                        'value' => $model['surat_tgl'] ?? '',
                                        'label' => 'Tanggal Surat',
                                        'name' => 'surat_tgl',
                                    ])
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Lampiran</label>
                                    <input type="file" class="form-control" id="filename" name="filename"
                                        value="{{ $model['filename'] ?? '' }}">
                                    @if (isset($model['filename']))
                                        <a href="{{ asset($model['filename']) }}" download terget="_blank">
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
                                    <button type="submit" class="btn btn-primary">
                                        {{ $isNew ? 'Simpan' : 'Ubah' }}
                                    </button>
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
    <script></script>
@endsection
