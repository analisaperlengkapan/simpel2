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
                                <h5 class="card-title mb-0">Detail</h5>
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

            <div class="col-lg-12 mb-4">
                <div class="hstack gap-2 justify-content-left">
                    <a href="{{ url($controller) }}" class="btn btn-outline-primary">Kembali</a>
                </div>
            </div>
        </div>
    </form>



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
                    url: "{{'/bmn/penghapusan/penghapusansk/gridDataAsset/'.$model['id'] }}",
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
                ]
            });
        })
    </script>
@endsection
