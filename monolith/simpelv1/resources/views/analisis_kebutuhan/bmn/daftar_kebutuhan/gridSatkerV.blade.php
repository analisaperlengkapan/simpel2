@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
<div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Daftar Kebutuhan BMN Satker</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                @if(request()->segment(3) == 'penyusunan-prioritas')
                <form action="{{ $controller }}" method="POST" class="ajaxForm">
                    @csrf
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <input class="form-control" type="file" name="excel_file" accept=".xlsx, .xls">
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                            <button class="btn btn-primary" type="submit">Import</button>
                            </div>
                        </div>
                    </div>
                </form>
                <hr/>
                @endif
                <table id="{{ $tableId }}" class="display table table-bordered dt-responsive" style="width:100%">
                    <thead>
                        <tr>
                            <th>Id Sistem</th>
                            <th>Tahun Anggaran</th>
                            <th>Nama Pengadaan</th>
                            <th>Satker</th>
                            <th>Kode Barang</th>
                            <th>Nama Barang</th>
                            <th>Jumlah Barang</th>
                            <th>Jumlah Barang Disetujui</th>
                            <th>Jumlah Barang Ditolak</th>
                            <th>Keterangan Barang</th>
                            <th>Alasan Pengadaan</th>
                            <th>Prioritas</th>
                        </tr>
                    </thead>
                </table>
            </div>
            </div>
        </div>
    </div>
    <style>
    .dataTables_length {
        width: auto;
        float: right;
    }
</style>
@endsection

@section('js')
    <script>
        const tableId = `{{ $tableId }}`;
        $(function() {
            $('#' + tableId+' thead th').css({"background-color": "#405189", "color": "#ffffff","text-align":"center","text-transform":"uppercase"});
            const dt = $('#' + tableId).DataTable({
                serverSide: true,
                processing: true,
                deferRender: true,
                ordering: false,
                dom: dtLayoutPrint,
                buttons: [{
                    extend: "excel",
                    action: function (e, dt, node, config) {
                        let param = dt.ajax.params();
                        $.ajax({
                            url: "{{ '/analisis-kebutuhan/bmn/penyusunan-prioritas/cetakExcel?id='.$id  }}",
                            type: "POST",
                            data: param,
                            xhrFields: {
                                responseType: "blob",
                            },
                            success: function (result, status, xhr) {
                                var disposition = xhr.getResponseHeader(
                                    "content-disposition"
                                );
                                var matches = /"([^"]*)"/.exec(disposition);
                                var filename =
                                    matches != null && matches[1]
                                        ? matches[1]
                                        : "template.xlsx";
                                var blob = new Blob([result], {
                                    type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                                });
                                var link = document.createElement("a");
                                link.href = window.URL.createObjectURL(blob);
                                link.download = filename;
                                document.body.appendChild(link);
                                link.click();
                                document.body.removeChild(link);
                            },
                            error: function (error) {
                                console.log(error);
                            },
                        });
                    },
                }],
                responsive :false,
                scrollX:true,
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ '/analisis-kebutuhan/bmn/penyusunan-prioritas/gridDataSatker?id='.$id  }}",
                    dataSrc: 'data',
                },
                columns: [{
                        data: 'id'
                    },{
                        data: 'tahun'
                    },
                    {
                        data: 'nama'
                    },
                    {
                        data: 'satker'
                    },
                    {
                        data: 'kode_barang'
                    },
                    {
                        data: 'nm_barang'
                    },
                    {
                        data: 'jumlah'
                    },
                    {
                        data: 'jml_setuju'
                    },
                    {
                        data: 'jml_tolak'
                    },
                    {
                        data: 'keterangan'
                    },
                    {
                        data: 'alasan'
                    },
                    {
                        data: 'prioritas'
                    },
                ]
            });

            $('#' + tableId).on('focusout','.prioritas',function(){
                let id = $(this).data('id');
                let prioritas = $(this).val();
                if(prioritas){
                    var data = new FormData();
                    data.append('id', id);
                    data.append('prioritas', prioritas);
                    $.ajax({
                        method: "POST",
                        url: `{{ $controller. '/savePrioritas' }}`,
                        data: data,
                        processData: false,
                        contentType: false,
                        success: function(){
                        },
                        error: showError,
                    }).done(function( msg ) {
                        dt.ajax.reload();
                    });
                }

            });
        })
    </script>
@endsection
