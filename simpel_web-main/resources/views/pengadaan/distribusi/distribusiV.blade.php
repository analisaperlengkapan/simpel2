@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
@php
    $request = request();
    $jenis = $request->segment(3);
@endphp
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }}</h5>
                        </div>
                        <div class="flex-shrink-0">
                            @if ($jenis == 'pengisian')
                            <a href="{{ URL::to('/pengadaan/distribusi/pengisian/create') }}" type="button" class="btn btn-success btn-label waves-effect waves-light">
                                <i class="ri-add-line label-icon align-middle fs-16 me-2"></i> Tambah
                            </a>
                            @else
                            <button type="button" id="btn-add" class="btn btn-success btn-label waves-effect waves-light">
                                <i class="ri-add-line label-icon align-middle fs-16 me-2"></i> Tambah
                            </button>
                            @endif
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive" style="width:100%">
                        <thead>
                            <tr>
                                <th>Nomor Kontrak @include('components.dtFilterInput',['index' => 0,'column' => 'Satker'])</th>
                                <th>Tanggal Kontrak @include('components.dtFilterInput',['index'=>1,'column'=>''])</th>
                                <th>Nilai Kontrak @include('components.dtFilterInput',['index'=>2,'column'=>''])</th>
                                <th>Nama Barang @include('components.dtFilterInput',['index' => 3,'column' => 'Satker'])</th>
                                <th>Jumlah Barang @include('components.dtFilterInput',['index' => 4,'column' => 'Satker'])</th>
                                <th>Nilai Barang @include('components.dtFilterInput',['index' => 5,'column' => 'Satker'])</th>
                                <th>Satker Tujuan @include('components.dtFilterInput',['index' => 6,'column' => 'Satker'])</th>
                                <th>File</th>
                                <th>Status @include('components.dtFilterInput',['index' => 8,'column' => 'Satker'])</th>
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

<div class="modal fade" id="modal-hist" data-bs-backdrop="static" data-bs-keyboard="false" tabindex="-1" role="dialog" aria-labelledby="staticBackdropLabel" aria-hidden="true">
    <div class="modal-dialog modal-lg" role="document">
        <div class="modal-content">
            <div class="modal-header">
                <h5 class="modal-title">History Penyimpanan dan Distribusi</h5>
                <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Tutup"></button>
            </div>
            <div class="modal-body p-5">
                <table id="tb-hist" class="display table table-bordered dt-responsive" style="width:100%">
                    <thead>
                        <tr class="text-center">
                            <th >No</th>
                            <th>Status</th>
                            <th>Tanggal</th>
                            <th>Keterangan</th>
                        </tr>
                    </thead>
                </table>
                <div class="mt-4">
                    <div class="hstack gap-2">
                        <a href="javascript:void(0);" class="btn btn-link link-success fw-medium" data-bs-dismiss="modal"><i class="ri-close-line me-1 align-middle"></i> Tutup</a>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>

<div class="modal fade" id="modal-add" data-bs-backdrop="static" data-bs-keyboard="false" tabindex="-1" role="dialog" aria-labelledby="staticBackdropLabel" aria-hidden="true">
    <div class="modal-dialog modal-xl" role="document">
        <div class="modal-content">
            <div class="modal-header">
                <h5 class="modal-title">Daftar Barang</h5>
                <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Tutup"></button>
            </div>
            <div class="modal-body p-5">
                <table id="tb-add" class="display table table-bordered dt-responsive" style="width:100%">
                    <thead>
                            <tr class="text-center">
                                <th>Nomor Kontrak</th>
                                <th>Tanggal Kontrak</th>
                                <th>Nama Barang</th>
                                <th>Jumlah Barang</th>
                                <th>Nilai Barang</th>
                                <th>Satker Tujuan</th>
                                <th>File</th>
                                <th>Status</th>
                                <th>Pilih</th>
                            </tr>
                        </thead>
                    </table>
                <div class="mt-4">
                    <div class="hstack gap-2">
                        <a href="javascript:void(0);" class="btn btn-link link-success fw-medium" data-bs-dismiss="modal"><i class="ri-close-line me-1 align-middle"></i> Tutup</a>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>

