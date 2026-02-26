@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Panduan</h5>
                        </div>
                        <div class="flex-shrink-0">
                            <a href="{{ URL::to('/suport/bantuan/create') }}" type="button" class="btn btn-success btn-label waves-effect waves-light"><i class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                Tambah
                            </a>
                        </div>
                    </div>
                    <div class="row mt-3">
                        @include('components.dtSearchBox', [
                            'tableId' => $tableId,
                            'filterBy' => [
                                ['text' => 'Judul/Tentang', 'value' => 'judul']
                            ],
                        ])
                    </div>
                </div>
                <div class="card-body">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive" style="width:100%">
                        <thead>
                            <tr>
                                <th>Judul/Tentang</th>
                                <th>File Panduan</th>
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
                url: "{{ URL::to('/suport/bantuan/gridData') }}",
                dataSrc: 'data',
                data: (data) => {
                        const filterType = $('#dt-filter-by').data('filterby');
                        if (filterType.trim() !== '') {
                            data.filterBy = filterType;
                        }
                    }
            },
            columns: [
                {
                    data: 'judul',searchable: true
                },
                {
                    "data": function (row, data, index, display) {
                        const fileSK = row.file_panduan ? `<a href="{{ url('${row.file_panduan}') }}" class="text-nowrap" download terget="_blank"> <i class="ri-download-cloud-line"></i> File Panduan</a><br/>`:'-';
                        let file = fileSK;
                        return file;
                    }
                },
                {
                    data: 'id',
                    classname: "text-center",
                    render: (data, type, row) => {
                        const url = `{{ url('/suport/bantuan/${data}') }}`;
                        return `
                            <div class="text-center btn-group" role="group">
                                @include('components.showBtn', [
                                    'url' => '${url}',
                                    'className'=>'btn-icon'
                                ])
                                @include('components.updateBtn', [
                                    'url' => '${url}/edit',
                                    'className'=>'btn-icon'
                                ])
                                @include('components.deleteBtn', [
                                    'url' => '${url}',
                                    'title' => '${row.id}',
                                    'tableId'=>'${tableId}',
                                    'className'=>'btn-icon'
                                ])
                            </div>
                        `;
                    },searchable: false
                }
            ]

        });
    });
</script>
@endsection
