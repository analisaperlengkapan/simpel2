@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <form action="{{ $controller . '/savePengajuan' }}" method="POST" class="ajaxForm">
        <div class="row">
            <div class="col-lg-12">
                <div class="card">
                    <div class="card-header">
                        <div class="d-flex align-items-center">
                            <div class="flex-grow-1">
                                <h5 class="card-title mb-0">Pengisian Pakaian Pegawai</h5>
                            </div>
                        </div>
                    </div>
                    <div class="card-body p-4">
                        <input type="hidden" id="pengajuanId" name="pengajuanId" value="{{ $pengajuan['id'] }}">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
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
                                <h5 class="card-title mb-0">Pegawai</h5>
                            </div>
                        </div>
                    </div>
                    <div class="card-body">
                        <table class="table align-middle  mb-0 my-dt" id="pegawai-table">
                            <thead class="table-light">
                                <tr>
                                    <th scope="row">#</th>
                                    <th scope="col">Nama / NIP</th>
                                    <th scope="col">Jabatan / Pangkat</th>
                                    <th scope="col">Ukuran Baju</th>
                                    <th scope="col">Ukuran Celana</th>
                                    <th scope="col">Ukuran Sepatu</th>
                                    <th scope="row" class="text-center">Aksi</th>
                                </tr>
                            </thead>
                            <tbody></tbody>
                        </table>
                    </div>
                </div>
            </div>
            @foreach ($pegawais as $pegawai)
                <div id="inputan-{{ $pegawai->nip }}">
                    <input type="hidden" name="pegawais[{{ $loop->index }}][nip]" id="pegawais-{{ $pegawai->nip }}-nip"
                        value="{{ $pegawai->nip }}" />
                    <input type="hidden" name="pegawais[{{ $loop->index }}][pangkat]"
                        id="pegawais-{{ $pegawai->nip }}-pangkat" value="{{ $pegawai->pangkat }}" />
                    <input type="hidden" name="pegawais[{{ $loop->index }}][jabatan]"
                        id="pegawais-{{ $pegawai->nip }}-jabatan" value="{{ $pegawai->jabatan }}" />
                    <input type="hidden" name="pegawais[{{ $loop->index }}][ukuran_baju]"
                        id="pegawais-{{ $pegawai->nip }}-ukuran_baju" value="{{ $pegawai->ukuran_baju ?? '' }}" />
                    <input type="hidden" name="pegawais[{{ $loop->index }}][ukuran_celana]"
                        id="pegawais-{{ $pegawai->nip }}-ukuran_celana" value="{{ $pegawai->ukuran_celana ?? '' }}" />
                    <input type="hidden" name="pegawais[{{ $loop->index }}][ukuran_sepatu]"
                        id="pegawais-{{ $pegawai->nip }}-ukuran_sepatu" value="{{ $pegawai->ukuran_sepatu ?? '' }}" />
                    <input type="hidden" name="pegawais[{{ $loop->index }}][with_hijab]"
                        id="pegawais-{{ $pegawai->nip }}-with_hijab" value="{{ $pegawai->with_hijab ?? 0 }}" />
                </div>
            @endforeach
            <div class="col-lg-12">
                <div class="card">
                    <div class="card-header">
                        <div class="d-flex align-items-center">
                            <div class="flex-grow-1">
                                <h5 class="card-title mb-0">Aktifitas Pengajuan</h5>
                            </div>
                        </div>
                    </div>
                    <div class="card-body">
                        <table class="table align-middle mb-0 my-dt">
                            <thead class="table-light">
                                <tr>
                                    <th scope="row">Waktu</th>
                                    <th scope="col">Nama</th>
                                    <th scope="col">Jabatan / Pangkat</th>
                                    <th scope="col">Role</th>
                                    <th scope="col">Aktifitas</th>
                                    <th scope="col">Komentar</th>
                                </tr>
                            </thead>
                            <tbody>
                                @forelse ($aktifitasHistories as $history)
                                    <tr class="text-center">
                                        <td>{{ MyHelper::dateFormat($history->created_at) }}</td>
                                        <td>{{ $history->nama }}</td>
                                        <td class="text-left">{{ $history->jabatan }} <br> {{ $history->pangkat }}</td>
                                        <td>{{ $history->role }}</td>
                                        <td>{{ $history->nama_aktifitas }}</td>
                                        <td>{{ $history->komentar }}</td>
                                    </tr>
                                @empty
                                    <tr>
                                        <td colspan="6" class="text-center">Belum ada Data</td>
                                    </tr>
                                @endforelse
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
            @if ($aktifitas->canChange)
                <div class="col-lg-12">
                    <div class="card">
                        <div class="card-header">
                            <div class="d-flex align-items-center">
                                <div class="flex-grow-1">
                                    <h5 class="card-title mb-0">Aksi</h5>
                                </div>
                            </div>
                        </div>
                        <div class="card-body p-4">
                            <div class="row">
                                <div class="mb-3 col-lg-6">
                                    <select class="form-control" data-choices data-choices-search name="ms_aktifitas_id">
                                        @foreach ($aktifitasOptions as $act)
                                            <option value="{{ $act->id }}">{{ $act->nama }}</option>
                                        @endforeach
                                    </select>
                                </div>
                            </div>

                            <div class="row">
                                <div class="mb-3 col-lg-6">
                                    <textarea name="komentar" class="form-control" rows="5" placeholder="Tuliskan Komentar"></textarea>
                                </div>
                            </div>

                        </div>
                    </div>
                </div>
            @endif
            <div class="col-lg-12 mb-4">
                <div class="hstack gap-2 justify-content-left">
                    <a href="{{ url('settings/user') }}" class="btn btn-outline-primary">Kembali</a>
                    @if ($aktifitas->canChange)
                        <button type="submit" class="btn btn-primary">
                            Simpan
                        </button>
                    @endif
                </div>
            </div>
        </div>
    </form>

    <div id="inputModal" class="modal fade zoomIn" tabindex="-1" aria-labelledby="inputModal" aria-hidden="true"
        style="display: none;">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title">Tambah Data Pegawai</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form id="inputForm">
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-nama-nip" class="form-label">Nama / NIP</label>
                                    <p class="text-muted" id="modal-nama-nip"></p>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-jabatan-pangkat" class="form-label">Jabatan / Pangkat</label>
                                    <p class="text-muted" id="modal-jabatan-pangkat"></p>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-ukuran-baju" class="form-label">Ukuran Baju</label>
                                    <select class="form-control" id="modal-ukuran-baju">
                                        <option value="" selected>Pilih Ukuran</option>
                                        {!! $ukuranBajuOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-ukuran-celana" class="form-label">Ukuran Celana</label>
                                    <select class="form-control" id="modal-ukuran-celana">
                                        <option value="" selected>Pilih Ukuran</option>
                                        {!! $ukuranCelanaOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-ukuran-sepatu" class="form-label">Ukuran Sepatu</label>
                                    <select class="form-control" id="modal-ukuran-sepatu">
                                        <option value="" selected>Pilih Ukuran</option>
                                        {!! $ukuranSepatuOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row" id="div-hijab">
                            <div class="col-lg-12">
                                <div class="form-check">
                                    <label class="form-check-label" for="modal-with-hijab">Dengan Jilbab</label>
                                    <input class="form-check-input" type="checkbox" name="with-hijab" value="1"
                                        id="modal-with-hijab">
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-light" data-bs-dismiss="modal" onc>Tutup</button>
                    <button type="button" id="simpanPegawai" class="btn btn-primary">Simpan</button>
                </div>

            </div>
        </div>
    </div>
@endsection

@section('js')
    <script>
        const dataPegawai = {{ Js::from($pegawais) }}
        let selectedRow = null;
        let selectedIdx = null;
        const tableId = '#pegawai-table';
        $(function() {
            $('#modal-ukuran-celana, #modal-ukuran-baju, #modal-ukuran-sepatu').select2({
                dropdownParent: '#inputModal'
            });
            const pegawaiTable = $(tableId).DataTable({
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                data: dataPegawai,
                columns: [{
                        data: 'nip',
                        render: (data, type, row, meta) => meta.row + 1
                    },
                    {
                        data: 'nama',
                        render: (data, type, row) => `${row.nama}<br>${row.nip}`
                    },
                    {
                        data: 'jabatan',
                        render: (data, type, row) => `${row.jabatan}<br>${row.pangkat}`
                    },
                    {
                        data: 'ukuran_baju',
                        className: 'text-center',
                        render: (data, type, row) => row.ukuran_baju || '-'

                    },
                    {
                        data: 'ukuran_celana',
                        className: 'text-center',
                        render: (data, type, row) => row.ukuran_celana || '-'

                    },
                    {
                        data: 'ukuran_sepatu',
                        className: 'text-center',
                        render: (data, type, row) => row.ukuran_sepatu || '-'
                    },
                    {
                        data: 'id',
                        render: (data, type, row, meta) => {
                            const disabled = `{{ $aktifitas->canChange ? '' : 'disabled' }}`;

                            return `
                                <div class="d-flex justify-content-center gap-2">
                                    <button type="button"
                                        class="btn btn-primary waves-effect waves-light ${disabled} " onClick="ubahPegawai(${meta.row})">
                                        <i class="ri-pencil-fill"></i>
                                    </button>
                                    <button type="button" class="btn btn-danger waves-effect waves-light deletePegawai ${disabled}" data-idx="${meta.row}">
                                        <i class="ri-delete-bin-5-fill"></i>
                                    </button>
                                </div>
                        `;
                        },
                    }
                ]
            });

            $('#simpanPegawai').on('click', function() {
                const nip = selectedRow.nip;
                const baju = $('#modal-ukuran-baju').val();
                const celana = $('#modal-ukuran-celana').val();
                const sepatu = $('#modal-ukuran-sepatu').val();

                $(`#pegawais-${nip}-ukuran_baju`).val(baju);
                $(`#pegawais-${nip}-ukuran_celana`).val(celana);
                $(`#pegawais-${nip}-ukuran_sepatu`).val(sepatu);
                selectedRow.ukuran_baju = baju;
                selectedRow.ukuran_celana = celana;
                selectedRow.ukuran_sepatu = sepatu;
                $(tableId).DataTable().row(selectedIdx).data(selectedRow).draw();
                $('#inputModal').modal('hide');
            })
            $(document).on('click', '.deletePegawai', function() {
                const rowIdx = $(this).data('idx');
                const pegawai = dataPegawai[rowIdx];
                swal(`Yakin akan menghapus ${pegawai.nama}?`, {
                    icon: "info",
                    dangerMode: true,
                    buttons: true,
                }).then((res) => {
                    if (!res) return;
                    $(`#inputan-${pegawai.nip}`).remove();
                    pegawaiTable.row($(this).parents('tr')).remove().draw();
                });
            })
        })

        function ubahPegawai(rowIdx) {
            const pegawai = dataPegawai[rowIdx];
            selectedRow = $(tableId).DataTable().row(rowIdx).data();
            selectedIdx = rowIdx;


            $('#simpanPegawai').attr('data-nip', pegawai.nip);
            $('#modal-nama-nip').text(`${pegawai.nama} / ${pegawai.nip}`);
            $('#modal-jabatan-pangkat').text(`${pegawai.jabatan} / ${pegawai.pangkat}`);

            $('#modal-ukuran-baju').val(pegawai.ukuran_baju || '').trigger('change');
            $('#modal-ukuran-celana').val(pegawai.ukuran_celana || '').trigger('change');
            $('#modal-ukuran-sepatu').val(pegawai.ukuran_sepatu || '').trigger('change');
            $('#modal-with-hijab').attr('checked', pegawai?.with_hijab == 1);

            $('#div-hijab').toggleClass('invisible', pegawai.jenis_kelamin == 'L')
            $('#inputModal').modal('show');
        }

        // function deletePegawai(rowIdx) {
        //     const pegawai = dataPegawai[rowIdx];
        //     swal(`Yakin akan menghapus ${pegawai.nama}?`, {
        //         icon: "info",
        //         dangerMode: true,
        //         buttons: true,
        //     }).then((res) => {
        //         if (!res) return;
        //         $(`#inputan-${pegawai.nip}`).remove();
        //         $(tableId).DataTable().row(selectedIdx).remove().draw();
        //     });
        // }

        // function deleteRow(id) {
        //     $(`#pegawaiRow-${id}`).remove();
        // }

        // function appendRow({
        //     no,
        //     id,
        //     nama,
        //     nip,
        //     ukuran,
        // }) {
        //     const emp = $('#pegawaiEmpty');
        //     const elm = `
    //         <tr id="pegawaiRow-${id}"">
    //             <td>${emp.length > 0? no -1 : no  }</td>
    //             <td>${nip}</td>
    //             <td>${nama}</td>
    //             <td>${ukuran}</td>
    //             <td>
    //                 <div class="d-flex justify-content-center gap-2">
    //                     <button type="button" class="btn btn-danger waves-effect waves-light" onClick="deleteRow('${id}')">
    //                         <i class="ri-delete-bin-5-fill"></i>
    //                     </button>
    //                 </div>
    //                 <div>
    //                     <input type="hidden" name="pegawais[${id}][nip]" value="${nip}">
    //                     <input type="hidden" name="pegawais[${id}][ukuran]" value="${ukuran}">
    //                 </div>
    //             </td>
    //         </tr>`;
        //     $(tableIdtbody').append(elm);
        //     emp.remove();
        //     $('#inputModal').modal('hide')
        //     $('#nip').val("");
        //     $('#ukuran').val("");
        // }
    </script>
@endsection
