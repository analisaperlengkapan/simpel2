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
                            <h5 class="card-title mb-0">Pengajuan Kebutuhan BMN</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <div class="row">
                        <div class="col-lg-12">
                            <div class="mb-3">
                                <label for="nama" class="form-label">Tahun</label>
                                @csrf
                                <input type="hidden" name="id_pengajuan" placeholder="tahun exp. 2023" value="{{ $model['id'] ?? '' }}">
                                <input type="hidden" class="form-control" id="pengajuan_kebutuhan_bmn_satker_id" name="pengajuan_kebutuhan_bmn_satker_id" value="{{$pengajuanSatker['id']}}"/>
                                <input type="number" class="form-control-plaintext" id="tahun" name="tahun" placeholder="tahun exp. 2023" value="{{ $model['tahun'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-12">
                            <div class="mb-3">
                                <label for="nama" class="form-label">Nama</label>
                                <input class="form-control-plaintext" id="nama" name="nama" placeholder="nama" value="{{ $model['nama'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-12">
                            <div class="mb-3">
                                <label for="nama" class="form-label">Deskripsi</label>
                                <input class="form-control-plaintext" id="deskripsi" name="deskripsi" placeholder="deskripsi" value="{{ $model['deskripsi'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row mt-3">
                        <div class=" col-lg-6">
                            <div class="mb-3">
                                <label for="pilihan_satker" class="form-label">Satker</label>
                                <input class="form-control-plaintext" id="satker" name="satker" placeholder="satker" value="{{ $satker ?? '' }}">
                                </select>
                            </div>
                        </div>
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
                        <h5 class="card-title mb-0">Daftar Barang</h5>
                    </div>
                </div>
            </div>
            <div class="card-body">
                <table class="table align-middle  mb-0 my-dt" id="tb-barang">
                    <thead class="table-light">
                        <tr>
                            <th scope="col">Nama Barang</th>
                            <th scope="col">Jumlah Barang</th>
                            <th scope="col">Alasan Pengadaan</th>
                            <th scope="col" width="20%">File Pendukung</th>
                            <th scope="col">Skor</th>
                            <th scope="col">Prioritas</th>
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
                        <h5 class="card-title mb-0">Daftar Aset Satker</h5>
                    </div>
                </div>
            </div>
            <div class="card-body">
                <table id="tb-aset" class="display table table-bordered dt-responsive my-dt"
                    style="width:100%">
                    <thead>
                        <tr>
                            <th>Kode Barang</th>
                            <th>Nama Barang</th>
                            <th>NUP</th>
                            <th>Kondisi</th>
                            <th>Merk</th>
                        </tr>
                    </thead>
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
    <div class="col-lg-12 mb-4">
        <div class="hstack gap-2 justify-content-left">
            <a href="{{ url($controller) }}" class="btn btn-outline-primary">Kembali</a>
        </div>
    </div>
</form>
@endsection

@section('js')
    <script>
        $(function() {
            $('.selectTwo').select2();
            $('#pilihan_satker').on('change', function() {
                const val = $(this).val();
                $('#div-pilihan-satker').toggleClass('visually-hidden', val == 'semua')
            })
            $('#pilihan_satker').trigger('change');

            const tb_barang = $('#tb-barang').DataTable({
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ $controller . '/gridDataBarang/'.$pengajuanSatker['id'] }}",
                    dataSrc: 'data',
                },
                info: false,
                ordering: false,
                paging: false,
                columns: [
                    {
                        data: 'nama',
                    },
                    {
                        data: 'jumlah',
                    },
                    {
                        data: 'alasan',
                    },
                    {"data": function (row, data, index, display) {
                        let file = row.file_pendukung;
                        let hasil = "";
                        if(file){
                            let url = `{{ url('${file}') }}`;
                            hasil = '<a href="'+url+'" download terget="_blank"><i class="ri-download-cloud-line"></i> File Pendukung</a>';
                        }
                        return hasil;
                    }},
                    {
                        data: 'skor',
                    },
                    {"data": function (row, data, index, display) {
					    return "<input style='text-align:right;' type='number' width='10%' class='form-control prioritas' data-id='"+row.id+"' value='"+(row.prioritas)+"'/>";
                    }},
                ]
            });

            const tb_aset = $('#tb-aset').DataTable({
                serverSide: true,
                processing: true,
                deferRender: true,
                ordering: false,
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ $controller . '/gridDataSatkerAset?id=' }}"+$('#pengajuan_kebutuhan_bmn_satker_id').val(),
                    dataSrc: 'data',
                },
                columns: [{
                        data: 'kode_barang'
                    },
                    {
                        data: 'nm_barang'
                    },
                    {
                        data: 'nup',
                    },
                    {
                        data: 'kondisi',
                    },
                    {
                        data: 'merk',
                    },
                ]
            })

            $('#tb-barang').on('click', '.deleteBarang', function() {
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
                        url: `{{ $controller. '/deleteBarang/' }}`+id,
                        success: function(){
                            notify({
                                type: "success",
                                message: "Data Berhasil Dihapus",
                            });
                        },
                        error: showError,
                    }).done(function( msg ) {
                        tb_barang.ajax.reload();
                    });
                });
            }).on('click', '.editBarang', function(){
                let id = $(this).data('id');
                let nama = $(this).data('nama');
                let jumlah = $(this).data('jumlah');
                let alasan = $(this).data('alasan');
                let skor = $(this).data('skor');
                let file_pendukung = $(this).data('file_pendukung');

                $('#modal-id').val(id);
                $('#modal-nama').val(nama);
                $('#modal-jumlah').val(jumlah);
                $('#modal-alasan').val(alasan);
                $('#modal-skor').val(skor);
                $('#inputModal').modal('show');
            }).on('focusout','.prioritas',function(){
                let id = $(this).data('id');
                let prioritas = $(this).val();
                if(prioritas){
                    var data = new FormData();
                    data.append('id', id);
                    data.append('prioritas', prioritas);
                    $.ajax({
                        method: "POST",
                        url: `{{ $controller. '/savePrioritas' }}`,
                        data: data,
                        processData: false,
                        contentType: false,
                        success: function(){
                        },
                        error: showError,
                    }).done(function( msg ) {
                        tb_barang.ajax.reload();
                    });
                }
            });

            $('#inputModal').on('hidden.bs.modal', function(){
                $('#modal-id').val("");
                $('#modal-nama').val("");
                $('#modal-jumlah').val("");
                $('#modal-alasan').val("");
                $('#modal-skor').val("");
                $('#modal-file_pendukung').val("");
            });

            $('#simpanBarang').on('click', function() {
                let id = $('#modal-id').val();
                let nama = $('#modal-nama').val();
                let jumlah = $('#modal-jumlah').val();
                let alasan = $('#modal-alasan').val();
                let skor = $('#modal-skor').val();
                let pengajuan_kebutuhan_bmn_satker_id = $('#modal-pengajuan_kebutuhan_bmn_satker_id').val();
                var data = new FormData();
                data.append('id', id);
                data.append('nama', nama);
                data.append('jumlah', jumlah);
                data.append('alasan', alasan);
                data.append('skor', skor);
                data.append('pengajuan_kebutuhan_bmn_satker_id', pengajuan_kebutuhan_bmn_satker_id);
                var files = $('#modal-file_pendukung')[0].files;
                data.append('file_pendukung', files[0]);

                $.ajax({
                    method: "POST",
                    url: `{{ $controller. '/saveBarang' }}`,
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
                    tb_barang.ajax.reload();
                    $('#inputModal').modal('hide');
                });
            });
        })
    </script>
@endsection
