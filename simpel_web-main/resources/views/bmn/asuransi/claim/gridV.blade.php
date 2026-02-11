@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Claim Asuransi</h5>
                        </div>
                        <div class="flex-shrink-0">
                            <a href={{ url($controller . '/create') }} type="button"
                                class="btn btn-success btn-label waves-effect waves-light"><i
                                    class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                Tambah
                            </a>

                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <div class="card-body">
                        @include('components.dtFilterBox', [
                            'columns' => $columns,
                            'selected' => $defColumns,
                        ])
                        <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt"
                            style="width:100%">
                            <thead>
                                <tr>
                                    @foreach ($columns as $index => $column)
                                        <th>
                                            {{ $column }}
                                            @include('components.dtFilterInput', [
                                                'index' => $index,
                                                'column' => $column,
                                            ])
                                        </th>
                                    @endforeach
                                    <th>Aksi</th>
                                </tr>
                            </thead>
                        </table>
                    </div>
                </div>
            </div>
        </div>
        <!--end col-->
    </div>
@endsection

@section('js')
    <script>
        const tableId = `{{ $tableId }}`;
        const defColumn = @json($defColumns);
        $(function() {
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
                    url: "{{ url($controller . '/gridData') }}",
                    dataSrc: 'data',
                },
                columns: [{
                        data: 'inst_nama',
                        searchable: true,
                    },
                    {
                        data: 'kode_barang',
                        searchable: true,
                    },
                    {
                        data: 'nup',
                        searchable: false,
                    },
                    {
                        data: 'nm_barang',
                        searchable: true,
                    },
                    {
                        data: 'polis_no',
                        searchable: true,
                    },
                    {
                        data: 'polis_tgl',
                        searchable: true,
                    },
                    {
                        data: 'polis_premi',
                        searchable: false,
                    },
                    {
                        data: 'id',
                        className: "dt-center",
                        width: '15%',
                        render: (data, type, row) => {
                            const url =
                                `{{ url($controller . '/${data}') }}`;
                            return `
                            <div class="text-center btn-group" role="group">

                                @include('components.updateBtn', [
                                    'url' => '${url}/edit',
                                    'className' => 'btn-icon',
                                ])
                            </div>
                        `;
                        },
                        searchable: false
                    }
                ],

            });

        });
    </script>
@endsection
