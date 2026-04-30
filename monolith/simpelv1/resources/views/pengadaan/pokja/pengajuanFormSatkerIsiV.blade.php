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
                                            <th class="ps-0" width="20%" scope="row">Tanggal Pengajuan</th>
                                            <td width="5%">:</td>
                                            <td class="">
                                                {{ $model['tgl_pengajuan'] ?? '-' }}
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
                            @if ($canCreate && $aktifitas->canChange)
                                <div class="flex-shrink-0">
                                    <a href="#" type="button" data-bs-toggle="modal" data-bs-target="#inputModal"
                                        class="btn btn-success btn-label waves-effect waves-light"><i
                                            class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                        Tambah
                                    </a>
                                </div>
                            @endif
                        </div>
                    </div>
                    <div class="card-body">
                        <table class="table align-middle  mb-0 my-dt" id="pegawai-table">
                            <thead class="table-light">
                                <tr>
                                    <th scope="col">Nama / NIP</th>
                                    <th scope="col">Jabatan / Pangkat</th>
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
                                        <td>{{ $history->ms_aktifitas_id==1008?'Pengajuan Terbit SK Penetapan Pokmil':$history->nama_aktifitas }}</td>
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
                                            <option value="{{ $act->id }}">{{ $act->nama == 'Pembuatan User'?'Terbit SK Penetapan Pokja Pemilihan':$act->nama }}</option>
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

    <div id="inputModal" class="modal fade zoomIn" tabindex="-1" aria-labelledby="inputModal" aria-hidden="true"
        style="display: none;">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title">Form Data Pegawai</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form id="inputForm">
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-nama-nip" class="form-label">NIP</label>
                                    <div class="input-group">
                                        <input type="text" readonly class="form-control" id="modal-nip"/>
                                        <input type="hidden" readonly class="form-control" id="modal-pengajuan_pegawai_id"/>
                                        <input type="hidden" readonly class="form-control" id="modal-pengajuan_id"/>
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
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-jabatan-pangkat" class="form-label">Nama</label>
                                    <input readonly class="form-control" id="modal-nama"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-jabatan-pangkat" class="form-label">Jabatan</label>
                                    <input readonly class="form-control" id="modal-jabatan"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-jabatan-pangkat" class="form-label">Pangkat</label>
                                    <input readonly class="form-control" id="modal-pangkat"/>
                                </div>
                            </div>
                        </div>

                        @if($kategoriJudul == 'Layanan Penetapan Pokmil')
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-jabatan-pangkat" class="form-label">Contoh Dokumen Permohonan Penetapan Pokmil</label> -
                                    <a href="https://lpse.kejaksaan.go.id/eproc4/publik/detil_special?beritaId=469670" target="_blank">Link Download</a>
                                </div>
                            </div>
                        </div>
                        @endif

                        @foreach ($msFile as $file)
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-{{ $file->jenis }}" class="form-label">{{ $file->nama }}</label>
                                    <input type="file" class="form-control" id="modal-{{ $file->jenis }}"/>
                                </div>
                            </div>
                        </div>
                        @endforeach
                    </form>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-light" data-bs-dismiss="modal" onc>Tutup</button>
                    <button id="simpanPegawai" class="btn btn-primary">Simpan</button>
                </div>
            </div>
        </div>
    </div>
    <div id="editModal" class="modal fade zoomIn" tabindex="-1" aria-labelledby="inputModal" aria-hidden="true"
        style="display: none;">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title">Form {{ $kategoriJudul }} Pegawai</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form id="inputForm">
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal2-nip" class="form-label">NIP</label>
                                    <input type="text" readonly class="form-control-plaintext" id="modal2-nip"/>
                                    <input type="hidden" class="form-control" id="modal2-id"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-jabatan-pangkat" class="form-label">Nama</label>
                                    <input readonly class="form-control-plaintext" id="modal2-nama"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-jabatan-pangkat" class="form-label">Jabatan</label>
                                    <input readonly class="form-control-plaintext" id="modal2-jabatan"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-jabatan-pangkat" class="form-label">Pangkat</label>
                                    <input readonly class="form-control-plaintext" id="modal2-pangkat"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal2-sk_penetapan" class="form-label">SK Penetapan</label>
                                    <input type="file" class="form-control" id="modal2-sk_penetapan"/>
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-light" data-bs-dismiss="modal" onc>Tutup</button>
                    <button id="simpanUserPegawai" class="btn btn-primary">Simpan</button>
                </div>
            </div>
        </div>
    </div>
    <div id="modal-pegawai" class="modal fade" tabindex="-1" aria-labelledby="myModalLabel" aria-hidden="true" style="display: none;">
        <div class="modal-dialog modal-lg">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title" id="myModalLabel">Daftar Pegawai</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"> </button>
                </div>
                <div class="modal-body">
                    <table class="table align-middle  mb-0 my-dt" id="mspegawai-table" width="100%">
                        <thead class="table-light">
                            <tr>
                                <th scope="row">#</th>
                                <th scope="col">Nama / NIP</th>
                                <th scope="col">Jabatan / Pangkat</th>
                                <th scope="row" class="text-center">Pilih</th>
                            </tr>
                        </thead>
                        <tbody></tbody>
                    </table>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-light" data-bs-dismiss="modal">Tutup</button>
                </div>

            </div><!-- /.modal-content -->
        </div><!-- /.modal-dialog -->
    </div><!-- /.modal -->
