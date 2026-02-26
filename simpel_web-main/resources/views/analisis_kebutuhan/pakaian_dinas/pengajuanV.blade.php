@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Pengajuan Pakaian Dinas Pegawai</h5>
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
                    @include('components.dtFilterBox', [
                        'columns' => $columns,
                        'selected' => $defColumns,
                    ])
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt"
                        style="width:100%">
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
                                <th>Aksi</th>
                            </tr>
                        </thead>
                    </table>
                </div>
            </div>
        </div>
    </div>
@endsection

@section('js')
    <script>
        const tableId = `{{ $tableId }}`;
        const defColumn = @json($defColumns);
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
                        data: 'nama',
                        className: 'text-center',
                    },
                    {
                        data: 'deskripsi',
                        className: 'text-center',
                    },
                    {
                        data: 'tgl_mulai',
                        className: 'text-center',
                        render: (data, type, row) =>
                            `${dateFormatIndo(row.tgl_mulai)}`
                    },
                    {
                        data: 'tgl_selesai',
                        className: 'text-center',
                        render: (data, type, row) =>
                            `${dateFormatIndo(row.tgl_selesai)}`
                    },
                    {
                        data: 'id',
                        render: (data, type, row) => {
                            const url = `{{ $controller . '/' . '${data}' }}`;
                            if (`{{ $operasi }}` == 'CREATE') {
                                return `
                                <div class="d-flex justify-content-center gap-2">
                                    @include('components.updateBtn', [
                                        'url' => '${url}',
                                    ])
                                    @include('components.showBtn', [
                                        'url' => '${url}/list-satker',
                                    ])
                                    @include('components.deleteBtn', [
                                        'url' => '${url}',
                                        'title' => '${row.nama}',
                                        'tableId' => '${tableId}',
                                    ])
                                </div>`;
                            }

                            if (`{{ $operasi }}` == 'INPUT') {
                                return `
                                <div class="d-flex justify-content-center gap-2">
                                    @include('components.updateBtn', [
                                        'url' => '${url}/edit',
                                    ])
                                </div>`;
                            }

                            if (`{{ $operasi }}` == 'VIEW') {
                                return `
                                <div class="d-flex justify-content-center gap-2">
                                    @include('components.showBtn', [
                                        'url' => '${url}/list-satker',
                                    ])
                                </div>`;
                            }
                        }
                    }
                ]
            })
        })
    </script>
@endsection
