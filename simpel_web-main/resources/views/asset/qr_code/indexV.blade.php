@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Setting QR Code</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form>
                        <div class="row">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="satkers" class="form-label">Logo</label>
                                    <input type="file" id="logo" name="logo" class="form-control" />
                                    <a href="{{ url($qrcode['logo']) }}" download terget="_blank"><i
                                            class="ri-download-cloud-line"></i></a>
                                </div>
                                <button class="btn btn-primary" type="button" id="simpan">Simpan</button>
                            </div>
                        </div>
                    </form>
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
                            <h5 class="card-title mb-0">Cetak Label</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form>
                        <div class="row">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="satkers" class="form-label">Satker</label>
                                    <select class="form-control" id='satker' data-choices>
                                        {{-- <select class="form-control" id='satker' data-choices {{ ($userOperation == 'SATKER') ? "data-choices-text-disabled-true" : "" }} > --}}
                                        <option value="">Pilih Satker</option>
                                        {!! $satkerOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="jenis_asset" class="form-label">Aset</label>
                                    <select class="form-control" data-choices name="id_jenis_asset" id="id_jenis_asset">
                                        <option value="">Pilih Aset</option>
                                        {!! $jenisAssetOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row mt-3" style="margin-bottom: 30px;">
                            <div class="col-lg-12">
                                <div class="justify-content-start">
                                    <button type="button" id="btn-cari" class="btn btn-primary">Cari</button>
                                </div>
                            </div>
                        </div>
                    </form>
                    <hr />
                    <button type="button" id="btn-cetak" class="btn btn-primary">Cetak Label</button>
                    <form id="form-cetak-label" action="{{ URL::to($controller . '/cetakLabel') }}" method="POST" target="_blank" style="display:none;">
                        @csrf
                        <input type="hidden" name="inst_satkerkd" id="form-inst_satkerkd">
                        <input type="hidden" name="id_jenis_asset" id="form-id_jenis_asset">
                        <input type="hidden" name="id" id="form-id">
                    </form>
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive" style="width:100%">
                        <thead>
                            <tr>
                                @foreach ($columns as $index => $column)
                                    <th>
                                        {{ $column }}
                                        @include('components.dtFilterInput', [
                                            'index' => $index,
                                            'column' => $column,
                                        ])
                                    </th>
                                @endforeach
                                <th>Pilih <input type='checkbox' id='pilih-all' /></th>
                            </tr>
                        </thead>
                    </table>
                </div>
            </div>
        </div>
        <!--end col-->
    </div>
    <style>
        #{{ $tableId }} thead th {
            background-color: #405189;
            color: #ffffff;
            text-align: center;
            text-transform: uppercase;
        }

        .dataTables_length {
            width: auto;
            float: right;
        }
    </style>
@endsection

@section('js')
    <script>
        const tableId = `{{ $tableId }}`;
        $('#' + tableId + ' thead th').css({
            "background-color": "#405189",
            "color": "#ffffff",
            "text-align": "center",
            "text-transform": "uppercase"
        });
        $(function() {
            const dt = $('#' + tableId).DataTable({
                processing: true,
                serverSide: true,
                ordering: false,
                "deferRender": true,
                dom: "ltipr",
                lengthMenu: [
                    [10, 25, 50, 100, -1],
                    [10, 25, 50, 100, 'Semua']
                ],
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ URL::to($controller . '/gridData') }}" + '?id_jenis_asset=&inst_satkerkd=',
                    dataSrc: 'data',
                },
                responsive: false,
                scrollX: true,
                columns: [{
                        data: 'kdsatker_keu',
                        searchable: true
                    },
                    {
                        data: 'deskripsi',
                        searchable: true
                    },
                    {
                        data: 'kode_barang',
                        searchable: true
                    },
                    {
                        data: 'nm_barang',
                        searchable: true
                    },
                    {
                        data: 'nup',
                        searchable: false
                    },
                    {
                        data: 'kondisi',
                        searchable: true
                    },
                    {
                        data: 'merk',
                        searchable: false
                    },
                    {
                        data: 'id',
                        render: (data, type, row) => {
                            const url = `{{ url('/asset/bangunan_air/${data}') }}`;
                            const urlCetak = `{{ url($controller . '/cetakLabel/${data}') }}`;
                            return "<input type='checkbox' value='" + data + "' class='pilih'/>";
                        },
                        searchable: false
                    }
                ]

            });

            $('#btn-cari').on('click', function() {
                const satker = $('#satker').val();
                const id_jenis_asset = $('#id_jenis_asset').val();
                dt.ajax.url("{{ URL::to($controller . '/gridData') }}?id_jenis_asset=" + id_jenis_asset +
                    "&inst_satkerkd=" + satker).load()
            });

            $('#pilih-all').on('change', function() {
                if (this.checked) {
                    $('#' + tableId).find('.pilih').prop('checked', true);
                } else {
                    $('#' + tableId).find('.pilih').prop('checked', false);
                }
            });

            $('#btn-cetak').on('click', function() {
                var arrPilih = [];
                $('#' + tableId).find('.pilih:checkbox:checked').each(function() {
                    arrPilih.push($(this).val());
                });
                const satker = $('#satker').val();
                const id_jenis_asset = $('#id_jenis_asset').val();
                if(arrPilih.length === 0) {
                    alert('Pilih minimal satu aset untuk dicetak!');
                    return;
                }
                $('#form-inst_satkerkd').val(satker);
                $('#form-id_jenis_asset').val(id_jenis_asset);
                $('#form-id').val(arrPilih.join(","));
                $('#form-cetak-label').submit();
            });

            $('#simpan').on('click', function() {
                var data = new FormData();
                var files = $('#logo')[0].files;
                data.append('logo', files[0]);
                $.ajax({
                    method: "POST",
                    url: `{{ $controller }}`,
                    data: data,
                    processData: false,
                    contentType: false,
                    success: function() {
                        notify({
                            type: "success",
                            message: "Data Berhasil Disimpan",
                        });
                    },
                    error: showError,
                }).done(function(msg) {});
            });
        })
    </script>
@endsection
