@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Pencadangan</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/pengaturan/pencadangan" method="POST" class="ajaxForm">
                        @csrf
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2">
                                    <button type="submit" class="btn btn-primary">Buat Pencadangan</button>
                                </div>
                            </div>
                        </div>
                    </form>
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
                responsive: false,
                dom: dtLayoutPrint,
                lengthMenu: [
                    [10, 25, 50, 100, -1],
                    [10, 25, 50, 100, 'Semua']
                ],
                buttons: buttonPrintDt('Daftar Pencadangan',"{{ URL::to($controller) }}"),
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ URL::to($controller . '/gridData') }}",
                    dataSrc: 'data',
                },
                scrollX: true,
                "initComplete": function(settings, json) {
                    for (i = 0; i < (dt.columns().header().length) - 1; i++) {
                        if ($.inArray(i, defColumn) == -1) {
                            let column = dt.column(i);
                            column.visible(false);
                        }
                    }
                },
                columns: [{
                        data: 'created_at',
                        className: 'text-center',
                        render: (data, type, row) => {
                            return dateFormatIndo(data);
                        }
                    },
                    {
                        data: 'filename',
                        className: 'text-center',
                        searchable: false,
                    },
                    {
                        data: 'is_complete',
                        className: 'text-center',
                        searchable: false,
                        render: (data, type, row) => {
                            return data == 0 ? 'Proses Backup' : 'Selesai';
                        }
                    },
                    {
                        data: 'id',
                        className: "text-center",
                        render: (data, type, row) => {
                            const url = `{{ url($controller . '/${data}') }}`;
                            const disabled = row.is_complete == 1 ? '' : 'disabled';
                            return `
                                <div class="text-center btn-group" role="group">
                                    <a href="#" data-id="${data}" data-tanggal="${dateFormatIndo(row.created_at)}" type="button" class="btn btn-warning waves-effect waves-light restore-btn ${disabled} btn-icon"><i class="ri-time-line"></i></a>
                                    <a href="${url}" target="_blank" type="button" class="btn btn-primary waves-effect waves-light ${disabled} btn-icon"><i class="ri-download-fill"></i></a>
                                </div>
                            `;
                        },
                        searchable: false
                    }
                ]

            });

            $(document).on('click', '.restore-btn', function() {
                const id = $(this).data('id');
                const title = $(this).data('tanggal');
                swal(`Yakin akan Merestore Database ke tanggal ${title}?`, {
                    icon: "info",
                    dangerMode: true,
                    buttons: true,
                }).then((res) => {
                    if (!res) return;
                    $.post(`{{ $controller }}/restore`, {
                            id
                        }).done((res) => location.reload())
                        .fail(err => notify({
                            ...err.responseJSON,
                            type: 'danger'
                        }));
                });
            })
        });
    </script>
@endsection