<div class="modal fade" id="modal-status" tabindex="-1" aria-labelledby="exampleModalgridLabel" aria-modal="true">
    <div class="modal-dialog">
        <div class="modal-content">
            <div class="modal-header">
                <h5 class="modal-title" id="exampleModalgridLabel">Verifikasi</h5>
                <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
            </div>
            <div class="modal-body">
                <form action="javascript:void(0);">
                    <div class="row g-3">
                        <div class="col-xxl-12">
                            <div>
                                <select name="modal-id_status" id="modal-id_status" class="form-control"></select>
                                <input type="hidden" id="modal-id">
                            </div>
                        </div><!--end col-->
                        <div class="col-xxl-12">
                            <label for="" class="form-label">Keterangan</label>
                            <input type="text" id="modal-ket" class="form-control">
                        </div>
                        <div class="col-lg-12">
                            <div class="hstack gap-2 justify-content-end">
                                <button type="button" class="btn btn-light" data-bs-dismiss="modal">Batal</button>
                                <button type="button" id="simpan-status" class="btn btn-primary">Simpan</button>
                            </div>
                        </div><!--end col-->
                    </div><!--end row-->
                </form>
            </div>
        </div>
    </div>
</div>
@endsection

@section('js')
<script>
    const tableId = `{{ $tableId }}`;
    $(function() {
        var currentPath = window.location.pathname;
        var pathSegments = currentPath.split('/');
        var lastSegment = pathSegments[pathSegments.length - 1];
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
                url: "{{ URL::to('/pengadaan/distribusi/gridData') }}/"+lastSegment,
                dataSrc: 'data',
            },
            columns: [{
                    data: 'no_kontrak'
                },
                {
                    data: 'tgl_kontrak'
                },
                {
                    data: 'nilai_kontrak'
                },
                // {
                //     data: 'tgl_kontrak',
                //     render :(data, type, row) =>{
                //         return dateFormatIndo(row.tgl_kontrak);
                //     }
                // },
                {
                    data: 'nm_barang'
                },
                {
                    data: 'jml_barang'
                },
                {
                    data: 'nilai_barang'
                },
                {
                    data: 'inst_nama'
                },
                {"data": function (row, data, index, display) {
                    const fileSpk = row.file_spk?`<a href="{{ url('${row.file_spk}') }}" class="text-nowrap" download terget="_blank"> <i class="ri-download-cloud-line"></i> File SPK</a><br/>`:'';
                    const fileBast = row.file_bast?`<a href="{{ url('${row.file_bast}') }}" class="text-nowrap" download terget="_blank"> <i class="ri-download-cloud-line"></i> File BAST</a><br/>`:'';
                    const fileFoto = row.file_foto?`<a href="{{ url('${row.file_foto}') }}" class="text-nowrap" download terget="_blank"> <i class="ri-download-cloud-line"></i> File Foto</a>`:'';
                    let file = fileSpk+fileBast+fileFoto;
                    return file;
                }},
                {
                    data: 'status',
                    render :(data, type, row) =>{
                        return '<a href="#" class="history" data-id="'+row.id+'">'+row.status+'</a>';
                    }
                },
                {
                    data: 'id',
                    render: (data, type, row) => {
                        const url = `{{ url('/pengadaan/distribusi/pengisian/${data}') }}`;
                        const urlKonfirm = `{{ url('/pengadaan/distribusi/konfirmasi-penerimaan/${data}') }}`;
                        const urlCetak = `{{ url('/pengadaan/distribusi/cetakLabel/${data}') }}`;
                        const btnCetak = `@include('components.printBtn', [
                                            'url' => '${urlCetak}',
                                            'className'=>'btn-icon'
                                        ]) `;
                        const btnEdit = `@include('components.updateBtn', [
                                            'url' => '${url}',
                                            'className'=>'btn-icon'
                                        ]) `;
                        const btnEditConfirm = `@include('components.updateBtn', [
                            'url' => '${urlKonfirm}',
                            'className'=>'btn-icon'
                        ]) `;
                        const btnDel = `@include('components.deleteBtn', [
                                        'url' => '${url}',
                                        'title' => '${row.nm_barang}',
                                        'tableId'=>'${tableId}',
                                        'className'=>'btn-icon'
                                    ]) `;
                        const btnVerif = '<button data-id="'+data+'" data-file_bast="'+row.file_bast+'" data-file_foto="'+row.file_foto+'" data-id_status="'+row.id_status+'" type="button" class="btn btn-success waves-effect waves-light ubah-status"><i class="ri-checkbox-line label-icon align-middle fs-16 me-2"></i>Verifikasi</button>';
                        let btn = btnEdit+btnDel;
                        if(row.id_status==1 && lastSegment=='pengisian'){
                            btn = btnEdit+btnDel;
                        }else if(
                            (lastSegment=='masuk-gudang' && (row.id_status==2 || row.id_status==3 || row.id_status==4)) ||
                            (lastSegment=='keluar-gudang' && (row.id_status==5 || row.id_status==6 || row.id_status==7))
                        ){
                            btn = btnVerif+(lastSegment=='masuk-gudang'?btnCetak:'');
                        }else if(lastSegment=='konfirmasi-penerimaan' && row.id_status==8){
                            btn = btnVerif+btnEditConfirm;
                        }else{
                            btn = '';
                        }
                        return `<div class="text-center" role="group">${btn}</div>`;
                    },
                }
            ],
            columnDefs:
            [
                {
                    targets: 2,
                    render: $.fn.dataTable.render.number('.', '', 0, '')
                },
                {
                    targets: 5,
                    render: $.fn.dataTable.render.number('.', '', 0, '')
                },
            ],

        });

        const dtHist = $('#tb-hist').DataTable({
            info: false,
            ordering: false,
            paging: false,
            language: {
                url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
            },
            ajax: {
                url: "{{ URL::to('/pengadaan/distribusi/histData/0') }}",
                dataSrc: 'data',
            },
            columns: [
                {"data": function (row, data, index, display) {
                    return (display.row+1);
                }},
                {data: 'status'},
                {data: 'created_at'},
                {data: 'ket'},
            ]

        });

        $('#' + tableId).on('click', '.history', function(e){
            e.preventDefault();
            const id = $(this).data('id');
            const url = `{{ url('/pengadaan/distribusi/histData/${id}') }}`;
            dtHist.ajax.url(url).load();
            $('#modal-hist').modal('show');
        }).on('click', '.ubah-status', function(){
            const id = $(this).data('id');
            const id_status = $(this).data('id_status');
            const url = `{{ url('/pengadaan/distribusi/getOptionStatus') }}`;
            const file_bast = $(this).data('file_bast');
            const file_foto = $(this).data('file_foto');
            if(id_status!= 8 || (id_status== 8 && file_bast && file_foto)){
                $('#modal-id').val(id);
                $.ajax({
                    url: url,
                    method: "POST",
                    data:JSON.stringify({'jenis':lastSegment}),
                    contentType: "application/json",
                    success: (res) => {
                        var selectElement = $('#modal-id_status');
                        selectElement.empty();
                        $.each(res, function (index, option) {
                            selectElement.append($('<option>', {
                                value: option.id,
                                text: option.status
                            }));
                        });
                        $('#modal-status').modal('show');
                    },
                    error: (xhr, status, error) => {
                        console.log(xhr);
                        console.log(status);
                        console.log(error);
                    },
                });
            }else{
                swal('Info','Upload File BAST dan Foto terlebih dahulu','warning');
            }
        });

        const tb_add = $('#tb-add').DataTable({
            processing: true,
            serverSide: true,
            ordering: false,
            "deferRender": true,
            language: {
                url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
            },
            ajax: {
                url: "{{ URL::to('/pengadaan/distribusi/gridDataAdd') }}/"+lastSegment,
                dataSrc: 'data',
            },
            columns: [{
                    data: 'no_kontrak'
                },
                {
                    data: 'tgl_kontrak',
                    render :(data, type, row) =>{
                        return dateFormatIndo(row.tgl_kontrak);
                    }
                },
                {
                    data: 'nm_barang'
                },
                {
                    data: 'jml_barang'
                },
                {
                    data: 'nilai_barang'
                },
                {
                    data: 'inst_nama'
                },
                {"data": function (row, data, index, display) {
                    const fileSpk = row.file_spk?`<a href="{{ url('${row.file_spk}') }}" class="text-nowrap" download terget="_blank"> <i class="ri-download-cloud-line"></i> File SPK</a><br/>`:'';
                    const fileBast = row.file_bast?`<a href="{{ url('${row.file_bast}') }}" class="text-nowrap" download terget="_blank"> <i class="ri-download-cloud-line"></i> File BAST</a><br/>`:'';
                    const fileFoto = row.file_foto?`<a href="{{ url('${row.file_foto}') }}" class="text-nowrap" download terget="_blank"> <i class="ri-download-cloud-line"></i> File Foto</a>`:'';
                    let file = fileSpk+fileBast+fileFoto;
                    return file;
                }},
                {"data": function (row, data, index, display) {
                    return '<a href="#" class="history" data-id="'+row.id+'">'+row.status+'</a>';
                }},
                {data: 'id',
                    render: (data, type, row) => {
                        return '<button data-id="'+data+'" type="button" class="btn btn-success waves-effect waves-light btn-icon pilih"><i class="ri-add-circle-line"></i></button>';
                    },
                }
            ]

        });

        $('#btn-add').on('click', function(){
            const url = `{{ url('/pengadaan/distribusi/gridDataAdd/${lastSegment}') }}`;
            tb_add.ajax.url(url).load();
            $('#modal-add').modal('show');
        });

        $('#tb-add').on('click', '.pilih', function(){
            const id = $(this).data('id');
            const url = `{{ url('/pengadaan/distribusi/pilihData') }}`;
            swal(`Yakin akan memilih data ini ?`, {
                icon: "info",
                dangerMode: true,
                buttons: true,
            }).then((res) => {
                if (res) {
                    $.ajax({
                        url: url,
                        method: "POST",
                        data:JSON.stringify({'jenis':lastSegment,'id':id}),
                        contentType: "application/json",
                        success: (res) => {
                            notify({
                                type: res.status || "success",
                                message: res.msg || "Berhasil",
                            });
                            const table = $("#" + tableId).DataTable();
                            table.ajax.reload();
                            $('#modal-add').modal('hide');
                        },
                        error: (xhr, status, error) => {
                            console.log(xhr);
                            console.log(status);
                            console.log(error);
                        },
                    });
                }
            });
        });

        $('#simpan-status').on('click', function(){
            const id = $('#modal-id').val();
            const id_status = $('#modal-id_status').val();
            const ket = $('#modal-ket').val();
            const url = `{{ url('/pengadaan/distribusi/simpanUbahStatus') }}`;
            $.ajax({
                url: url,
                method: "POST",
                data:JSON.stringify({'id_status':id_status,'id':id,'ket':ket}),
                contentType: "application/json",
                success: (res) => {
                    notify({
                        type: res.status || "success",
                        message: res.msg || "Berhasil",
                    });
                    const table = $("#" + tableId).DataTable();
                    table.ajax.reload();
                    $('#modal-status').modal('hide');
                },
                error: (xhr, status, error) => {
                    console.log(xhr);
                    console.log(status);
                    console.log(error);
                },
            });
        });

        $('#modal-status').on('hide.bs.modal', function(){
            $('#modal-id').val('');
            $('#modal-id_status').val('');
            $('#modal-ket').val('');
        });
    })
</script>
@endsection
