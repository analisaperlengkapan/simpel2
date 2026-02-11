@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Sub Spesifikasi Pakaian Dinas</h5>
                        </div>
                        <div class="flex-shrink-0">
                            <a href="{{ URL::to('/master/pakaian-dinas/subspesifikasi-pakaian-dinas/create') }}" type="button" class="btn btn-success btn-label waves-effect waves-light"><i class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                Tambah
                            </a>
                        </div>
                    </div>
                    <div class="row mt-3">
                        @include('components.dtSearchBox', [
                            'tableId' => $tableId,
                            'filterBy' => [
                                ['text' => 'Jenis Pakaian', 'value' => 'c.nama'],
                                ['text' => 'Spesifikasi', 'value' => 'b.nama'],
                                ['text' => 'Sub Spesifikasi', 'value' => 'a.nama'],
                            ],
                        ])
                    </div>
                </div>
                <div class="card-body">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive" style="width:100%">
                        <thead>
                            <tr>
                                <th>No</th>
                                <th>Jenis Pakaian</th>
                                <th>Spesifikasi</th>
                                <th>Sub Spesifikasi</th>
                                <th>Aksi</th>
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
</style>
@endsection

@section('js')
<script>
    $(function() {
        const tableId = `{{ $tableId }}`;
        const dt = $('#' + tableId).DataTable({
            processing: true,
            serverSide: true,
            ordering: false,
            "deferRender": true,
            dom: dtLayout,
            language: {
                url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
            },
            ajax: {
                url: "{{ URL::to('/master/pakaian-dinas/subspesifikasi-pakaian-dinas/gridData') }}",
                dataSrc: 'data',
                data: (data) => {
                        const filterType = $('#dt-filter-by').data('filterby');
                        if (filterType.trim() !== '') {
                            data.filterBy = filterType;
                        }
                    }
            },
            columns: [
                {"data": function (row, data, index, display) {
                        return (display.row+1);
                }},
                {
                    data: 'jenis_pakaian'
                },
                {
                    data: 'nm_spesifikasi'
                },
                {
                    data: 'nama'
                },
                {
                    data: 'id',
                    render: (data, type, row) => {
                        const url = `{{ url('/master/pakaian-dinas/subspesifikasi-pakaian-dinas/${data}') }}`;
                        return `
                            <div class="text-center" role="group">
                                @include('components.updateBtn', [
                                    'url' => '${url}',
                                    'className'=>'btn-icon'
                                ])
                                @include('components.deleteBtn', [
                                    'url' => '${url}',
                                    'title' => '${row.nama}',
                                    'tableId'=>'${tableId}',
                                    'className'=>'btn-icon'
                                ])
                            </div>
                        `;
                    },searchable: false
                }
            ]

        })
    })
</script>
@endsection
