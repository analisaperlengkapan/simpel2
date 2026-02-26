@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <form action="{{ $controller }}" method="POST" class="ajaxForm">
        <div class="row">
            <div class="col-lg-12">
                <div class="card">
                    <div class="card-header">
                        <div class="d-flex align-items-center">
                            <div class="flex-grow-1">
                                <h5 class="card-title mb-0">{{ $isNew ? 'Tambah' : 'Edit' }} User SIMPLE</h5>
                            </div>
                        </div>
                    </div>
                    <div class="card-body p-4">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="username" class="form-label">NIP</label>
                                    <div class="input-group">
                                        <input type="number" class="form-control" placeholder="Cari NIP" id="nip"
                                            name="username" pattern="[0-9]" inputmode="numeric"
                                            value="{{ $model['username'] ?? '' }}" />
                                        <button class="input-group-text btn-dark btn" id="searchNip" type="button">
                                            <span class="">
                                                <i class="ri-search-line align-bottom me-1"></i>
                                                Cari
                                            </span>
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="table-responsive">
                                @php
                                    $takenField = ['nama', 'satker', 'pangkat', 'jabatan'];
                                @endphp
                                <table class="table table-borderless mb-0">
                                    <tbody>
                                        @foreach ($takenField as $item)
                                            <tr>
                                                <th class="ps-0" width="10%" scope="row">{{ ucwords($item) }}</th>
                                                <td width="5%">:</td>
                                                <td class="text-muted" id="peg-{{ $item }}">
                                                    {{ $model[$item] ?? '-' }}
                                                </td>
                                            </tr>
                                        @endforeach
                                        @if ($isNew)
                                            <tr>
                                                <th class="ps-0" width="10%" scope="row">Password</th>
                                                <td width="5%">:</td>
                                                <td class="text-muted">
                                                    <input type="password" class="form-control" style="width: 35%"
                                                        id="password" name="password" required placeholder="Password"
                                                        value="{{ $model['password'] ?? '' }}">
                                                </td>
                                            </tr>
                                        @elseif($canChangePassword)
                                            <tr>
                                                <th class="ps-0" width="10%" scope="row">Password</th>
                                                <td width="5%">:</td>
                                                <td class="text-muted">
                                                    @include('components.changePasswordModal', [
                                                        'user_id' => $model['id'],
                                                    ])
                                                    <button type="button" class="btn btn-secondary" data-bs-toggle="modal"
                                                        data-bs-target="#changePasswordModal">Ubah Password</button>

                                                </td>
                                            </tr>
                                        @endif

                                        @if ($canResetPassword)
                                            <tr>
                                                <th class="ps-0" width="10%" scope="row">Password</th>
                                                <td width="5%">:</td>
                                                <td class="text-muted">
                                                    <button type="button" id="resetPass" class="btn btn-secondary">Reset
                                                        Password</button>
                                                </td>
                                            </tr>
                                        @endif
                                    </tbody>
                                </table>
                            </div>
                        </div>

                        @if (!$isNew && session('userData.is_superadmin') == 1 && !empty($model['google2fa_secret']))
                            <!-- Tombol trigger modal -->
                            <button type="button" class="btn btn-danger mb-3" data-bs-toggle="modal" data-bs-target="#modalNonaktifkan2FA">
                                Nonaktifkan 2FA
                            </button>
                            <!-- Modal Konfirmasi -->
                            <div class="modal fade" id="modalNonaktifkan2FA" tabindex="-1" aria-labelledby="modalNonaktifkan2FALabel" aria-hidden="true">
                                <div class="modal-dialog">
                                    <div class="modal-content">
                                        <div class="modal-header">
                                            <h5 class="modal-title" id="modalNonaktifkan2FALabel">Konfirmasi Nonaktifkan 2FA</h5>
                                            <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                                        </div>
                                        <div class="modal-body">
                                            Apakah Anda yakin ingin menonaktifkan 2FA untuk user ini?
                                        </div>
                                        <div class="modal-footer">
                                            <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">Batal</button>
                                            <form id="formNonaktifkan2FA" method="POST" action="{{ url('/pengguna/nonaktifkan-2fa/'.$model['id']) }}" style="display:inline">
                                                @csrf
                                                <button type="submit" class="btn btn-danger">Ya, Nonaktifkan</button>
                                            </form>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        @endif

                    </div>
                </div>
            </div>

            <div class="col-lg-12">
                <div class="card">
                    <div class="card-header">
                        <div class="d-flex align-items-center">
                            <div class="flex-grow-1">
                                <h5 class="card-title mb-0">Roles</h5>
                            </div>
                        </div>
                    </div>
                    <div class="card-body">
                        <div class="row">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <select class="form-control" placeholder="Pilih Role" data-choices multiple
                                        data-choices-removeItem data-choices-search-false name="roles[]" id="roles">
                                        {!! $roleOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <hr>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2 justify-content-left">
                                    <a href="{{ $controller }}" class="btn btn-outline-primary">Kembali</a>
                                    <button type="submit" class="btn btn-primary">Simpan</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </form>
    <div id="roleModal" class="modal fade zoomIn" tabindex="-1" aria-labelledby="roleModal" aria-hidden="true"
        style="display: none;">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title">Tambah Role</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form id="roleForm">
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="ms_role_id" class="form-label">Role</label>
                                    <select class="form-control" data-choices data-choices-search-false name="ms_role_id"
                                        id="ms_role_id">
                                        <option value="" selected>Pilih Role</option>
                                        {!! $roleOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row" style="display:none;" id="satkerSelection">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <input type="hidden" id="ms_satker_pusat_id"
                                        value="{{ $model['satker_pusat'] ?? '' }}" />
                                    <label for="ms_satker_id" class="form-label">Satker</label>
                                    <select class="form-control" name="ms_satker_id" id="ms_satker_id">
                                        <option value="">Pilih Satker</option>
                                        {!! $satkerOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-light" data-bs-dismiss="modal" onc>Tutup</button>
                    <button type="button" id="roleAdd" class="btn btn-primary ">Tambah</button>
                </div>

            </div>
        </div>
    </div>
@endsection

@section('js')
    <script>
        const satkers = {{ Js::from($satkers) }}

        function deleteRow(id) {
            $(`#roleRow-${id}`).remove();
        }

        function appendRow({
            no,
            id,
            role,
            satker,
            msSatkerId,
            msSatkerPusatId,
            msRoleId
        }) {
            const elm = `
                <tr id="roleRow-${id}"">
                    <td>${no}</td>
                    <td>${role}</td>
                    <td>${satker}</td>
                    <td>
                        <div class="d-flex justify-content-center gap-2">
                            <button type="button" class="btn btn-danger waves-effect waves-light" onClick="deleteRow('${id}')">
                                <i class="ri-delete-bin-5-fill"></i>
                            </button>
                        </div>
                        <div>
                            <input type="hidden" name="roles[${id}][ms_role_id]" value="${msRoleId}">
                            <input type="hidden" name="roles[${id}][ms_satker_id]" value="${msSatkerId}">
                        </div>
                    </td>
                </tr>`

            $('#role-table tbody').append(elm);
            $('#roleModal').modal('hide')
        }
        let unitKerja = `{{ $model['satker_pusat'] ?? '' }}`;
        $(function() {
            // $('#selectSatker').select2({
            //     tags: true,
            //     dropdownParent: $('#roleModal')
            // });

            $('#ms_role_id').on('change', function() {
                const id = $(this).val();
                const text = $(this).find('option:selected').text();
                let invisible = true;
                if (text == 'Pelaksana Daerah' || text == 'Validator Daerah') invisible = false;
                $('#satkerSelection').toggleClass('invisible', invisible)
            })

            $('#roleAdd').on('click', function() {
                const roleElm = $('#ms_role_id');
                const satkerElm = $('#ms_satker_id');

                const msRoleId = roleElm.val();
                if (!msRoleId) return;
                const role = roleElm.find('option:selected').text();

                const msSatkerId = satkerElm.val();
                const satker = msSatkerId == '' ? '' : satkerElm.find('option:selected').text();
                const no = $('#role-table tbody>tr').length + 1

                appendRow({
                    role,
                    satker: msSatkerId == '00' ? unitKerja : satker,
                    msRoleId,
                    msSatkerId,
                    no,
                    id: shortId()
                })

            });

            $('#searchNip').on('click', function() {
                const nip = $('#nip').val();
                if (!nip) return;
                const takenField = {{ JS::from($takenField) }};
                const fNip = nip.replace(/\s/g, '');

                $.get(`/searchPegawai/${fNip}`)
                    .fail((a) =>
                        notify({
                            type: 'danger',
                            message: a.responseJSON.message
                        })
                    ).done(data => {
                        if (!data.body) return;
                        $('#ms_satker_id').val(data.body.inst_satkerkd);
                        $('#ms_satker_pusat_id').val(data.body.unitkerja_idk);
                        unitKerja = data.body.unitkerja_nama;
                        for (let field in data.body) {
                            if (takenField.includes(field)) {
                                const elm = `#peg-${field}`
                                const val = data.body[field] || null;
                                $(elm).text(val);
                            }
                        }
                    })
            })
            $('#resetPass').on('click', function() {
                swal(`Yakin akan Mereset Password ?`, {
                    icon: "info",
                    dangerMode: true,
                    buttons: true,
                }).then(val => {
                    if (!val) return;
                    const nip = $('#nip').val();
                    $.post('/auth/resetPassword', {
                            nip
                        })
                        .done((mgs) => notify(mgs))
                        .fail((err) => notify({
                            type: 'warning',
                            message: err?.responseJSON?.message || 'Gagal'
                        }))
                })
            })
        })
    </script>
@endsection