@endsection

@section('js')
    <script>
        let selectedRow = null;
        let selectedIdx = null;
        const tableId = '#pegawai-table';
        $(function() {
            const pegawaiTable = $(tableId).DataTable({
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ $controller . '/gridDataPegawai/'.$model['id'] }}",
                    dataSrc: 'data',
                },
                info: false,
                ordering: false,
                paging: false,
                columns: [
                    {
                        data: 'nama',
                        render: (data, type, row) => `${row.nama}<br>${row.nip}`
                    },
                    {
                        data: 'jabatan',
                        render: (data, type, row) => `${row.jabatan}<br>${row.pangkat}`
                    },
                    {"data": function (row, data, index, display) {

                        let file = row.filenya?(row.filenya).split('|#|'):[];
                        let hasil = "";
                        if(file.length>0){
                            $.each( file, function( i, val ) {
                                let a = val.split('---');
                                let url = `{{ url('${a[1]}') }}`;
                                hasil += '<a href="'+url+'" download terget="_blank"><i class="ri-download-cloud-line"></i> '+a[0]+'</a><br/>';
                            });
                        }
                        return hasil;
                    }},
                    {
                        data: 'id',
                        render: (data, type, row, meta) => {
                            const disabled = `{{ $aktifitas->canChange ? '' : 'disabled' }}`;
                            const aktifitas = `{{ $aktifitas->id }}`;
                            const editBtn = aktifitas==1008?`<button type="button" class="btn btn-primary btn-icon waves-effect waves-light ${disabled} editPegawai" data-id="${row.id}" data-sk_pengangkatan="${row.sk_pengangkatan}" data-nama="${row.nama}"  data-nip="${row.nip}"  data-pangkat="${row.pangkat}"  data-jabatan="${row.jabatan}"><i class="ri-pencil-fill"></i></button>`:'';
                            return `
                                <div class="d-flex justify-content-center gap-1">
                                    ${editBtn}
                                    <button type="button" class="btn btn-danger btn-icon waves-effect waves-light deletePegawai ${disabled}" data-id="${row.id}" data-nama="${row.nama}">
                                        <i class="ri-delete-bin-5-fill"></i>
                                    </button>
                                </div>
                        `;
                        },
                    }
                ]
            });

            const msPegawaiTable = $('#mspegawai-table').DataTable({
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ $controller . '/gridDataMsPegawai/'.$model['id'] }}",
                    dataSrc: 'data',
                },
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
                        data: 'id',
                        render: (data, type, row, meta) => {
                            const disabled = `{{ $aktifitas->canChange ? '' : 'disabled' }}`;
                            return `
                            <button type="button" class="btn btn-info btn-icon waves-effect waves-light pilihPegawai ${disabled}" data-nip="${row.nip}" data-nama="${row.nama}" data-jabatan="${row.jabatan}" data-pangkat="${row.pangkat}">
                                <i class=" ri-checkbox-line"></i>
                            </button>
                        `;
                        },
                    }
                ]
            });

            $('#searchNip').on('click', function(){
                msPegawaiTable.ajax.reload();
                $('#modal-pegawai').modal('show');
            });

            $('#mspegawai-table').on('click', '.pilihPegawai', function(){
                let nip = $(this).data('nip');
                let nama = $(this).data('nama');
                let pangkat = $(this).data('pangkat');
                let jabatan = $(this).data('jabatan');
                let pengajuan_id = $('#id').val();

                $('#modal-nip').val(nip);
                $('#modal-nama').val(nama);
                $('#modal-pangkat').val(pangkat);
                $('#modal-jabatan').val(jabatan);
                $('#modal-pengajuan_id').val(pengajuan_id);

                $('#modal-pegawai').modal('hide');
            });

            $('#inputModal').on('hidden.bs.modal', function(){
                $('#modal-nip').val("");
                $('#modal-nama').val("");
                $('#modal-pangkat').val("");
                $('#modal-jabatan').val("");
                $('#modal-pengajuan_id').val("");
            });

            $('#simpanPegawai').on('click', function() {
                let pengajuan_pegawai_id = $('#modal-pengajuan_pegawai_id').val();
                let nip = $('#modal-nip').val();
                let nama = $('#modal-nama').val();
                let pangkat = $('#modal-pangkat').val();
                let jabatan = $('#modal-jabatan').val();
                let pengajuan_id = $('#modal-pengajuan_id').val();
                var data = new FormData();
                data.append('pengajuan_pegawai_id', pengajuan_pegawai_id);
                data.append('nip', nip);
                data.append('nama', nama);
                data.append('pangkat', pangkat);
                data.append('jabatan', jabatan);
                data.append('pengajuan_id', pengajuan_id);
                @foreach ($msFile as $file)
                if($('#modal-{{ $file->jenis }}')[0].files.length>0){
                    var files = $('#modal-{{ $file->jenis }}')[0].files;
                    data.append('{{ $file->jenis }}', files[0]);
                }
                @endforeach

                $.ajax({
                    method: "POST",
                    url: `{{ $controller. '/savePegawai' }}`,
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
                    pegawaiTable.ajax.reload();
                    $('#inputModal').modal('hide');
                });
            });

            $('#pegawai-table').on('click', '.deletePegawai', function() {
                const id = $(this).data('id');
                const pegawai = $(this).data('nama');
                swal(`Yakin akan menghapus ${pegawai}?`, {
                    icon: "info",
                    dangerMode: true,
                    buttons: true,
                }).then((res) => {
                    if (!res) return;
                    $.ajax({
                        method: "DELETE",
                        url: `{{ $controller. '/deletePegawai/' }}`+id,
                        success: function(){
                            notify({
                                type: "success",
                                message: "Data Berhasil Dihapus",
                            });
                        },
                        error: showError,
                    }).done(function( msg ) {
                        pegawaiTable.ajax.reload();
                    });
                });
            }).on('click', '.editPegawai', function(){
                let id = $(this).data('id');
                let nip = $(this).data('nip');
                let pangkat = $(this).data('pangkat');
                let jabatan = $(this).data('jabatan');
                let nama = $(this).data('nama');

                $('#modal2-id').val(id);
                $('#modal2-nip').val(nip);
                $('#modal2-nama').val(nama);
                $('#modal2-pangkat').val(pangkat);
                $('#modal2-jabatan').val(jabatan);
                $('#editModal').modal('show');
            });

            $('#editModal').on('hidden.bs.modal', function(){
                $('#modal2-id').val("");
                $('#modal2-nip').val("");
                $('#modal2-nama').val("");
                $('#modal2-pangkat').val("");
                $('#modal2-jabatan').val("");
            });

            $('#simpanUserPegawai').on('click', function() {
                let id = $('#modal2-id').val();
                let sk_penetapan = $('#modal2-sk_penetapan').val();
                var data = new FormData();
                data.append('id', id);
                if($('#modal2-sk_penetapan')[0].files.length>0){
                    var files = $('#modal2-sk_penetapan')[0].files;
                    data.append('sk_penetapan', files[0]);
                }
                $.ajax({
                    method: "POST",
                    url: `{{ $controller. '/saveSkPenetapan' }}`,
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
                    pegawaiTable.ajax.reload();
                    $('#editModal').modal('hide');
                });
            });
        })
    </script>
@endsection
