@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="col-lg-12">
        <div class="card">
            <div class="card-header">
                <div class="d-flex align-items-center">
                    <div class="flex-grow-1">
                        <h5 class="card-title mb-0">Satuan Kerja</h5>
                    </div>
                </div>
                <div class="row mt-3">
                    @include('components.dtSearchBox', [
                        'tableId' => $tableId,
                    ])
                </div>
            </div>
            <div class="card-body">
                <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt" style="width:100%">
                    <thead>
                        <tr>
                            <th>Nama</th>
                            <th>Aksi</th>
                        </tr>
                    </thead>
                </table>
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
                        data: 'satker'
                    },
                    {
                        data: 'inst_satkerkd',
                        render: (data, type, row) => {
                            const url = `{{ $controller . '/' . '${data}' }}`;
                            return `
                                <div class="d-flex justify-content-center gap-2">
                                    @include('components.showBtn', [
                                        'url' => '${url}',
                                    ])
                                </div>`;
                        }
                    },
                ]
            })
        })
    </script>
@endsection
