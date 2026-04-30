@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Aset Tanah</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    @include('components.dtFilterBox', [
                        'columns' => $columns,
                        'selected'=> $defColumns
                    ])
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive" style="width:100%">
                        <thead>
                            <tr>
                                @foreach ($columns as $index => $column)
                                <th>
                                    {{ $column }}
                                    @include('components.dtFilterInput',[
                                        'index' => $index,
                                        'column' => $column,
                                        ]
                                    )
                                </th>
                                @endforeach
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
    
    .dataTables_length {
        width: auto;
        float: right;
    }

    .dataTables_info {
        width: 30%;
        float: left;
        padding-top:10px;
    }

    .dataTables_paginate {
        width: 70%;
        float: right;
        padding-top:10px;
    }
</style>
@endsection

@section('js')
<script>
    const tableId = `{{ $tableId }}`;
    const defColumn = @json($defColumns);
    $('#' + tableId+' thead th').css({"background-color": "#405189", "color": "#ffffff","text-align":"center","text-transform":"uppercase"});
    $(function() {
        const dt = $('#' + tableId).DataTable({
            processing: true,
            serverSide: true,
            ordering: false,
            "deferRender": true,
            dom: dtLayoutPrint,
            lengthMenu: [
                [10, 25, 50, 100, -1],
                [10, 25, 50, 100, 'Semua']
            ],
            buttons: buttonPrintDt('Daftar Aset Tanah',"{{ URL::to($controller) }}"),
            language: {
                url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
            },
            ajax: {
                url: "{{ URL::to($controller.'/gridData') }}",
                dataSrc: 'data',
                type:'POST'
            },
            responsive :false,
            scrollX:true,
            "initComplete": function(settings, json) {
                for(i=0;i<(dt.columns().header().length)-1;i++){
                    if($.inArray( i, defColumn )==-1){
                        let column = dt.column(i);
                        column.visible(false);
                    }
                }
            },
            columns: [{
                    data: 'id_satker_keu'
                },
                {
                    data: 'nama_satker'
                },
                {
                    data: 'kode_barang'
                },
                {
                    data: 'nama_barang'
                },
                {
                    data: 'nup'
                },
                {
                    data: 'kondisi'
                },
                {
                    data: 'jenis_dokumen'
                },
                {
                    data: 'kepemilikan'
                },
                {
                    data: 'jenis_sertifikat'
                },
                {
                    data: 'merk'
                },
                {
                    data: 'tgl_rekam_pertama',
                    render :(data, type, row) =>{
                        return dateFormatIndo(row.tgl_rekam_pertama);
                    }
                },
                {
                    data: 'tanggal_perolehan',
                    render :(data, type, row) =>{
                        return dateFormatIndo(row.tanggal_perolehan);
                    }
                },
                {
                    data: 'nilai_perolehan_pertama'
                },
                {
                    data: 'nilai_mutasi'
                },
                {
                    data: 'nilai_perolehan'
                },
                {
                    data: 'nilai_penyusutan'
                },
                {
                    data: 'nilai_buku'
                },
                {
                    data: 'kuantitas'
                },
                {
                    data: 'luas_tanah_seluruhnya'
                },
                {
                    data: 'luas_tanah_untuk_bangunan'
                },
                {
                    data: 'luas_tanah_sarana_lingkungan'
                },
                {
                    data: 'luas_tanah_kosong'
                },
                {
                    data: 'jumlah_foto'
                },
                {
                    data: 'status_penggunaan'
                },
                {
                    data: 'status_pengelolaan'
                },
                {
                    data: 'no_psp'
                },
                {
                    data: 'tgl_psp'
                },
                {
                    data: 'alamat'
                },
                {
                    data: 'rt_rw'
                },
                {
                    data: 'kelurahan_desa'
                },
                {
                    data: 'kecamatan'
                },
                {
                    data: 'uraian_kab_kota'
                },
                {
                    data: 'kode_kab_kota'
                },
                {
                    data: 'uraian_provinsi'
                },
                {
                    data: 'kode_provinsi'
                },
                {
                    data: 'kode_pos'
                },
                {
                    data: 'jumlah_kib'
                },
                {
                    data: 'sbsk'
                },
                {
                    data: 'optimalisasi'
                },
                {
                    data: 'status_sbsn'
                },
                {
                    data: 'id',
                    render: (data, type, row) => {
                        const url = `{{ url('/asset/tanah/${data}') }}`;
                        const urlCetak = `{{ url($controller.'/cetakLabel/${data}') }}`;
                        return `
                            <div class="text-center btn-group" role="group">
                                @include('components.showBtn', [
                                    'url' => '${url}',
                                    'className'=>'btn-icon'
                                ])
                            </div>
                        `;
                    },searchable: false
                }
            ]
        });
    })
</script>
@endsection
