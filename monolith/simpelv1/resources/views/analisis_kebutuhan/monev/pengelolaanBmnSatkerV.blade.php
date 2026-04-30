@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Pengelolaan BMN - {{$satker->inst_nama}}</h5>
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
                                <th>Aset</th>
                                <th>Nama</th>
                                <th>Jumlah</th>
                                <th>Satuan</th>
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
                    url: "{{ $controller . '/gridDataSatker?='.$satker->inst_satkerkd }}",
                    dataSrc: 'data',
                    data: (data) => {
                        const filterType = $('#dt-filter-by').data('filterby');
                        if (filterType?.trim() !== '') {
                            data.filterBy = filterType;
                        }
                    }
                },
                columns: [{
                        data: 'jenis_asset'
                    },
                    {
                        data: 'nama'
                    },
                    {
                        data: 'jumlah',
                        className: 'text-center',
                    },
                    {
                        data: 'satuan',
                        className: 'text-center',
                    },
                    {
                        data: 'id',
                        render: (data, type, row) => {
                            const url = `{{ '/analisis-kebutuhan/bmn/pengajuan/${data}' }}`;
                            return `
                                <div class="d-flex justify-content-center gap-2">
                                    @include('components.showBtn', [
                                        'url' => '${url}',
                                    ])
                                </div>`;
                        },
                    }
                ]
            })
        })
    </script>
@endsection
