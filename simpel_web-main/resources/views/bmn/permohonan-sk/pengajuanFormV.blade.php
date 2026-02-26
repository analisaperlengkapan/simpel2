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
                                <h5 class="card-title mb-0">Pengajuan {{$kategoriJudul}}</h5>
                            </div>
                        </div>
                    </div>
                    <div class="card-body p-4">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <input type="hidden" id="pengajuan_id" name="pengajuan_id" value="{{ $model['pengajuan_id'] }}">
                        <div class="row">
                            <div class="table-responsive">
                                <table class="table table-borderless mb-0">
                                    <tbody>
                                        <tr>
                                            <th class="ps-0" width="20%" scope="row">Satker</th>
                                            <td width="5%">:</td>
                                            <td class="">
                                                {{ $model['inst_nama'] ?? '-' }}
                                            </td>
                                        </tr>
                                        <tr>
                                            <th class="ps-0" width="20%" scope="row">Pembuat Pengajuan</th>
                                            <td width="5%">:</td>
                                            <td class="">
                                                {{ $model['created_by'] ?? '-' }}
                                            </td>
                                        </tr>
                                        <tr>
                                            <th class="ps-0" width="20%" scope="row">Nomor Surat Permohonan</th>
                                            <td width="5%">:</td>
                                            <td class="">
                                                {{ $model['no_surat_permohonan'] ?? '-' }}
                                            </td>
                                        </tr>
                                        <tr>
                                            <th class="ps-0" width="20%" scope="row">Tanggal Surat Permohonan</th>
                                            <td width="5%">:</td>
                                            <td class="">
                                                {{ $model['tgl_surat_permohonan'] ?? '-' }}
                                            </td>
                                        </tr>
                                        <tr>
                                            <th class="ps-0" width="20%" scope="row">Kategori</th>
                                            <td width="5%">:</td>
                                            <td class="">
                                                {{ $model['kategori'] ? $kategori[$model['kategori']].($model['kategori']==2?' ('.$jenis[$model['jenis']].')':'') : '-' }}
                                            </td>
                                        </tr>
                                        <tr>
                                            <th class="ps-0" width="20%" scope="row">File Permohonan</th>
                                            <td width="5%">:</td>
                                            <td class="">
                                                @if($model['file_surat_permohonan'])
                                                <a href="{{url($model['file_surat_permohonan'])}}" download terget="_blank"><i class="ri-download-cloud-line"></i> Download</a>
                                                @else
                                                -
                                                @endif
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
                                <h5 class="card-title mb-0">Upload File Pendukung</h5>
                            </div>
                        </div>
                    </div>
                    <div class="card-body">
                        <table class="table align-middle  mb-0 my-dt">
                            <thead class="table-light">
                                <tr>
                                    <th scope="col" width="25%">Nama File</th>
                                    <th scope="col">Nomor</th>
                                    <th scope="col">Tanggal</th>
                                    <th scope="col" width="25%">File</th>
                                </tr>
                            </thead>
                            <tbody>
                                @foreach($msFile as $key => $data)
                                <tr>
                                    <td>{{$data->nm_file}}
                                        <input type="hidden" name="ms_penghapusan_file_id[{{$key}}]" value="{{$data->ms_penghapusan_file_id}}" @if($data->kategori != 'sk' || ($data->is_validator != $isValidator)) disabled @endif/>
                                        <input type="hidden" name="file_id[{{$key}}]" value="{{$data->id}}" @if($data->kategori != 'sk' || ($data->is_validator != $isValidator)) disabled @endif/>
                                        <input type="hidden" name="file_jenis_file[{{$key}}]" value="{{$data->jenis_file}}" @if($data->kategori != 'sk' || ($data->is_validator != $isValidator)) disabled @endif/>
                                    </td>
                                    <td><input class="form-control" name="file_nomor[{{$key}}]" value="{{$data->nomor}}" @if(!$data->is_nomor) type="hidden" @endif @if($data->kategori != 'sk' || ($data->is_validator != $isValidator)) disabled @endif/></td>
                                    <td>
                                    <div class="form-icon right" @if(!$data->is_nomor) style="display:none"  @endif>
                                        <input data-provider="flatpickr" class="form-control" name="file_tanggal[{{$key}}]" value="{{$data->tanggal}}" @if($data->kategori != 'sk' || ($data->is_validator != $isValidator)) disabled @endif>
                                        <i class="ri-calendar-2-fill"></i>
                                    </div>
                                    </td>
                                    <td>
                                        <input class="form-control" type="file" name="file_file[{{$key}}]" @if($data->kategori != 'sk' || ($data->is_validator != $isValidator)) disabled @endif>
                                        @if($data->file)
                                        <a href="{{url($data->file)}}" download terget="_blank"><i class="ri-download-cloud-line"></i> Download</a>
                                        @endif
                                    </td>
                                </tr>
                                @endforeach
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>

            <div class="col-lg-12">
                <div class="card">
                    <div class="card-header">
                        <div class="d-flex align-items-center">
                            <div class="flex-grow-1">
                                <h5 class="card-title mb-0">Fotocopy Keputusan Penetapan Status Penggunaannya</h5>
                            </div>
                            @if ($canCreate && $aktifitas->canChange)
                                <div class="flex-shrink-0">
                                    <a href="#" type="button" data-bs-toggle="modal" data-bs-target="#inputModalFotocopy"
                                        class="btn btn-success btn-label waves-effect waves-light"><i
                                            class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                        Tambah
                                    </a>
                                </div>
                            @endif
                        </div>
                    </div>
                    <div class="card-body">
                        <table class="table align-middle  mb-0 my-dt" id="fc-table">
                            <thead class="table-light">
                                <tr>
                                    <th scope="col">Nomor</th>
                                    <th scope="col">Tanggal</th>
                                    <th scope="col" width="20%">File</th>
                                    <th scope="row" class="text-center" width="15%">Aksi</th>
                                </tr>
                            </thead>
                            <tbody></tbody>
                        </table>
                    </div>
                </div>
            </div>
            <div class="col-lg-12">
                <div class="card">
                    <div class="card-header">
                        <div class="d-flex align-items-center">
                            <div class="flex-grow-1">
                                <h5 class="card-title mb-0">Foto terkini BMN yang akan dihapus</h5>
                            </div>
                            @if ($canCreate && $aktifitas->canChange)
                                <div class="flex-shrink-0">
                                    <a href="#" type="button" data-bs-toggle="modal" data-bs-target="#inputModalFoto"
                                        class="btn btn-success btn-label waves-effect waves-light"><i
                                            class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                        Tambah
                                    </a>
                                </div>
                            @endif
                        </div>
                    </div>
                    <div class="card-body">
                        <table class="table align-middle  mb-0 my-dt" id="foto-table">
                            <thead class="table-light">
                                <tr>
                                    <th scope="col">Keterangan</th>
                                    <th scope="col" width="20%">File</th>
                                    <th scope="row" class="text-center" width="15%">Aksi</th>
                                </tr>
                            </thead>
                            <tbody></tbody>
                        </table>
                    </div>
                </div>
            </div>
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


    <div id="inputModalFotocopy" class="modal fade zoomIn" tabindex="-1" aria-labelledby="inputModalFotocopy" aria-hidden="true" style="display: none;">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title">Form Fotocopy Keputusan Penetapan Status Penggunaannya</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form id="inputForm">
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal2-nip" class="form-label">Nomor</label>
                                    <input type="text" class="form-control" id="fotocopy-nomor"/>
                                    <input type="hidden" class="form-control" id="fotocopy-id"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    @include('components.datepicker',[
                                        'value'=>'',
                                        'label'=>'Tanggal',
                                        'name'=>'fotocopy-tanggal'
                                        ]
                                    )
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal2-sk_penetapan" class="form-label">File</label>
                                    <input type="file" class="form-control" id="fotocopy-file"/>
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-light" data-bs-dismiss="modal" onc>Tutup</button>
                    <button id="simpanFotoCopy" class="btn btn-primary">Simpan</button>
                </div>
            </div>
        </div>
    </div>

    <div id="inputModalFoto" class="modal fade zoomIn" tabindex="-1" aria-labelledby="inputModalFotocopy" aria-hidden="true" style="display: none;">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title">Form Foto terkini BMN yang akan dihapus</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form id="inputForm">
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal2-nip" class="form-label">Keterangan</label>
                                    <input type="text" class="form-control" id="foto-ket"/>
                                    <input type="hidden" class="form-control" id="foto-id"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal2-sk_penetapan" class="form-label">File</label>
                                    <input type="file" class="form-control" id="foto-file"/>
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-light" data-bs-dismiss="modal" onc>Tutup</button>
                    <button id="simpanFoto" class="btn btn-primary">Simpan</button>
                </div>
            </div>
        </div>
    </div>
