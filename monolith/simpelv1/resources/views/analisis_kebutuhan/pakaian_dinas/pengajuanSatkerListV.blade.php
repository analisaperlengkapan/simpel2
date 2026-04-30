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
                    <div class="row">
                        <div class="d-flex align-items-center">
                            <div class="flex-grow-1">
                                <button id="checkAll" type="button"
                                    class="btn btn-primary btn-label waves-effect waves-light"><i
                                        class="ri-check-line label-icon align-middle fs-16 me-2"></i>
                                    Pilih Semua
                                </button>
                            </div>
                            <div class="flex-shrink-0">
                                <button type="button" data-aktifitas = "{{ $approveAct }}"
                                    class="btn btn-success btn-label waves-effect waves-light validatorAction"><i
                                        class="ri-check-line label-icon align-middle fs-16 me-2"></i>
                                    Setuju
                                </button>
                                <button type="button" data-aktifitas="{{ $rejectAct }}"
                                    class="btn btn-danger btn-label waves-effect waves-light validatorAction"><i
                                        class="ri-cross-line label-icon align-middle fs-16 me-2"></i>
                                    Revisi
                                </button>
                            </div>
                        </div>
                    </div>
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt mt-3"
                        style="width:100%">
                        <thead>
                            <tr>
                                <th>#</th>
                                <th>Nama</th>
                                <th>Status</th>
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
            $("#checkAll").on('click', function() {
                $('.satker-check:not(:disabled)').prop('checked', true);

            })


            const id_wilayah = `{{ request('id_wilayah') }}`;
            const tableId = `{{ $tableId }}`;

            const enableOn = @json($enableCheckOn);
            const dt = $('#' + tableId).DataTable({
                pageLength: 100,
                lengthChange: false,
                serverSide: true,
                processing: true,
                deferRender: true,
                ordering: false,
                dom: dtLayout,
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: `{{ $controller . '/gridDataSatker?pengajuanId=' . $pengajuan['id'] }}&id_wilayah=${id_wilayah}`,
                    dataSrc: 'data',
                    data: (data) => {
                        const filterType = $('#dt-filter-by').data('filterby');
                        if (filterType?.trim() !== '') {
                            data.filterBy = filterType;
                        }
                    }
                },
                columns: [{
                        data: 'id',
                        className: 'text-center',
                        render: (data, type, row) => {
                            const disabled = enableOn.includes(row.ms_aktifitas_id) ? '' :
                                'disabled';
                            return `<input type="checkbox" value="${data}" class="satker-check form-check-input" ${disabled}>`
                        }
                    }, {
                        data: 'satker'
                    },
                    {
                        data: 'status',
                        className: 'text-center'
                    },
                    {
                        data: 'pengajuan_pakaian_dinas_id',
                        render: (data, type, row) => {
                            const url = `{{ $controller . '/' . '${data}' }}`;
                            const satker = `{{ session('userData.current_role.ms_satker_id') }}` ==
                                row.inst_satkerkd ? '' :
                                `?satker=${row.inst_satkerkd}&isPusat=${row.is_pusat}`;
                            if (!data) return '';
                            return `
                                <div class="d-flex justify-content-center gap-2">
                                    @include('components.showBtn', [
                                        'url' => '${url}/edit${satker}',
                                    ])
                                </div>`;
                        }
                    },
                ]
            })

            $('.validatorAction').on('click', function() {
                const ms_aktifitas_id = $(this).data('aktifitas');
                const pengajuanSatkerId = [];
                const elms = $('.satker-check:checked');
                if (elms.length < 1) {
                    swal('Info', 'Pilih minimal 1 Satker', 'info');
                    return;
                }
                elms.each(function() {
                    pengajuanSatkerId.push($(this).val());
                })
                const pengajuanId = `{{ $pengajuan['id'] }}`;
                $.post(`{{ $controller }}/validatorAction`, {
                    pengajuanSatkerId,
                    ms_aktifitas_id,
                    pengajuanId
                }, function(res) {
                    dt.ajax.reload();
                })
            })
        })
    </script>
@endsection
