@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Data Pengadaan Langsung Kontrak</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive" style="width:100%">
                        <thead>
                            <tr>
                                <th>Th. Anggaran @include('components.dtFilterInput',['index' => 0,'column' => 'thn_ang'])</th>
                                <th>Nama Satker @include('components.dtFilterInput',['index' => 1,'column' => 'combo_satker_all'])</th>
                                <th>No Kontrak @include('components.dtFilterInput',['index' => 2,'column' => 'no_kontrak'])</th>
                                <th>Tgl. Kontrak @include('components.dtFilterInput',['index'=>3,'column'=> 'tgl_kontrak'])</th>
                                <th>Uraian @include('components.dtFilterInput',['index'=>4,'column'=> 'uraian_kontrak'])</th>
                                <th>Nilai Kontrak @include('components.dtFilterInput',['index'=>5,'column'=>'combo_nilai_kontrak'])</th>
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
                url: "{{ URL::to('/pengadaan/rencanapengadaanlangsung/gridData') }}",
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
                    data: 'no_kontrak',
                    width: '20%',
                    className:'dt-center'
                },
                {
                    data: 'tanggal_kontrak',
                    width: '12%',
                    className:'dt-center'
                },
                {
                    data: 'uraian_kontrak',
                },
                {
                    data: 'nilai_kontrak',
                    width: '12%'
                },
                {
                    data: 'id_kontrak',
                    classname: "text-center",
                    render: (data, type, row) => {
                        const url = `{{ url('/pengadaan/rencanapengadaanlangsung/${data}') }}`;
                        //const urlCetak = `{{ url('/asset/tik/cetakLabel/${data}') }}`;
                        return `
                            <div class="text-center btn-group" role="group">
                                @include('components.showBtn', [
                                    'url' => '${url}',
                                    'className'=>'btn-icon'
                                ])

                                {{--
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
                                --}}
                            </div>
                        `;
                    },searchable: false
                }
            ],
            columnDefs: [
            {
                targets: 5,
                className: 'dt-body-right',
                render: $.fn.dataTable.render.number('.', '.', 0, '')
            }
        ],
        });
    });
</script>
@endsection
