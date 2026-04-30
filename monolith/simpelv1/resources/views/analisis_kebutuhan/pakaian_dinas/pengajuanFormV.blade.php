@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card ">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $isNew ? 'Tambah' : 'Edit' }} Pengajuan Pakaian Dinas</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body p-4">
                    <form action="{{ $controller }}" method="POST" class="ajaxForm">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="nama" class="form-label">Nama</label>
                                    <input class="form-control" id="nama" name="nama" placeholder="nama" required
                                        value="{{ $model['nama'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="deskripsi" class="form-label">Deskripsi</label>
                                    <textarea name="deskripsi" id="textarea" class="form-control" rows="5">{{ $model['deskripsi'] ?? '' }}</textarea>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="tahun" class="form-label">Tahun</label>
                                    <select class="form-control" data-choices data-choices-sorting-false
                                        id="tahun" name="tahun">
                                        <option value="">Pilih Tahun</option>
                                        {!! $tahuns !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="form-check form-switch form-check-right">
                                    <input class="form-check-input" type="checkbox" role="switch" name="is_reguler" id="is_reguler"
                                        value="0">
                                    <label class="form-check-label" for="flexSwitchCheckRightDisabled">Permintaan
                                        Cepat</label>
                                </div>
                            </div>
                        </div>
                        <div class="row" id="div-tanggal">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    @include('components.datepicker', [
                                        'value' => $model['tgl_mulai'] ?? '',
                                        'label' => 'Tanggal Mulai',
                                        'name' => 'tgl_mulai',
                                    ])
                                </div>
                            </div>
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    @include('components.datepicker', [
                                        'value' => $model['tgl_selesai'] ?? '',
                                        'label' => 'Tanggal Selesai',
                                        'name' => 'tgl_selesai',
                                    ])
                                </div>
                            </div>
                        </div>

                        <div class="row mt-3">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="pilihan_satker" class="form-label">Satker</label>
                                    <select class="form-control" data-choices data-choices-sorting-false id="pilihan_satker"
                                        name="pilihan_satker">
                                        <option value="semua"
                                            @php $selectedSatker = $model['pilihan_satker'] ?? null; @endphp
                                            {{ $selectedSatker == 'semua' ? 'selected' : '' }}>
                                            Semua Satker</option>
                                        <option value="sebagian" {{ $selectedSatker == 'sebagian' ? 'selected' : '' }}>
                                            Sebagian Satker</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row visually-hidden" id="div-pilihan-satker">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="satkers" class="form-label">Pilih Satker</label>
                                    <select class="form-control selectTwo text-black" multiple="multiple" name="satkers[]">
                                        {!! $satkerOptions !!}
                                    </select>
                                </div>
                                <div>
                                    <div class="form-check form-check-right">
                                        <input class="form-check-input" type="checkbox" value="1"
                                            {{ $model['dengan_unit_kerja'] ?? 0 == 1 ? 'checked' : '' }}
                                            name="dengan_unit_kerja" id="pilihan_satker_dengan_unit">
                                        <label class="form-check-label" for="pilihan_satker_dengan_unit">
                                            Dengan Unit Kerja
                                        </label>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="row mt-3">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="ms_jenis_pakaian_dinas_id" class="form-label">Pakaian</label>
                                    <select class="form-control" data-choices data-choices-sorting-false
                                        id="ms_jenis_pakaian_dinas_id" name="ms_jenis_pakaian_dinas_id">
                                        <option value="">Pilih Pakaian</option>
                                        {!! $jenisPakaianOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row mt-3">
                            <div class="col" id="spec-list">
                            </div>
                            <div class="row mt-3">
                                <div class="col-lg-12">
                                    <div class="hstack gap-2 justify-content-start">
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
    <script>
        const pakaianSpecs = {{ Js::from($specOptions) }}
        console.log(pakaianSpecs);

        function genereateCheckbox(msJenisPakaianId, isFirst = false) {
            const elm = pakaianSpecs
                .filter(spec => spec.ms_jenis_pakaian_id == msJenisPakaianId)
                .map(spec =>
                    `<div class="custom-control custom-checkbox">
                        <input type="checkbox"
                    ${isFirst && spec.checked ? 'checked' : ''}
                     class="custom-control-input" name="spesifikasi_id[]"
                             value="${spec.value}"
                            id="spec-${spec.value}" />
                        <label class="custom-control-label"
                            for="spec-${spec.value}">${spec.text}</label>
                    </div>`
                );
            if (elm.length < 1) return $('#spec-list').html(
                '<p>Spesifikasi Pakaian Tidak Ditemukan</p>');
            $('#spec-list').html(elm);
        }

        $(function() {
            $('.selectTwo').select2()
            genereateCheckbox(`{{ $model['ms_jenis_pakaian_dinas_id'] ?? '' }}`, true);

            $('#pilihan_satker').on('change', function() {
                const val = $(this).val();
                $('#div-pilihan-satker').toggleClass('visually-hidden', val == 'semua')
            })
            $('#pilihan_satker').trigger('change');

            $('#ms_jenis_pakaian_dinas_id').on('change', function() {
                const ms_jenis_pakaian_id = $(this).val();
                genereateCheckbox(ms_jenis_pakaian_id);
            })
            $('#is_reguler').on('change', function(){
                const isFast = $(this).attr('checked');
                $('#div-tanggal').toggleClass('visually-hidden', isFast);
            })

        })
    </script>
@endsection
