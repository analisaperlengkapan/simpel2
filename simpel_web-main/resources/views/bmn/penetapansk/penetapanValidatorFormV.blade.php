@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <form action="/bmn/penetapan/penetapansk" method="POST" class="ajaxForm" enctype="multipart/form-data">
        <div class="row">
            <div class="col-lg-12">
                <div class="card">
                    <div class="card-header">
                        <div class="d-flex align-items-center">
                            <div class="flex-grow-1">
                                <h5 class="card-title mb-0">{{ $judul }} Pengajuan</h5>
                            </div>
                        </div>
                    </div>
                    <div class="card-body">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row" id="div-tanggal">
                            <div class=" col-lg-5">
                                <div class="mb-3">
                                    <label for="sp_no" class="form-label">Nomor Surat</label>
                                    <input class="form-control" disabled id="sp_no" name="sp_no"
                                        placeholder="Nomor Surat" value="{{ $model['sp_no'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="sp_tgl" class="form-label">Tanggal Surat</label>
                                    <input class="form-control" disabled id="sp_tgl" name="sp_tgl"
                                        placeholder="Nomor Surat" value="{{ MyHelper::dateFormatIndo($model['sp_tgl']) }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label class="form-label"> Surat Pernyataan Tanggung Jawab Mutlak</label>
                                    <br>
                                    @if (isset($model['sp_file']))
                                        <a href="{{ asset($model['sp_file']) }}" download terget="_blank">
                                            <i class="ri-download-cloud-line"></i> Download File
                                        </a>
                                    @endif
                                </div>
                            </div>
                        </div>

                    </div>
                </div>
                <div class="col-lg-12">
                    <div class="card">
                        <div class="card-header">
                            <div class="d-flex align-items-center">
                                <div class="flex-grow-1">
                                    <h5 class="card-title mb-0">Asset</h5>
                                </div>
                            </div>
                        </div>
                        <div class="card-body">
                            <table class="table align-middle table-nowrap mb-0 my-dt" id="table-modal">
                                <thead class="table-light">
                                    <tr>
                                        <th scope="row">#</th>
                                        <th scope="col">Asset</th>
                                        <th scope="col">Barang</th>
                                    </tr>
                                </thead>
                                <tbody>

                                </tbody>
                                @foreach ($assets as $asset)
                                    <tr id="assetRow-{{ $asset->id }}">
                                        <td>{{ $loop->index + 1 }}</td>
                                        <td>{{ $asset->asset_nama }}</td>
                                        <td>{{ $asset->barang_nama }}</td>
                                    </tr>
                                @endforeach
                            </table>
                            <hr>

                        </div>
                    </div>
                </div>
                @if (in_array($aktifitas['id'], [2002]))
                    <div class="col-lg-12">
                        <div class="card">
                            <div class="card-header">
                                <div class="d-flex align-items-center">
                                    <div class="flex-grow-1">
                                        <h5 class="card-title mb-0">SK PSP</h5>
                                    </div>
                                </div>
                            </div>
                            <div class="card-body">
                                <div class="row">
                                    <div class=" col-lg-5">
                                        <div class="mb-3">
                                            <label for="sk_no" class="form-label">Nomor SK</label>
                                            <input class="form-control" id="sk_no" name="sk_no"
                                                placeholder="Nomor Surat" value="{{ $model['sk_no'] ?? '' }}" />
                                        </div>
                                    </div>
                                    <div class=" col-lg-3">
                                        <div class="mb-3">
                                            @include('components.datepicker', [
                                                'value' => $model['sk_tgl'] ?? '',
                                                'label' => 'Tanggal SK',
                                                'name' => 'sk_tgl',
                                            ])
                                        </div>
                                    </div>
                                </div>
                                <div class="row">
                                    <div class="col-lg-6">
                                        <div class="mb-3">
                                            <label for="sk_file" class="form-label">SK PSP</label>
                                            <input type="file" class="form-control" id="sk_file" name="sk_file"
                                                value="{{ $model['sk_file'] ?? '' }}">
                                            @if (isset($model['sk_file']))
                                                <a href="{{ asset($model['sk_file']) }}" download terget="_blank">
                                                    <i class="ri-download-cloud-line"></i> Download File
                                                </a>
                                            @endif
                                        </div>
                                    </div>
                                </div>

                            </div>
                        </div>
                    </div>
                @endif

                @include('components.activityForm', [$aktifitasHistories, $aktifitas, $aktifitasOptions])
                <div class="row" style="margin-bottom: 10px">
                    <div class="col-lg-12">
                        <div class="hstack gap-2 justify-content-left">
                            <a href="{{ $controller }}" class="btn btn-outline-primary">Kembali</a>
                            <button type="submit" class="btn btn-primary">Simpan</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </form>

    <div id="formModal" class="modal fade zoomIn" tabindex="-1" aria-labelledby="formModal" aria-hidden="true"
        style="display: none;">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title">Tambah Asset</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form id="formnyaModal">
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <input type="hidden" name="asset_nama" id="asset_nama" />
                                    <label for="asset_kode" class="form-label">Jenis Asset</label>
                                    <select class="form-control" data-choices data-choices-search-false id="asset_kode"
                                        name="asset_kode">
                                        <option value="" selected disabled>Pilih Asset</option>
                                        {!! $assetOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <input type="hidden" name="barang_nama" id="barang_nama" />
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="barang_kode" class="form-label">Barang</label>
                                    <select class="form-control" id="barang_kode" name="barang_kode">
                                        <option value="" selected>Pilih Barang</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-light" data-bs-dismiss="modal" onc>Tutup</button>
                    <button type="button" id="assetAdd" class="btn btn-primary ">Tambah</button>
                </div>

            </div>
        </div>
    </div>
@endsection

@section('js')
    <script>
        function deleteRow(id) {
            $(`#assetRow-${id}`).remove();
        }

        function appendRow({
            no,
            id,
            data,
        }) {

            let elm = `
                <tr id="assetRow-${id}">
                    <td>${no}</td>
                    <td>${data.asset_nama}</td>
                    <td>${data.barang_nama}</td>
                    <td>
                        <div class="d-flex justify-content-center gap-2">
                            <button type="button" class="btn btn-danger waves-effect waves-light" onClick="deleteRow('${id}')">
                                <i class="ri-delete-bin-5-fill"></i>
                            </button>
                        </div>
                        <div>`;

            for (const key in data) {
                const value = data[key];
                elm += `<input type="hidden" name="assets[${id}][${key}]" value="${value}">`
            }
            elm += `
                        </div>
                    </td>
                </tr>`;

            $('#table-modal tbody').append(elm);
            $('#formModal').modal('hide')
        }

        let inputtedAssets = [];

        $(function() {
            var isReadOnly = '{{ $readOnly }}';
            // if (isReadOnly) {
            //     $('input, select').prop('readonly', true);
            // }

            $('#barang_kode').select2({
                dropdownParent: $('#formModal')
            });
            $('#asset_kode').on('change', function() {
                const id = $(this).val();
                const text = $(this).find('option:selected').text();
                $('#asset_nama').val(text);

                $.get(`{{ $controller }}/getBarang/${id}`).done(res => {
                    if (res.length < 1) {
                        notify({
                            message: 'Tidak ada Barang!',
                            type: 'danger'
                        });
                        return false;
                    }
                    $('#barang_kode').empty().select2({
                        data: res,
                        dropdownParent: $('#formModal')
                    }).trigger('change');
                });

            })
            $('#barang_kode').on('change', function() {
                const text = $(this).find('option:selected').text();
                $('#barang_nama').val(text);
            })
            $('#assetAdd').on('click', function() {
                const data = $('#formnyaModal').serializeFormJSON();
                if (data.barang_kode == '') return;
                const exists = inputtedAssets.find(inputted => inputted.barang_kode == data.barang_kode);
                if (exists) {
                    notify({
                        message: 'Barang Sudah Terinput!',
                        type: 'danger'
                    });
                    return false;
                }
                const no = $('#table-modal tbody>tr').length + 1;
                appendRow({
                    data,
                    no,
                    id: shortId()
                })

            });
        })
    </script>
@endsection
