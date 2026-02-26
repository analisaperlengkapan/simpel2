@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Kritik dan Saran</h5>
                        </div>
                        <div class="flex-shrink-0">
                            <a href="{{ URL::to('/suport/kritik/create') }}" type="button" class="btn btn-success btn-label waves-effect waves-light"><i class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                Tambah
                            </a>
                        </div>
                    </div>
                    <div class="row mt-3">
                        @include('components.dtSearchBox', [
                            'tableId' => $tableId,
                            'filterBy' => [
                                ['text' => 'User', 'value' => 'created_by'],
                                ['text' => 'kritik', 'value' => 'kritik'],
                                ['text' => 'saran', 'value' => 'saran'],
                            ],
                        ])
                    </div>
                </div>
                <div class="card-body">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive" style="width:100%">
                        <thead>
                            <tr>
                                <th>User</th>
                                <th>kritik</th>
                                <th>saran</th>                            
                                <th width="15%">Aksi</th>
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
                url: "{{ URL::to('/suport/kritik/gridData') }}",
                dataSrc: 'data',
                data: (data) => {
                        const filterType = $('#dt-filter-by').data('filterby');
                        if (filterType.trim() !== '') {
                            data.filterBy = filterType;
                        }
                    }
            },
            columns: [{
                    data: 'created_by',searchable: true
                },
                {
                    data: 'kritik',searchable: true
                },
                {
                    data: 'saran',searchable: true
                }, 
                {
                    data: 'id',
                    render: (data, type, row) => {
                        const url = `{{ url('/suport/kritik/${data}') }}`;
                        return `
                        <div class="text-center btn-group" role="group">
                                @include('components.showBtn', [
                                    'url' => '${url}',
                                ])
                                @include('components.updateBtn', [
                                    'url' => '${url}/edit',
                                ])
                                @include('components.deleteBtn', [
                                    'url' => '${url}',
                                    'title' => '${row.judul}',
                                    'tableId'=>'${tableId}'
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
