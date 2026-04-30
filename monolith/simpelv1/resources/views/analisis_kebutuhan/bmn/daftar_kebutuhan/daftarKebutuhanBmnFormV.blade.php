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
                                <label for="nama" class="form-label">Tahun Anggaran</label>
                                @csrf
                                <input type="hidden"id="id_pengajuan" name="id_pengajuan" placeholder="tahun exp. 2023" value="{{ $model['id'] ?? '' }}">
                                <input type="hidden" class="form-control" name="pengajuan_kebutuhan_bmn_satker_id" id="pengajuan_kebutuhan_bmn_satker_id" value="{{$pengajuanSatker['id']}}"/>
                                <input type="number" class="form-control-plaintext" id="tahun" name="tahun" placeholder="tahun exp. 2023" value="{{ $model['tahun'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row" id="div-tanggal">
                            <div class=" col-lg-6">
                            <div class="mb-3">
                                <label for="nama" class="form-label">Tanggal Mulai</label>
                                <input class="form-control-plaintext" id="nama" name="nama" placeholder="nama" value="{{ $model['tgl_mulai'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-6">
                            <div class="mb-3">
                                <label for="nama" class="form-label">Tanggal Selesai</label>
                                <input class="form-control-plaintext" id="nama" name="nama" placeholder="nama" value="{{ $model['tgl_selesai'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-12">
                            <div class="mb-3">
                                <label for="nama" class="form-label">Nama Pengajuan</label>
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
    <div class="row">
        <div class="col-lg-12">
            <div class="card ">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Pengajuan Aset</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body p-4">
                    <div class="table-responsive">
                        <table id="table_tembusan" class="table table-bordered">
                            <thead class="table-light">
                                <tr>
                                    <th width="10%">#</th>
                                    <th width="40%">Kode Barang</th>
                                    <th width="40%">Keterangan</th>
                                </tr>
                            </thead>
                            <tbody>
                            @foreach($asset as $index => $data)
                            <tr data-id="{{$index}}">
                                <td class="text-center">{{$index+1}}</td>
                                <td>
                                    <select id="pilih_barang_{{$index}}" name="asset_kode_barang[]" class="form-control selectTwo" disabled>
                                    @foreach($listBarang as $item)
                                        <option {{ $item->kode_barang==$data['kode_barang']?'selected':'' }} value="{{ $item->kode_barang }}">{{ $item->kode_barang.' - '.$item->nama_barang }}</option>
                                    @endforeach
                                    </select>
                                </td>
                                <td><textarea disabled name="asset_keterangan[]" class="form-control" style="height:50px;">{{$data['keterangan']}}</textarea></td>
                            </tr>
                            @endforeach
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
                        <h5 class="card-title mb-0">Daftar Barang</h5>
                    </div>
                    @if ($operasi == 'INPUT' && ($aktifitas->canChange))
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
                <table class="table align-middle  mb-0 my-dt" id="tb-barang">
                    <thead class="table-light">
                        <tr>
                            <th scope="col">Kode Barang</th>
                            <th scope="col">Nama Barang</th>
                            <th scope="col">Jumlah di Satker</th>
                            <th scope="col">Jumlah Pengadaan</th>
                            <th scope="col">Alasan Pengadaan</th>
                            <th scope="col" width="20%">File Pendukung</th>
                            <th scope="col">Jumlah Disetujui</th>
                            <th scope="col">Jumlah Ditolak</th>
                            <th scope="col">Prioritas</th>
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
                        <h5 class="card-title mb-0">Daftar Aset Satker</h5>
                    </div>
                </div>
            </div>
            <div class="card-body">
                <table id="tb-aset" class="display table table-bordered dt-responsive my-dt"
                    style="width:100%">
                    <thead class="table-light">
                        <tr>
                            <th>Kode Barang</th>
                            <th>Nama Barang</th>
                            <th>NUP</th>
                            <th>Tanggal Perolehan</th>
                            <th>Nilai Perolehan</th>
                            <th>Kondisi</th>
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
            @if ($operasi == 'CREATE')
                <button type="button" class="btn btn-warning" id="btn-selesai">
                    Selesaikan Pengajuan
                </button>
            @endif
        </div>
    </div>
