@extends('layout.main')
@section('content')
    <link rel="stylesheet" href="{{ url('assets/libs/glightbox/css/glightbox.min.css') }}">
    @include('components.breadcums', $breadcums)
    @if ($isPengajuanExpired)
        <div class="alert bg-danger border-danger text-white material-shadow" role="alert">
            <strong>Tidak Bisa Input!</strong> - Sudah Melewati batas tanggal input!!
        </div>
    @endif
    <form action="{{ $controller . '/savePengajuan' }}" method="POST" class="ajaxForm">
        <div class="row">
            <div class="col-lg-12">
                <div class="card">
                    <div class="card-header">
                        <div class="d-flex align-items-center">
                            <div class="flex-grow-1">
                                <h5 class="card-title mb-0">Pengisian Pakaian Pegawai {{ $satkerInfo['inst_nama'] ?? '' }}
                                </h5>
                            </div>
                        </div>
                    </div>
                    <div class="card-body p-4">
                        <input type="hidden" id="pengajuanId" name="pengajuanId" value="{{ $pengajuan['id'] }}" />
                        @php
                            if ($whereData['ms_satker_id'] == '00') {
                                $msSatkerId = $whereData['ms_satker_pusat_id'];
                                $isPusat = 1;
                            } else {
                                $msSatkerId = $whereData['ms_satker_id'];
                                $isPusat = 0;
                            }
                        @endphp
                        <input type="hidden" id="ms_satker_id" name="ms_satker_id" value="{{ $msSatkerId }}" />
                        <input type="hidden" id="is_pusat" name="is_pusat" value="{{ $isPusat }}" />
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
                    <div class="card-body">
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="text-center">
                                    <ul class="list-inline categories-filter animation-nav" id="filter">
                                        <li class="list-inline-item"><a class="categories active" data-filter="*">Semua</a>
                                        </li>
                                        @foreach ($pakaians as $pakaian)
                                            <li class="list-inline-item"><a class="categories"
                                                    id="filter-{{ $pakaian->spesifikasi_id }}"
                                                    data-filter=".{{ $pakaian->spesifikasi_id }}">{{ $pakaian->spesifikasi_nama }}</a>
                                            </li>
                                        @endforeach

                                    </ul>
                                </div>

                                <div class="row gallery-wrapper">
                                    @foreach ($fotos as $foto)
                                        <div class="element-item col-xxl-3 col-xl-4 col-sm-6 {{ $foto->ms_spesifikasi_pakaian_dinas_id }}"
                                            data-category="{{ $foto->ms_spesifikasi_pakaian_dinas_id }}">
                                            <div class="gallery-box card">
                                                <div class="gallery-container">
                                                    <a class="image-popup" href="{{ asset($foto->path) }}"
                                                        data-title="{{ $foto->filename }}">
                                                        <img class="gallery-img img-fluid mx-auto"
                                                            src="{{ asset($foto->path) }}" alt="" />
                                                        <div class="gallery-overlay">
                                                            <h5 class="overlay-caption">{{ $foto->filename }}</h5>
                                                        </div>
                                                    </a>
                                                </div>
                                            </div>
                                        </div>
                                    @endforeach
                                </div>
                            </div>
                        </div>
                        <!-- end row -->
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
                                    <th scope="col">Eselon</th>
                                    @foreach ($pakaians as $pakaian)
                                        <th scope="col">{{ $pakaian->spesifikasi_nama }}</th>
                                    @endforeach
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
                    <input type="hidden" name="pegawais[{{ $loop->index }}][nip]"
                        id="pegawais-{{ $pegawai->nip }}-nip" value="{{ $pegawai->nip }}" />
                    <input type="hidden" name="pegawais[{{ $loop->index }}][pangkat]"
                        id="pegawais-{{ $pegawai->nip }}-pangkat" value="{{ $pegawai->pangkat }}" />
                    <input type="hidden" name="pegawais[{{ $loop->index }}][jabatan]"
                        id="pegawais-{{ $pegawai->nip }}-jabatan" value="{{ $pegawai->jabatan }}" />
                    <input type="hidden" name="pegawais[{{ $loop->index }}][gol_kd]"
                        id="pegawais-{{ $pegawai->nip }}-gol_kd" value="{{ $pegawai->gol_kd }}" />
                    <input type="hidden" name="pegawais[{{ $loop->index }}][eselon]"
                        id="pegawais-{{ $pegawai->nip }}-eselon" value="{{ $pegawai->eselon }}" />
                    <input type="hidden" name="pegawais[{{ $loop->index }}][jenis_kelamin]"
                        id="pegawais-{{ $pegawai->nip }}-jenis_kelamin" value="{{ $pegawai->jenis_kelamin }}" />
                    <input type="hidden" name="pegawais[{{ $loop->index }}][jenis]"
                        id="pegawais-{{ $pegawai->nip }}-jenis" value="{{ $pegawai->jenis }}" />
                    <input type="hidden" name="pegawais[{{ $loop->index }}][with_hijab]"
                        id="pegawais-{{ $pegawai->nip }}-with_hijab" value="{{ $pegawai->with_hijab ?? 0 }}" />
                    @foreach ($pakaians as $pakaian)
                        @php
                            $ukuranya = $mappedUkurans[$pegawai->id][$pakaian->id] ?? '';
                        @endphp
                        <input type="hidden" name="pegawais[{{ $loop->parent->index }}][details][{{ $pakaian->id }}]"
                            id="pegawais-{{ $pegawai->nip }}-{{ $pakaian->id }}" value="{{ $ukuranya }}" />
                    @endforeach
                </div>
            @endforeach

            @include('components.activityForm', [$aktifitasHistories, $aktifitas, $aktifitasOptions])
            <div class="col-lg-12 mb-4">
                <div class="hstack gap-2 justify-content-left">
                    <a href="{{ url($controller) }}" class="btn btn-outline-primary">Kembali</a>
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
                    <input type="hidden" name="modal-pegawai-idx" id="modal-pegawai-idx">
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
                        @foreach ($pakaians as $pakaian)
                            @php
                                $additionalClass =
                                    $pakaian->subspesifikasi_gender == 'SEMUA'
                                        ? ''
                                        : 'gender gender-' . $pakaian->subspesifikasi_gender;
                            @endphp
                            <div class="row {{ $additionalClass }}">
                                <div class="col-lg-12">
                                    <div class="mb-3">
                                        <label for="inputan-{{ $pakaian->id }}" class="form-label">Ukuran
                                            {{ $pakaian->spesifikasi_nama }}</label>
                                        <select class="form-control" data-spek-id="{{ $pakaian->id }}"
                                            id="inputan-{{ $pakaian->id }}" name="{{ $pakaian->id }}">
                                            <option value="" selected>Pilih Ukuran</option>
                                            {!! $ukurans[$pakaian->spesifikasi_ukuran_group] !!}
                                        </select>
                                    </div>
                                </div>
                            </div>
                        @endforeach
                        {{-- <div class="row">
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
                        </div> --}}
                        <div class="row" id="div-hijab">
                            <div class="col-lg-12">
                                <div class="form-check">
                                    <label class="form-check-label" for="modal-with-hijab">Pakaian Muslimah</label>
                                    <input class="form-check-input" type="checkbox" name="with_hijab" value="1"
                                        id="modal-with-hijab">
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
                <div class="modal-footer">
                    <div class="col-12 d-flex  justify-content-between">
                        <button type="button" id="deletePegawai" class="btn btn-danger">Nonaktifkan</button>
                        <div>
                            <button type="button" class="btn btn-light" data-bs-dismiss="modal">Tutup</button>
                            <button type="button" id="simpanPegawai" class="btn btn-primary">Simpan</button>
                        </div>
                    </div>

                </div>
            </div>
        </div>
    </div>
