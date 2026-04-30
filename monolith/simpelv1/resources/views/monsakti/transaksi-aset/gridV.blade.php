<div class="row">
    <div class="col-lg-12">
        <div class="card">
            <div class="card-header">
                <div class="d-flex align-items-center">
                    <div class="flex-grow-1">
                        <h5 class="card-title mb-0">Monsakti Transaksi Aset</h5>
                    </div>
                </div>
            </div>
            <div class="card-body">
                <table id="tb-monsakti-traset" class="display table table-bordered dt-responsive my-dt" style="width:100%">
                    <thead>
                        <tr>
                            <!-- <th>Kode Register @include('components.dtFilterInput',['index' => '0', 'column' => 'kode_register'])</th> -->
                            <th width="15%">Keterangan </th>
                            <th>Periode Trx</th>
                            <th>Th. Anggaran </th>
                            <th>Nilai Aset</th>
                            <th>Nilai Aset Neraca</th>
                            <th>Nilai Perubahan</th>
                            <th>Nilai Perubahan Neraca</th>
                            <th>Sisa Masa Manfaat </th>
                            <th>Masa Manfaat </th>
                            <th>Tgl. Buku </th>
                            
                        </tr>
                    </thead>
                </table>
            </div>
        </div>
    </div>
    <!--end col-->
</div>
<style>
    #tb-monsakti-traset thead th {
        background-color: #405189;
        color: #ffffff;
        text-align: center;
        text-transform: uppercase;
    }
    
    .dataTables_length {
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


<script>
    const tableId = 'tb-monsakti-traset';
    $('#tb-monsakti-traset thead th').css({"background-color": "#405189", "color": "#ffffff","text-align":"center","text-transform":"uppercase"});
    $(function() {
        const dt = $('#tb-monsakti-traset').DataTable({
            serverSide: true,
            processing: true,
            deferRender: true,
            ordering: false,
            dom: dtLayout,
            language: {
                url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
            },
            ajax: {
                url: "{{ URL::to('/monsakti/transaksi-aset/gridData') }}?kode_barang={{ $model['kode_barang'] }}&nup={{ $model['nup'] }}&kdsatker={{ $model['id_satker_keu'] }}&kduakpb={{ $model['kode_satker'] }}",
                dataSrc: 'data',
            },
            //responsive :false,
            scrollX:true,
            columns: [
                // {
                //     data: 'kode_register',searchable: true
                // },
                {
                    data: 'ket',searchable: true
                },
                {
                    data: 'per',searchable: true
                },
                {
                    className:'dt-center',
                    data: 'thn_ang',
                    searchable: true
                },
                {
                    data: 'na',searchable: false,
                    className:'dt-right',
                    render: $.fn.dataTable.render.number('.', '.', 0, '')
                },
                {
                    data: 'nan',searchable: false,
                    className:'dt-right',
                    render: $.fn.dataTable.render.number('.', '.', 0, '')
                },
                {
                    data: 'np',searchable: false,
                    className:'dt-right',
                    render: $.fn.dataTable.render.number('.', '.', 0, '')
                },
                {
                    data: 'npn',searchable: false,
                    className:'dt-right',
                    render: $.fn.dataTable.render.number('.', '.', 0, '')
                },
                {
                    data: 'sm',searchable: true,
                    className:'dt-center',
                },
                {
                    data: 'mm',searchable: true,
                    className:'dt-center',
                },
                {
                    data: 'tgl_buku',searchable: false,
                    className:'dt-center',
                },
                
            ]

        })
    })
</script>