</form>

<div id="inputModal" class="modal fade zoomIn" tabindex="-1" aria-labelledby="inputModal" aria-hidden="true"
    style="display: none;">
    <div class="modal-dialog modal-dialog-centered">
        <div class="modal-content">
            <div class="modal-header">
                <h5 class="modal-title">Form Data Barang</h5>
                <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
            </div>
            <div class="modal-body">
                <form id="inputForm">
                    <div class="row">
                        <div class="col-lg-12">
                            <div class="mb-3">
                                <label for="modal-nama-nip" class="form-label">Nama Barang</label>
                                <select class="form-control" name="modal-nama" id="modal-nama">
                                    @foreach($kdBarangOptions as $data)
                                    <option value="{{ $data->kode_barang.'#'.$data->nama_barang }}">{{ $data->kode_barang.'-'.$data->nama_barang }}</option>
                                    @endforeach
                                </select>
                                <input type="hidden" class="form-control" id="modal-pengajuan_kebutuhan_bmn_satker_id" value="{{$pengajuanSatker['id']}}"/>
                                <input type="hidden" class="form-control" id="modal-id"/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-12">
                            <div class="mb-3">
                                <label for="modal-jabatan-pangkat" class="form-label">Jumlah Barang</label>
                                <input type="number" class="form-control" id="modal-jumlah" @if($operasi == 'CREATE') readonly @endif/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-12">
                            <div class="mb-3">
                                <label for="modal-jabatan-pangkat" class="form-label">Alasan Pengadaan</label>
                                <input class="form-control" id="modal-alasan" @if($operasi == 'CREATE') readonly @endif/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-12">
                            <div class="mb-3">
                                <label class="form-label">File Pendukung</label>
                                <input type="file" class="form-control" id="modal-file_pendukung" @if($operasi == 'CREATE') disabled @endif/>
                            </div>
                        </div>
                    </div>
                    <div class="row" @if($operasi != 'CREATE') style="display: none;" @endif>
                        <div class="col-lg-12">
                            <div class="mb-3">
                                <label for="modal-jabatan-pangkat" class="form-label">Jumlah Barang Disetujui</label>
                                <input type="number" min="0" class="form-control" id="modal-jml_setuju" @if($operasi != 'CREATE') readonly @endif/>
                            </div>
                        </div>
                    </div>
                    <div class="row" @if($operasi != 'CREATE') style="display: none;" @endif>
                        <div class="col-lg-12">
                            <div class="mb-3">
                                <label for="modal-jabatan-pangkat" class="form-label">Keterangan Approval Barang</label>
                                <textarea class="form-control" id="modal-keterangan" @if($operasi != 'CREATE') readonly @endif></textarea>
                            </div>
                        </div>
                    </div>
                </form>
            </div>
            <div class="modal-footer">
                <button type="button" class="btn btn-light" data-bs-dismiss="modal">Tutup</button>
                <button id="simpanBarang" class="btn btn-primary">Simpan</button>
            </div>
        </div>
    </div>