@endsection

@section('js')
    <script>
        const dataPegawai = {{ Js::from($pegawais) }}
        const pakaians = {{ Js::from($pakaians) }}
        const mappedUkurans = {{ Js::from($mappedUkurans) }}
        let selectedRow = null;
        let selectedIdx = null;
        const tableId = '#pegawai-table';
        $(function() {
            const inputanElmn = pakaians.map(pakaian => {
                $(`#inputan-${pakaian.id}`).select2({
                    dropdownParent: '#inputModal'
                });
            })
            const mainCols = [{
                    data: 'nip',
                    render: (data, type, row, meta) => meta.row + 1
                },
                {
                    data: 'nama',
                    className: 'text-center',
                    render: (data, type, row) => `${row.nama}<br>${row.nip}`
                },
                {
                    data: 'jabatan',
                    className: 'text-center',
                    render: (data, type, row) => `${row.jabatan}<br>${row.pangkat}`
                },
                {
                    data: 'eselon',
                    className: 'text-center',
                },
            ];

            function changers(nip) {
                const x = nip.replace('/\//g', '_');
                // return nip;
            }

            const pakaianCols = pakaians.map(pakaian =>
                ({
                    data: 'nip',
                    className: 'text-center',
                    render: (data, type, row) => {
                        const ukuranya = mappedUkurans[`${row.id}`]?.[`${pakaian.id}`] ?? false;
                        const idElm = `#pegawais-${ changers(row.nip) }-${ pakaian.id }`
                        if (ukuranya) return ukuranya
                        //data masternya
                        const
                            defaultUkuran = dataPegawai.find(pegawai => pegawai.nip === row.nip) ??
                            false;
                        if (pakaian.spesifikasi_ukuran_group == 'CELANA') {
                            $(idElm).val(ukuranya || defaultUkuran
                                ?.ukuran_celana);
                            return row.ukuran_celana || '-';
                        }
                        if (pakaian.spesifikasi_ukuran_group == 'BAJU') {
                            $(idElm).val(ukuranya || defaultUkuran
                                ?.ukuran_baju);
                            return row.ukuran_baju || '-';
                        }
                        if (pakaian.spesifikasi_ukuran_group == 'SEPATU') {
                            $(idElm).val(ukuranya || defaultUkuran
                                ?.ukuran_sepatu);
                            return row.ukuran_sepatu || '-';
                        }
                        return '-';
                    }
                }))

            const actionCols = [{
                data: 'id',
                render: (data, type, row, meta) => {
                    const disabled = `{{ $aktifitas->canChange ? '' : 'disabled' }}`;

                    return `
                                <div class="d-flex justify-content-center gap-2">
                                    <button type="button"
                                        class="btn btn-primary waves-effect waves-light ${disabled} " onClick="ubahPegawai(${meta.row})">
                                        <i class="ri-pencil-fill"></i>
                                    </button>

                                </div>
                        `;
                },
            }]
            const pegawaiTable = $(tableId).DataTable({
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                data: dataPegawai,
                pageLength: -1,
                columns: [...mainCols, ...pakaianCols, ...actionCols]
            });

            $('#simpanPegawai').on('click', function() {
                const nip = selectedRow.nip;
                const formData = $('#inputForm').serializeFormJSON();
                $(`#pegawais-${nip}-with_hijab`).val(formData?.with_hijab || 0);
                const newUkuran = {};
                for (let field in formData) {
                    const ukuranya = formData[field];
                    $(`#pegawais-${nip}-${field}`).val(ukuranya);
                    const dataPakaian = pakaians.find(pakaian => pakaian.id == field);
                    switch (dataPakaian?.spesifikasi_ukuran_group) {
                        case 'BAJU':
                            selectedRow.ukuran_baju = ukuranya;
                            break;
                        case 'CELANA':
                            selectedRow.ukuran_celana = ukuranya;
                            break;
                        case 'SEPATU':
                            selectedRow.ukuran_sepatu = ukuranya;
                            break;
                        default:
                            break;
                    }
                    newUkuran[field] = ukuranya;
                }
                mappedUkurans[selectedRow.id] = newUkuran;
                const rowIdx = $('#modal-pegawai-idx').val();
                dataPegawai[rowIdx]['with_hijab'] = formData?.with_hijab ? 1 : 0;
                $(tableId).DataTable().row(selectedIdx).data(selectedRow).draw();
                $('#inputModal').modal('hide');
            });

            $('#deletePegawai').on('click', function() {
                const nip = selectedRow.nip;

                swal(`Yakin akan menghapus ${selectedRow.nama}?`, {
                    icon: "info",
                    dangerMode: true,
                    buttons: true,
                }).then((res) => {
                    if (!res) return;
                    dataPegawai.splice(selectedIdx, 1);
                    pegawaiTable.clear().rows.add(dataPegawai).draw();
                    $(`#inputan-${nip}`).remove();
                    $('#inputModal').modal('hide');
                });
            });

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
            $('#modal-pegawai-idx').val(rowIdx);

            $(`.gender`).toggleClass('visually-hidden', true);
            $(`.gender-${pegawai.jenis_kelamin}`).toggleClass('visually-hidden', false);

            $('#simpanPegawai').attr('data-nip', pegawai.nip);
            $('#modal-nama-nip').text(`${pegawai.nama} / ${pegawai.nip}`);
            $('#modal-jabatan-pangkat').text(`${pegawai.jabatan} / ${pegawai.pangkat}`);
            pakaians.map(pakaian => {
                let defaultUkuran = '';
                switch (pakaian.spesifikasi_ukuran_group) {
                    case 'BAJU':
                        defaultUkuran = selectedRow.ukuran_baju;
                        break;
                    case 'CELANA':
                        defaultUkuran = selectedRow.ukuran_celana;
                        break;
                    case 'SEPATU':
                        defaultUkuran = selectedRow.ukuran_sepatu;
                        break;
                    default:
                        defaultUkuran = '';
                        break;
                }
                const defaultVal = mappedUkurans[`${pegawai.id}`]?.[`${pakaian.id}`] ?? defaultUkuran;
                $(`#inputan-${pakaian.id}`).val(defaultVal).trigger('change');
            })
            $('#modal-with-hijab').prop('checked', pegawai?.with_hijab == 1);

            $('#div-hijab').toggleClass('visually-hidden', pegawai.jenis_kelamin == 'L')
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

    <script src="{{ url('assets/libs/isotope-layout/isotope.pkgd.min.js') }}"></script>
    <script src="{{ url('assets/libs/glightbox/js/glightbox.min.js') }}"></script>
    <script>
        $(function() {
            var GalleryWrapper = document.querySelector('.gallery-wrapper');
            if (GalleryWrapper) {
                var iso = new Isotope('.gallery-wrapper', {
                    itemSelector: '.element-item',
                    layoutMode: 'fitRows'
                });
            }

            // bind filter button click
            var filtersElem = document.querySelector('.categories-filter');
            if (filtersElem) {
                filtersElem.addEventListener('click', function(event) {
                    // only work with buttons
                    if (!matchesSelector(event.target, 'li a')) {
                        return;
                    }
                    var filterValue = event.target.getAttribute('data-filter');
                    if (filterValue) {
                        // use matching filter function
                        iso.arrange({
                            filter: filterValue
                        });
                    }
                });
            }

            // change is-checked class on buttons
            var buttonGroups = document.querySelectorAll('.categories-filter');
            if (buttonGroups) {
                Array.from(buttonGroups).forEach(function(btnGroup, index) {
                    var buttonGroup = btnGroup;
                    radioButtonGroup(buttonGroup);
                });
            }

            function radioButtonGroup(buttonGroup) {
                buttonGroup.addEventListener('click', function(event) {
                    // only work with buttons
                    if (!matchesSelector(event.target, 'li a')) {
                        return;
                    }
                    buttonGroup.querySelector('.active').classList.remove('active');
                    event.target.classList.add('active');
                });
            }
        })
        var lightbox = GLightbox({
            selector: '.image-popup',
            title: false,
        });
    </script>
@endsection
