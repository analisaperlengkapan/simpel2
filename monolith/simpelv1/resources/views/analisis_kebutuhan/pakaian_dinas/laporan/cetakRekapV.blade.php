@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Cetak Rekap Pengajuan Pakaian Dinas</h5>
                        </div>
                    </div>
                    <div class="row mt-3">
                        @include('components.dtSearchBox', [
                            'tableId' => $tableId,
                        ])
                    </div>
                </div>
                <div class="card-body">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt"
                        style="width:100%">
                        <thead>
                            <tr>
                                <th>Nama</th>
                                <th>Deskripsi</th>
                                <th>Periode</th>
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
        $(function() {
            const tableId = `{{ $tableId }}`;
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
                    data: (data) => {
                        const filterType = $('#dt-filter-by').data('filterby');
                        if (filterType?.trim() !== '') {
                            data.filterBy = filterType;
                        }
                    }
                },
                columns: [{
                        data: 'nama'
                    },
                    {
                        data: 'deskripsi'
                    },
                    {
                        data: 'tgl_mulai',
                        className: 'text-center',
                        render: (data, type, row) =>
                            `${dateFormat(row.tgl_mulai)} S/D ${dateFormat(row.tgl_selesai)}`
                    },
                    {
                        data: 'id',
                        render: (data, type, row) => {
                            const url = `{{ $controller . '/' . '${data}' }}`;
                            return `
                                <div class="d-flex justify-content-center gap-2">
                                    @include('components.printBtn', [
                                        'url' => '${url}',
                                        'className' => 'btn-icon',
                                    ])
                                </div>`;
                        },
                    }
                ]
            })
        })
    </script>
@endsection
