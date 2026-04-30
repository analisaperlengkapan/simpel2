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
                                            <th class="ps-0" width="20%" scope="row">Nomor Surat Permohonan Surat Keputusan</th>
                                            <td width="5%">:</td>
                                            <td class="">
                                                {{ $permohonansk['no_surat_permohonan'] ?? '-' }}
                                            </td>
                                        </tr>
                                        <tr>
                                            <th class="ps-0" width="20%" scope="row">Tanggal Surat Permohonan Surat Keputusan</th>
                                            <td width="5%">:</td>
                                            <td class="">
                                                {{ $permohonansk['tgl_surat_permohonan'] ?? '-' }}
                                            </td>
                                        </tr>
                                        <tr>
                                            <th class="ps-0" width="20%" scope="row">Kategori</th>
                                            <td width="5%">:</td>
                                            <td class="">
                                            {{ $permohonansk['kategori'] ? $kategori[$permohonansk['kategori']].($permohonansk['kategori']==2?' ('.$jenis[$permohonansk['jenis']].')':'') : '-' }}
                                            </td>
                                        </tr>
                                        <tr>
                                            <th class="ps-0" width="20%" scope="row">File Permohonan Surat Keputusan</th>
                                            <td width="5%">:</td>
                                            <td class="">
                                                @if($permohonansk['file_surat_permohonan'])
                                                <a href="{{url($permohonansk['file_surat_permohonan'])}}" download terget="_blank"><i class="ri-download-cloud-line"></i> Download</a>
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
                                <h5 class="card-title mb-0">File Pendukung</h5>
                            </div>
                            <div class="flex-shrink-0">
                                <a href="#" type="button" data-bs-toggle="modal" data-bs-target="#inputModalLain"
                                    class="btn btn-success btn-label waves-effect waves-light"><i
                                        class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                    Tambah
                                </a>
                            </div>
                        </div>
                    </div>
                    <div class="card-body">
                        <table class="table align-middle  mb-0 my-dt" id="file-table">
                            <thead class="table-light">
                                <tr>
                                    <th scope="col">Nomor</th>
                                    <th scope="col">Tanggal</th>
                                    <th scope="col">Keterangan/Nama File</th>
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
            <div class="col-lg-12 mb-4">
                <div class="hstack gap-2 justify-content-left">
                    <a href="{{ url($controller) }}" class="btn btn-outline-primary">Kembali</a>
                    <button type="submit" class="btn btn-primary"> Simpan </button>
                </div>
            </div>
        </div>
    </form>

    <div id="inputModalLain" class="modal fade zoomIn" tabindex="-1" aria-labelledby="inputModalFotocopy" aria-hidden="true" style="display: none;">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title">File Pendukung Lainnya</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form id="inputForm">
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal2-nip" class="form-label">Nomor</label>
                                    <input type="text" class="form-control" id="lain-nomor"/>
                                    <input type="hidden" class="form-control" id="lain-id"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    @include('components.datepicker',[
                                        'value'=>'',
                                        'label'=>'Tanggal',
                                        'name'=>'lain-tanggal'
                                        ]
                                    )
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal2-nip" class="form-label">Keterangan/Nama File *</label>
                                    <input type="text" class="form-control" id="lain-ket"/>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal2-sk_penetapan" class="form-label">File</label>
                                    <input type="file" class="form-control" id="lain-file"/>
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-light" data-bs-dismiss="modal" onc>Tutup</button>
                    <button id="simpanLain" class="btn btn-primary">Simpan</button>
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
            const fileTable = $('#file-table').DataTable({
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ $controller . '/gridDataFilelain/'.$model['id'] }}",
                    dataSrc: 'data',
                },
                info: false,
                ordering: false,
                paging: false,
                columns: [
                    {
                        data: 'nomor',
                        render: (data, type, row) => `${data ?? ''}`
                    },
                    {
                        data: 'tanggal',
                        render :(data, type, row) =>{
                            return row.tanggal?dateFormatIndo(row.tanggal):'';
                        }
                    },
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
                            return `
                                <div class="d-flex justify-content-center gap-1">
                                    <button type="button" class="btn btn-danger btn-icon waves-effect waves-light delete" data-id="${row.id}">
                                        <i class="ri-delete-bin-5-fill"></i>
                                    </button>
                                </div>
                        `;
                        },
                    }
                ]
            });

            $('#file-table').on('click', '.delete', function() {
                const id = $(this).data('id');
                swal(`Yakin akan menghapus?`, {
                    icon: "info",
                    dangerMode: true,
                    buttons: true,
                }).then((res) => {
                    if (!res) return;
                    $.ajax({
                        method: "DELETE",
                        url: `{{ $controller. '/deleteFilelain/' }}`+id,
                        success: function(){
                            notify({
                                type: "success",
                                message: "Data Berhasil Dihapus",
                            });
                        },
                        error: showError,
                    }).done(function( msg ) {
                        fileTable.ajax.reload();
                    });
                });
            });

            $('#inputModalLain').on('hidden.bs.modal', function(){
                $('#lain-id').val("");
                $('#lain-nomor').val("");
                $('#lain-tanggal').val("");
                $('.input').val("");
                $('#lain-file').val("");
                $('#lain-ket').val("");
            });

            $('#simpanLain').on('click', function() {
                let id = $('#lain-id').val();
                let nomor = $('#lain-nomor').val();
                let tanggal = $('#lain-tanggal').val();
                let pengajuan_id = $('#id').val();
                let ket = $('#lain-ket').val();
                var data = new FormData();
                data.append('id', id);
                data.append('nomor', nomor);
                data.append('tanggal', tanggal);
                data.append('pengajuan_id', pengajuan_id);
                data.append('ket', ket);
                if($('#lain-file')[0].files.length>0){
                    var files = $('#lain-file')[0].files;
                    data.append('file', files[0]);
                }
                $.ajax({
                    method: "POST",
                    url: `{{ $controller. '/saveLain' }}`,
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
                    fileTable.ajax.reload();
                    $('#inputModalLain').modal('hide');
                });
            });
        })
    </script>
@endsection
