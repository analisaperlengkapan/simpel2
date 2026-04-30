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
                                <input type="hidden" name="id_pengajuan" placeholder="tahun exp. 2023" value="{{ $model['id'] ?? '' }}">
                                <input type="hidden" class="form-control" id="pengajuan_kebutuhan_bmn_satker_id" name="pengajuan_kebutuhan_bmn_satker_id" value="{{$pengajuanSatker['id']}}"/>
                                <input type="number" class="form-control-plaintext" id="tahun" name="tahun" placeholder="tahun exp. 2023" value="{{ $model['tahun'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row" id="div-tanggal">
                            <div class=" col-lg-6">
                            <div class="mb-3">
                                <label for="nama" class="form-label">Tanggal Mulai</label>
                                <input class="form-control-plaintext" readonly id="nama" name="nama" placeholder="nama" value="{{ $model['tgl_mulai'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-6">
                            <div class="mb-3">
                                <label for="nama" class="form-label">Tanggal Selesai</label>
                                <input class="form-control-plaintext" readonly id="nama" name="nama" placeholder="nama" value="{{ $model['tgl_selesai'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-12">
                            <div class="mb-3">
                                <label for="nama" class="form-label">Nama Pengajuan</label>
                                <input class="form-control-plaintext" readonly id="nama" name="nama" placeholder="nama" value="{{ $model['nama'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-12">
                            <div class="mb-3">
                                <label for="nama" class="form-label">Deskripsi</label>
                                <input class="form-control-plaintext" readonly id="deskripsi" name="deskripsi" placeholder="deskripsi" value="{{ $model['deskripsi'] ?? '' }}">
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
                    url: "{{ '/analisis-kebutuhan/bmn/pengajuan/gridDataBarang/'.$pengajuanSatker['id'] }}",
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
        })
    </script>
@endsection
