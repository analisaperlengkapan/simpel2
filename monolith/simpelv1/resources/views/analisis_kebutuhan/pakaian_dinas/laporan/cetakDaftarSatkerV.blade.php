@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Cetak Daftar Pengajuan Pakaian Dinas</h5>
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
                                <th>Status</th>
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
                    url: `{{ $controller . '/gridDataSatker?pengajuanId='.$pengajuanId }}`,
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
                        data: 'status',
                        className:'text-center'
                    },
                    {
                        data: 'id',
                        render: (data, type, row) => {
                            const url = `{{ $controller . '/' . '${row.pengajuan_pakaian_dinas_id}/cetak?pengajuanSatkerId=${row.id}' }}`;
                            const disabled = row.ms_aktifitas_id || 'disabled'
                            return `
                                <div class="d-flex justify-content-center gap-2">
                                    @include('components.printBtn', [
                                        'url' => '${url}',
                                        'className' => 'btn-icon ${disabled}',
                                    ])
                                </div>`;
                        },
                    }
                ]
            })
        })
    </script>
@endsection
