@extends('layout.main')
@section('content')
<div class="row">
    <div class="col-12">
        <div class="page-title-box d-sm-flex align-items-center justify-content-between">
            <div class="page-title-left">
                <ol class="breadcrumb m-0">
                    <li class="breadcrumb-item"><a href="javascript: void(0);">Master</a></li>
                    <li class="breadcrumb-item active">Standar Spesifikasi dan Standar Jumlah (SSSJ) </li>
                </ol>
            </div>
        </div>
    </div>
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Master SSSJ</h5>
                        </div>
                        <div class="flex-shrink-0">
                            <a href="{{ URL::to('/master/sssj/create') }}" type="button" class="btn btn-success btn-label waves-effect waves-light"><i class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                Tambah
                            </a>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <table id="sssj-dt" class="display table table-bordered dt-responsive" style="width:100%">
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
        <!--end col-->
    </div>
</div>
@endsection

@section('js')
<script>
    $(function() {
        const tableId = '#sssj-dt';
        $(tableId).DataTable({
            processing: true,
            serverSide: true,
            ordering: false,
            "deferRender": true,
            ajax: {
                url: "{{ URL::to('/master/sssj/gridData') }}",
                dataSrc: 'data',
            },
            columns: [
                {
                    data: 'nama',searchable: true
                },
                {
                    data: 'id',
                    render: (data, type, row) => {
                        const url = `{{ url('/master/sssj/${data}') }}`;
                        return `
                            <div>
                                @include('components.updateBtn', [
                                    'url' => '${url}',
                                ])
                                @include('components.deleteBtn', [
                                    'url' => '${url}',
                                    'title' => '${row.nama}',
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
