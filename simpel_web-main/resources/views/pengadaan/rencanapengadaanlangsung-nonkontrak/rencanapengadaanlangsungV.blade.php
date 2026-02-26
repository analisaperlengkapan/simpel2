@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Data Pengadaan Langsung Non Kontrak</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive" style="width:100%">
                        <thead>
                            <tr>
                                <th>Th. Anggaran @include('components.dtFilterInput',['index' => 0,'column' => 'thn_ang'])</th>
                                <th>Nama Satker @include('components.dtFilterInput',['index' => 1,'column' => 'combo_satker_all'])</th>
                                <th>No Dokumen @include('components.dtFilterInput',['index' => 2,'column' => 'no_dokumen'])</th>
                                <th>Tgl. BAST @include('components.dtFilterInput',['index'=>3,'column'=>'tanggal_bast'])</th>
                                <th>Kategori BAST @include('components.dtFilterInput',['index'=>4,'column'=>'kategori_bast'])</th>
                                <th>Uraian BAST @include('components.dtFilterInput',['index'=>5,'column'=>'uraian_bast'])</th>
                                <th>Nilai BAST @include('components.dtFilterInput',['index'=>6,'column'=>'nilai_bast'])</th>
                                <th>Nama Suplier @include('components.dtFilterInput',['index'=>7,'column'=>'supplier_wp_wb'])</th>
                                <th width="5%">Aksi</th>
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
    const tableId = `{{ $tableId }}`;
    $(function() {
        $('#combo_satker_all').select2();

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
                url: "{{ URL::to('/pengadaan/pengadaanlangsung-nonkontrak/gridData') }}",
                dataSrc: 'data',
            },
            columns: [
                {
                    data: 'thn_ang',
                    width: '10%',
                    className:'dt-center'
                },
                {
                    data: 'inst_nama',
                    width: '15%'
                },
                {
                    data: 'no_dokumen',searchable: true
                },
                {
                    data: 'tgl_bast',searchable: true
                },
                {
                    data: 'kategori_bast',searchable: true
                },
                {
                    data: 'uraian_bast',searchable: true
                },
                {
                    data: 'nilai_bast',searchable: true
                },
                {
                    data: 'nama_suplier',searchable: true
                },
                {
                    data: 'id_bast',
                    classname: "text-center",
                    render: (data, type, row) => {
                        const url = `{{ url('/pengadaan/pengadaanlangsung-nonkontrak/${data}') }}`;
                        //const urlCetak = `{{ url('/asset/tik/cetakLabel/${data}') }}`;
                        return `
                            <div class="text-center btn-group" role="group">
                                @include('components.showBtn', [
                                    'url' => '${url}',
                                    'className'=>'btn-icon'
                                ])
                                @if ($canCreate)
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
                                @endif
                            </div>
                        `;
                    },searchable: false
                }
            ],
            columnDefs: [
            {
                targets: 6,
                className: 'dt-body-right',
                render: $.fn.dataTable.render.number('.', '.', 0, '')
            }
        ],
        });
    });
</script>
@endsection
