@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{$kategoriJudul}}</h5>
                        </div>
                        <!-- @if ($canCreate)
                            <div class="flex-shrink-0">
                                <a href="#" type="button" data-bs-toggle="modal" data-bs-target="#myModal"
                                    class="btn btn-success btn-label waves-effect waves-light"><i
                                        class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                    Tambah
                                </a>
                            </div>
                        @endif -->
                    </div>
                </div>
                <div class="card-body">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt"
                        style="width:100%">
                        <thead>
                            <tr>
                                <th>Nomor Permohonan @include('components.dtFilterInput',['index'=>0,'column' => 'Nama'])</th>
                                <th>Tanggal Permohonan @include('components.datepicker',['index'=>1,'name'=>'','className'=>'column-filter'])</th>
                                <th>Kategori @include('components.dtFilterInput',['index'=>2,'column'=>'kategori'])</th>
                                <th>Satker @include('components.dtFilterInput',['index' => 3,'column' => 'Satker'])</th>
                                <th>Aktifitas @include('components.dtFilterInput',['index' => 4,'column' => 'Aktifitas'])</th>
                                <th>File Persetujuan</th>
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
                columns: [
                    {
                        data: 'no_surat_permohonan'
                    },
                    {
                        data: 'tgl_surat_permohonan',
                        render :(data, type, row) =>{
                            return dateFormatIndo(row.tgl_surat_permohonan);
                        }
                    },
                    {
                        data: 'kategori'
                    },
                    {
                        data: 'inst_nama'
                    },
                    {
                        data: 'aktifitas'
                    },
                    {
                        data: 'id',
                        render: (data, type, row) => {
                            const url = `{{ '/bmn/penghapusan/penghapusansk/downloadZip/' . '${data}' }}`;
                            if(data){
                                return `<a href="${url}"><i class="ri-download-cloud-line"></i>Download</a>`;
                            }else{
                                return '';
                            }
                        }
                    },
                    {
                        data: 'id',
                        render: (data, type, row) => {
                            const url = `{{ $controller . '/' . '${data}' }}`;
                            return `
                                <div class="d-flex justify-content-center gap-2">
                                    @include('components.updateBtn', [
                                        'url' => '${url}',
                                    ])
                                </div>`;
                        },
                    }
                ]
            });
        })
    </script>
@endsection