@endsection

@section('js')
    <script>
        let selectedRow = null;
        let selectedIdx = null;
        $('#modal-kode_barang').select2({
            dropdownParent: $('#inputModal')
        });
        const tableId = '#fc-table';
        $(function() {
            const fcTable = $(tableId).DataTable({
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ $controller . '/gridDataFotoCopy/'.($model['pengajuan_id']??0) }}",
                    dataSrc: 'data',
                },
                info: false,
                ordering: false,
                paging: false,
                columns: [
                    {
                        data: 'nomor',
                        render: (data, type, row) => `${data}`
                    },
                    {
                        data: 'tanggal',
                        render :(data, type, row) =>{
                            return dateFormatIndo(row.tanggal);
                        }
                    },
                    {"data": function (row, data, index, display) {

                        let url = `{{ url('${row.file}') }}`;
                        let hasil = '<a href="'+url+'" download terget="_blank"><i class="ri-download-cloud-line"></i> Download</a><br/>';
                        return hasil;
                    }},
                    {
                        data: 'id',
                        render: (data, type, row, meta) => {
                            const disabled = `{{ $aktifitas->canChange ? '' : 'disabled' }}`;
                            const aktifitas = `{{ $aktifitas->id }}`;
                            const editBtn = `<button type="button" class="btn btn-primary btn-icon waves-effect waves-light ${disabled} editPegawai" data-id="${row.id}" data-kode_barang="${row.kode_barang}" data-nm_barang="${row.nm_barang}"  data-keterangan="${row.keterangan}"  data-no_sk="${row.no_sk}" data-tgl_sk="${row.tgl_sk}"><i class="ri-pencil-fill"></i></button>`;
                            return `
                                <div class="d-flex justify-content-center gap-1">
                                    <button type="button" class="btn btn-danger btn-icon waves-effect waves-light delete ${disabled}" data-id="${row.id}" data-nomor="${row.nomor}">
                                        <i class="ri-delete-bin-5-fill"></i>
                                    </button>
                                </div>
                        `;
                        },
                    }
                ]
            });

            const fotoTable = $('#foto-table').DataTable({
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ $controller . '/gridDataFoto/'.($model['pengajuan_id']??0) }}",
                    dataSrc: 'data',
                },
                info: false,
                ordering: false,
                paging: false,
                columns: [
                    {
                        data: 'ket',
                        render: (data, type, row) => `${data}`
                    },
                    {"data": function (row, data, index, display) {

                        let url = `{{ url('${row.file}') }}`;
                        let hasil = '<a href="'+url+'" download terget="_blank"><i class="ri-download-cloud-line"></i> Download</a><br/>';
                        return hasil;
                    }},
                    {
                        data: 'id',
                        render: (data, type, row, meta) => {
                            const disabled = `{{ $aktifitas->canChange ? '' : 'disabled' }}`;
                            const aktifitas = `{{ $aktifitas->id }}`;
                            const editBtn = `<button type="button" class="btn btn-primary btn-icon waves-effect waves-light ${disabled} editPegawai" data-id="${row.id}" data-kode_barang="${row.kode_barang}" data-nm_barang="${row.nm_barang}"  data-keterangan="${row.keterangan}"  data-no_sk="${row.no_sk}" data-tgl_sk="${row.tgl_sk}"><i class="ri-pencil-fill"></i></button>`;
                            return `
                                <div class="d-flex justify-content-center gap-1">
                                    <button type="button" class="btn btn-danger btn-icon waves-effect waves-light delete ${disabled}" data-id="${row.id}">
                                        <i class="ri-delete-bin-5-fill"></i>
                                    </button>
                                </div>
                        `;
                        },
                    }
                ]
            });

            $('#fc-table').on('click', '.delete', function() {
                const id = $(this).data('id');
                const nomor = $(this).data('nomor');
                swal(`Yakin akan menghapus ${nomor}?`, {
                    icon: "info",
                    dangerMode: true,
                    buttons: true,
                }).then((res) => {
                    if (!res) return;
                    $.ajax({
                        method: "DELETE",
                        url: `{{ $controller. '/deleteFotocopy/' }}`+id,
                        success: function(){
                            notify({
                                type: "success",
                                message: "Data Berhasil Dihapus",
                            });
                        },
                        error: showError,
                    }).done(function( msg ) {
                        fcTable.ajax.reload();
                    });
                });
            }).on('click', '.editPegawai', function(){
                let id = $(this).data('id');
                let kode_barang = $(this).data('kode_barang');
                let nm_barang = $(this).data('nm_barang');
                let keterangan = $(this).data('keterangan');
                let no_sk = $(this).data('no_sk');
                let tgl_sk = $(this).data('tgl_sk');

                $('#modal2-id').val(id);
                $('#modal2-kode_barang').val(kode_barang);
                $('#modal2-nm_barang').val(nm_barang);
                $('#modal2-keterangan').val(keterangan);
                $('#modal2-no_sk').val(no_sk);
                $('#modal2-tgl_sk').val(tgl_sk);
                $('#editModal').modal('show');
            });

            $('#foto-table').on('click', '.delete', function() {
                const id = $(this).data('id');
                const nomor = $(this).data('nomor');
                swal(`Yakin akan menghapus ?`, {
                    icon: "info",
                    dangerMode: true,
                    buttons: true,
                }).then((res) => {
                    if (!res) return;
                    $.ajax({
                        method: "DELETE",
                        url: `{{ $controller. '/deleteFoto/' }}`+id,
                        success: function(){
                            notify({
                                type: "success",
                                message: "Data Berhasil Dihapus",
                            });
                        },
                        error: showError,
                    }).done(function( msg ) {
                        fotoTable.ajax.reload();
                    });
                });
            }).on('click', '.editPegawai', function(){
                let id = $(this).data('id');
                let kode_barang = $(this).data('kode_barang');
                let nm_barang = $(this).data('nm_barang');
                let keterangan = $(this).data('keterangan');
                let no_sk = $(this).data('no_sk');
                let tgl_sk = $(this).data('tgl_sk');

                $('#modal2-id').val(id);
                $('#modal2-kode_barang').val(kode_barang);
                $('#modal2-nm_barang').val(nm_barang);
                $('#modal2-keterangan').val(keterangan);
                $('#modal2-no_sk').val(no_sk);
                $('#modal2-tgl_sk').val(tgl_sk);
                $('#editModal').modal('show');
            });

            $('#inputModalFotocopy').on('hidden.bs.modal', function(){
                $('#fotocopy-id').val("");
                $('#fotocopy-nomor').val("");
                $('#fotocopy-tanggal').val("");
                $('#fotocopy-file').val("");
            });

            $('#inputModalFoto').on('hidden.bs.modal', function(){
                $('#foto-id').val("");
                $('#foto-ket').val("");
                $('#foto-file').val("");
            });

            $('#simpanFotoCopy').on('click', function() {
                let id = $('#fotocopy-id').val();
                let nomor = $('#fotocopy-nomor').val();
                let tanggal = $('#fotocopy-tanggal').val();
                let pengajuan_id = $('#pengajuan_id').val();
                var data = new FormData();
                data.append('id', id);
                data.append('nomor', nomor);
                data.append('tanggal', tanggal);
                data.append('pengajuan_id', pengajuan_id);
                if($('#fotocopy-file')[0].files.length>0){
                    var files = $('#fotocopy-file')[0].files;
                    data.append('file', files[0]);
                }
                $.ajax({
                    method: "POST",
                    url: `{{ $controller. '/saveFotocopy' }}`,
                    data: data,
                    processData: false,
                    contentType: false,
                    success: function(){
                        notify({
                            type: "success",
                            message: "Data Berhasil Disimpan",
                        });
                    },
                    error: showError,
                }).done(function( msg ) {
                    fcTable.ajax.reload();
                    $('#inputModalFotocopy').modal('hide');
                });
            });

            $('#simpanFoto').on('click', function() {
                let id = $('#foto-id').val();
                let ket = $('#foto-ket').val();
                let pengajuan_id = $('#pengajuan_id').val();
                var data = new FormData();
                data.append('id', id);
                data.append('ket', ket);
                data.append('pengajuan_id', pengajuan_id);
                if($('#foto-file')[0].files.length>0){
                    var files = $('#foto-file')[0].files;
                    data.append('file', files[0]);
                }
                $.ajax({
                    method: "POST",
                    url: `{{ $controller. '/saveFoto' }}`,
                    data: data,
                    processData: false,
                    contentType: false,
                    success: function(){
                        notify({
                            type: "success",
                            message: "Data Berhasil Disimpan",
                        });
                    },
                    error: showError,
                }).done(function( msg ) {
                    fotoTable.ajax.reload();
                    $('#inputModalFoto').modal('hide');
                });
            });
        })
    </script>
@endsection