</div>
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
                        data: 'kode_barang',
                    },
                    {
                        data: 'nama',
                    },
                    {
                        data: 'jumlah_exist',
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
                        data: 'jml_setuju',
                    },
                    {"data": function (row, data, index, display) {
                        let jml = row.jumlah;
                        let jml_setuju = row.jml_setuju;
                        let jml_tolak = jml_setuju?jml-jml_setuju:'';
                        return jml_tolak;
                    }},
                    {
                        data: 'prioritas',
                    },
                    {
                        data: 'id',
                        render: (data, type, row, meta) => {
                            const disabled = `{{ ($aktifitas->canChange) ? '' : 'disabled' }}`;
                            const aktifitas = `{{ $aktifitas->id }}`;
                            const editBtn = `<button type="button" class="btn btn-primary btn-icon waves-effect waves-light ${disabled} editBarang" data-id="${row.id}" data-kode="${row.kode_barang}" data-keterangan="${row.keterangan}" data-nama="${row.nama}" data-jumlah="${row.jumlah}" data-alasan="${row.alasan}" data-skor="${row.skor}"  data-file_pendukung="${row.file_pendukung}"><i class="ri-pencil-fill"></i></button>`;
                            return `
                                <div class="d-flex justify-content-center gap-1">
                                    ${editBtn}
                                    <button type="button" class="btn btn-danger btn-icon waves-effect waves-light deleteBarang ${disabled}" data-id="${row.id}" data-nama="${row.nama}">
                                        <i class="ri-delete-bin-5-fill"></i>
                                    </button>
                                </div>
                        `;
                        },
                    }
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
                    url: "{{ '/analisis-kebutuhan/bmn/penyusunan-prioritas/gridDataSatkerAset?id=' }}"+$('#pengajuan_kebutuhan_bmn_satker_id').val(),
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
                        data: 'tgl_perolehan',
                        render :(data, type, row) =>{
                            return dateFormatIndo(row.tgl_perolehan);
                        }
                    },
                    {
                        data: 'nilai_perolehan',
                    },
                    {
                        data: 'kondisi',
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
                let kode = $(this).data('kode');
                let jumlah = $(this).data('jumlah');
                let alasan = $(this).data('alasan');
                let keterangan = $(this).data('keterangan');
                let file_pendukung = $(this).data('file_pendukung');

                $('#modal-id').val(id);
                $('#modal-nama').val(kode+'#'+nama);
                $('#modal-jumlah').val(jumlah);
                $('#modal-alasan').val(alasan);
                $('#modal-keterangan').val(keterangan);
                $('#inputModal').modal('show');
            });

            $('#inputModal').on('hidden.bs.modal', function(){
                $('#modal-id').val("");
                $('#modal-nama').val("");
                $('#modal-jumlah').val("");
                $('#modal-alasan').val("");
                $('#modal-jml_setuju').val("");
                $('#modal-keterangan').val("");
                $('#modal-file_pendukung').val("");
            });

            $('#simpanBarang').on('click', function() {
                let id = $('#modal-id').val();
                let namaArr = $('#modal-nama').val().split("#");
                let kode = namaArr[0];
                let nama = namaArr[1];
                let jumlah = $('#modal-jumlah').val();
                let alasan = $('#modal-alasan').val();
                let jml_setuju = $('#modal-jml_setuju').val();
                let keterangan = $('#modal-keterangan').val();
                let pengajuan_kebutuhan_bmn_satker_id = $('#modal-pengajuan_kebutuhan_bmn_satker_id').val();
                var data = new FormData();
                if(parseInt(jml_setuju)>parseInt(jumlah)){
                    notify({
                            type: "warning",
                            message: "Jumlah disetujui lebih besar dari jumlah barang",
                        });
                        return;
                }
                data.append('id', id);
                data.append('kode', kode);
                data.append('nama', nama);
                data.append('jumlah', jumlah);
                data.append('alasan', alasan);
                data.append('jml_setuju', jml_setuju);
                data.append('keterangan', keterangan);
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

            $('#btn-selesai').on('click', function(){
                const ms_aktifitas_id = 3006;
                const id_pengajuan = $('#id_pengajuan').val();
                const pengajuan_kebutuhan_bmn_satker_id = $('#pengajuan_kebutuhan_bmn_satker_id').val();
                const type = 'selesai';
                swal(`Yakin akan selesaikan pengajuan?`, {
                    icon: "info",
                    dangerMode: true,
                    buttons: true,
                }).then((res) => {
                    if (!res) return;
                    $.ajax({
                        method: "POST",
                        data:{ms_aktifitas_id,id_pengajuan,pengajuan_kebutuhan_bmn_satker_id,type},
                        url: `{{ $controller . '/savePengajuan' }}`,
                        success: function(){
                            window.location.href = `{{ $controller }}`;
                        },
                        error: showError,
                    });
                });
            });
        })
    </script>
@endsection
