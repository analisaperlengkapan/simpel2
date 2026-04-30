@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Pengajuan</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body p-4">
                    <div class="row">
                        <div class="table-responsive">
                            <table class="table table-borderless mb-0">
                                <tbody>
                                    <tr>
                                        <th class="ps-0" width="10%" scope="row">Judul</th>
                                        <td width="5%">:</td>
                                        <td class="text-muted">
                                            {{ $pengajuan['nama'] ?? '-' }}
                                        </td>
                                    </tr>
                                    <tr>
                                        <th class="ps-0" width="10%" scope="row">Permintaan</th>
                                        <td width="5%">:</td>
                                        <td class="text-muted">
                                            {{ $pengajuan['is_reguler'] == 1 ? 'Reguler' : 'Cepat' }}
                                        </td>
                                    </tr>
                                    @if ($pengajuan['is_reguler'] == 1)
                                        <tr>
                                            <th class="ps-0" width="10%" scope="row">Periode</th>
                                            <td width="5%">:</td>
                                            <td class="text-muted">
                                                {{ $pengajuan['tgl_mulai'] }} S/D {{ $pengajuan['tgl_selesai'] }}
                                            </td>
                                        </tr>
                                    @endif
                                    <tr>
                                        <th class="ps-0" width="10%" scope="row">Deskripsi</th>
                                        <td width="5%">:</td>
                                        <td class="text-muted">
                                            {{ $pengajuan['deskripsi'] ?? '-' }}
                                        </td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </div>
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Satuan Kerja</h5>
                        </div>
                    </div>
                    <div class="row mt-3">
                        @include('components.dtSearchBox', [
                            'tableId' => $tableId,
                        ])
                    </div>

                </div>
                <div class="card-body">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt"
                        style="width:100%">
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
    </div>
@endsection

@section('js')
    <script>
        $(function() {
            const tableId = `{{ $tableId }}`;
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
                    url: "{{ $controller . '/gridDataSatker?pengajuanId=' . $pengajuan['id'] }}",
                    dataSrc: 'data',
                    data: (data) => {
                        const filterType = $('#dt-filter-by').data('filterby');
                        if (filterType?.trim() !== '') {
                            data.filterBy = filterType;
                        }
                        data.unitKerja = $('#unit_kerja').val();
                    }
                },
                columns: [{
                        data: 'satker'
                    },
                    {
                        data: 'pengajuan_pakaian_dinas_id',
                        render: (data, type, row) => {
                            const url =
                                `{{ $controller }}/${data}/list-satker?id_wilayah=${row.inst_satkerkd}`;

                            return `
                                <div class="d-flex justify-content-center gap-2">
                                    @include('components.showBtn', [
                                        'url' => '${url}',
                                    ])
                                </div>`;
                        }
                    },
                ]
            })

            $('#unit_kerja').on('change', function() {
                dt.ajax.reload();
            })
        })
    </script>
@endsection
