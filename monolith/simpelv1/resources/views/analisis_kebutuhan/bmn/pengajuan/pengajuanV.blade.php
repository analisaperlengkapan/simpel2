@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
<div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Pengajuan Kebutuhan BMN</h5>
                        </div>
                        @if ($operasi == 'CREATE')
                            <div class="flex-shrink-0">
                                <a href="{{ $controller . '/create' }}" type="button"
                                    class="btn btn-success btn-label waves-effect waves-light"><i
                                        class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                    Tambah
                                </a>
                            </div>
                        @endif
                    </div>
                </div>
                <div class="card-body">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt"
                        style="width:100%">
                        <thead>
                            <tr>
                                <th>Nama Permintaan @include('components.dtFilterInput',['index' => 0,'column' => 'Nama'])</th>
                                <th>Deskripsi @include('components.dtFilterInput',['index' => 1,'column' => 'Deskripsi'])</th>
                                <th>Tahun Anggaran @include('components.dtFilterInput',['index' => 2,'column' => 'Tahun'])</th>
                                <th>Periode</th>
                                <th>Aksi</th>
                            </tr>
                        </thead>
                    </table>
                </div>
            </div>
        </div>
    </div>
<style>
    #{{ $tableId }} thead th {
        background-color: #405189;
        color: #ffffff;
        text-align: center;
        text-transform: uppercase;
    }
</style>
@endsection

@section('js')
    <script>
        const tableId = `{{ $tableId }}`;
        $(function() {
            const dt = $('#' + tableId).DataTable({
                serverSide: true,
                processing: true,
                deferRender: true,
                ordering: false,
                dom: dtLayout,
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ $controller . '/gridData' }}",
                    dataSrc: 'data',
                },
                columns: [{
                        data: 'nama'
                    },
                    {
                        data: 'deskripsi'
                    },
                    {
                        data: 'tahun',
                    },
                    {"data": function (row, data, index, display) {
                        return dateFormatIndo(row.tgl_mulai)+' s.d '+dateFormatIndo(row.tgl_selesai);
                    }},
                    {
                        data: 'id',
                        render: (data, type, row) => {
                            const url = `{{ $controller . '/' . '${data}' }}`;
                            @if ($operasi == 'CREATE')
                            return `
                                <div class="d-flex justify-content-center gap-2">
                                    @include('components.updateBtn', [
                                        'url' => '${url}',
                                    ])
                                    <a href="${url+'/list-satker'}" type="button" class="btn btn-success waves-effect waves-light" title="Approve"><i class="ri-checkbox-line"></i></a>
                                    @include('components.deleteBtn', [
                                        'url' => '${url}',
                                        'title' => '${row.nama}',
                                        'tableId' => '${tableId}',
                                    ])
                                </div>`;
                            @else
                            return `
                                <div class="d-flex justify-content-center gap-2">
                                    <a href="${url+'/list-satker'}" type="button" class="btn btn-success waves-effect waves-light" title="Approve"><i class="ri-checkbox-line"></i></a>
                                </div>`;
                            @endif
                        },
                    }
                ]
            })
        })
    </script>
@endsection
