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
                                        <tr>
                                            <th class="ps-0" width="20%" scope="row">Nama Pengajuan</th>
                                            <td width="5%">:</td>
                                            <td class="">
                                                {{ $model['nama'] ?? '-' }}
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
                                <h5 class="card-title mb-0">Daftar BMN Dihapus</h5>
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
                                    <th scope="col">Kode Barang</th>
                                    <th scope="col">Nama Barang</th>
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

    <div id="inputModal" class="modal fade zoomIn" tabindex="-1" aria-labelledby="inputModal" aria-hidden="true"
        style="display: none;">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title">Form Data BMN</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form id="inputForm">
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-jabatan-pangkat" class="form-label">Barang</label>
                                    <select id="modal-kode_barang" name="modal-kode_barang" class="form-control selectTwo">
                                    @foreach($listBarang as $item)
                                        <option value="{{ $item->kode_barang }}">{{ $item->kode_barang.' - '.$item->nama_barang }}</option>
                                    @endforeach
                                    </select>
                                    <input type="hidden" readonly class="form-control" id="modal-pengajuan_asset_id"/>
                                    <input type="hidden" readonly class="form-control" id="modal-pengajuan_id" value="{{ $model['id'] }}"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-keterangan" class="form-label">Keterangan</label>
                                    <input class="form-control" id="modal-keterangan"/>
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-light" data-bs-dismiss="modal">Tutup</button>
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
                                    <label for="modal2-nip" class="form-label">Kode Barang</label>
                                    <input type="text" readonly class="form-control-plaintext" id="modal2-kode_barang"/>
                                    <input type="hidden" class="form-control" id="modal2-id"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-jabatan-pangkat" class="form-label">Nama Barang</label>
                                    <input readonly class="form-control-plaintext" id="modal2-nm_barang"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-jabatan-pangkat" class="form-label">Keterangan</label>
                                    <input readonly class="form-control-plaintext" id="modal2-keterangan"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-jabatan-pangkat" class="form-label">Nomor SK</label>
                                    <input class="form-control" id="modal2-no_sk"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    @include('components.datepicker',[
                                        'value'=>'',
                                        'label'=>'Tanggal SK',
                                        'name'=>'modal2-tgl_sk'
                                        ]
                                    )
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
        $('#modal-kode_barang').select2({
            dropdownParent: $('#inputModal')
        });
        const tableId = '#pegawai-table';
        $(function() {
            const pegawaiTable = $(tableId).DataTable({
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ $controller . '/gridDataAsset/'.$model['id'] }}",
                    dataSrc: 'data',
                },
                info: false,
                ordering: false,
                paging: false,
                columns: [
                    {
                        data: 'kode_barang',
                        render: (data, type, row) => `${data}`
                    },
                    {
                        data: 'nm_barang',
                        render: (data, type, row) => `${data}`
                    },
                    {
                        data: 'keterangan',
                        render: (data, type, row) => (data?data:'')
                    },
                    {"data": function (row, data, index, display) {

                        let file = row.filenya?(row.filenya).split('|#|'):[];
                        let hasil = "";
                        let no_sk = row.no_sk?"Nomor SK : "+row.no_sk+"<br/>":'';
                        let tgl_sk = row.tgl_sk?"Tanggal SK : "+row.tgl_sk+"<br/>":'';
                        if(file.length>0){
                            $.each( file, function( i, val ) {
                                let a = val.split('---');
                                let url = `{{ url('${a[1]}') }}`;
                                hasil += '<a href="'+url+'" download terget="_blank"><i class="ri-download-cloud-line"></i> '+a[0]+'</a><br/>';
                            });
                        }
                        return no_sk+tgl_sk+hasil;
                    }},
                    {
                        data: 'id',
                        render: (data, type, row, meta) => {
                            const disabled = `{{ $aktifitas->canChange ? '' : 'disabled' }}`;
                            const aktifitas = `{{ $aktifitas->id }}`;
                            const editBtn = aktifitas==3004?`<button type="button" class="btn btn-primary btn-icon waves-effect waves-light ${disabled} editPegawai" data-id="${row.id}" data-kode_barang="${row.kode_barang}" data-nm_barang="${row.nm_barang}"  data-keterangan="${row.keterangan}"  data-no_sk="${row.no_sk}" data-tgl_sk="${row.tgl_sk}"><i class="ri-pencil-fill"></i></button>`:'';
                            return `
                                <div class="d-flex justify-content-center gap-1">
                                    ${editBtn}
                                    <button type="button" class="btn btn-danger btn-icon waves-effect waves-light deletePegawai ${disabled}" data-id="${row.id}" data-nama="${row.nm_barang}">
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
                $('#modal-kode_barang').val("").trigger('change');
                $('#modal-keterangan').val("");
                $('#modal-pengajuan_asset-id').val("");
            });

            $('#simpanPegawai').on('click', function() {
                let pengajuan_asset_id = $('#modal-pengajuan_asset_id').val();
                let kode_barang = $('#modal-kode_barang').val();
                let keterangan = $('#modal-keterangan').val();
                let pengajuan_id = $('#modal-pengajuan_id').val();
                var data = new FormData();
                data.append('pengajuan_asset_id', pengajuan_asset_id);
                data.append('kode_barang', kode_barang);
                data.append('keterangan', keterangan);
                data.append('pengajuan_id', pengajuan_id);

                $.ajax({
                    method: "POST",
                    url: `{{ $controller. '/saveAsset' }}`,
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
                        url: `{{ $controller. '/deleteAsset/' }}`+id,
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

            $('#editModal').on('hidden.bs.modal', function(){
                $('#modal2-id').val("");
                $('#modal2-kode_barang').val("");
                $('#modal2-nm_barang').val("");
                $('#modal2-keterangan').val("");
                $('#modal2-no_sk').val("");
                $('#modal2-tgl_sk').val("");
            });

            $('#simpanUserPegawai').on('click', function() {
                let id = $('#modal2-id').val();
                let sk_penetapan = $('#modal2-sk_penetapan').val();
                let no_sk = $('#modal2-no_sk').val();
                let tgl_sk = $('#modal2-tgl_sk').val();
                var data = new FormData();
                data.append('id', id);
                data.append('no_sk', no_sk);
                data.append('tgl_sk', tgl_sk);
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
