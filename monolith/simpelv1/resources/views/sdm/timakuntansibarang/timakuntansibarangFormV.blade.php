@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
<form action="/sdm/timakuntansibarang" method="POST" class="ajaxForm" enctype="multipart/form-data">
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }}</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    @csrf
                    @if (!$isNew)
                        <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                    @endif
                    <div class="row">
                        {{--
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Kode Satker *</label>
                                <select class="form-control" data-choices data-choices-sorting-false name="kdsatker_keu" id="kdsatker_keu">
                                    <option value="">Pilih Satker</option>
                                    {!! $satkerOptions !!}
                                </select>
                            </div>
                        </div>
                        --}}

                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Satker </label>
                                <input type="text" class="form-control" id="satker" name="satker" value="{{ $model['inst_nama'] ?? '' }}" disabled>
                                <input type="hidden" class="form-control" id="kdsatker_keu" name="kdsatker_keu" value="{{ $model['kdsatker_keu'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-2">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Jenis SK *</label>
                                <select class="form-control" data-choices data-choices-sorting-false name="jenis_sk" id="jenis_sk">
                                    <option value="">Pilih Jenis</option>
                                    {!! $jenisOptions !!}
                                </select>
                            </div>
                        </div>
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Nomor SK *</label>
                                <input type="hidden" id="isNew" name="isNew" value="{{ $isNew }}">
                                <input type="text" class="form-control" id="no_surat" name="no_surat" value="{{ $model['no_surat'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-2">
                            <div class="mb-3">
                                <label for="name" class="form-label">Tanggal SK *</label>
                                <input type="date" class="form-control" id="tgl_surat" name="tgl_surat" value="{{ $model['tgl_surat'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Dikeluarkan Di</label>
                                <input class="form-control" id="dikeluarkan_di" name="dikeluarkan_di" value="{{ $model['dikeluarkan_di'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row" >
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="nip" class="form-label">File SK *</label>
                                <input type="file" class="form-control" id="file_sk" name="file_sk" value="{{ $model['file_sk'] ?? '' }}">
                                @if(isset($model['file_sk']))
                                <a href="{{ url($model['file_sk']) }}" download terget="_blank">
                                    <i class="ri-download-cloud-line"></i> Download File
                                </a>
                                @endif
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
                        <h5 class="card-title mb-0">Pegawai</h5>
                    </div>
                    @if ($canCreate)
                        <div class="flex-shrink-0">
                            <button type="button" data-bs-toggle="modal" data-bs-target="#inputModal"
                                class="btn btn-success btn-label waves-effect waves-light"><i
                                    class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                Tambah
                            </button>
                        </div>
                    @endif
                </div>
            </div>
            <div class="card-body jpn-index">
                <table class="table align-middle  mb-0 my-dt" id="pegawai-table">
                    <thead class="table-light">
                        <tr>
                            <th scope="col">No</th>
                            <th scope="col">Nama / NIP</th>
                            <th scope="col">Jabatan / Pangkat</th>
                            <th scope="col">Jabatan dalam Tim</th>
                            <th scope="row" class="text-center" width="15%">Aksi</th>
                        </tr>
                    </thead>
                    <tbody>
                        @foreach ($modelPegawai as $key => $peg)
                        <tr data-id="{{ $peg['nip'] }}">
                            <td class="text-center"><span class="frmnojpn" data-row-count="{{$key+1}}">{{$key+1}}</span><input type="hidden" name="jpnid[]" value="{{$peg['nip'].'#'.$peg['nama'].'#'.$peg['pangkat'].'#'.$peg['jabatan'].'#'.$peg['jabatan_tim']}}" /></td>
                            <td>{{$peg['nama']}}<br />{{$peg['nip']}}</td>
                            <td>{{$peg['jabatan']}}<br />{{$peg['pangkat']}}</td>
                            <td>{{$peg['jabatan_tim']}}</td>
                            <td class="text-center"><button type="button" class="btn btn-danger btn-icon waves-effect waves-light btn_hapusjpn"  @if (!$canCreate) disabled @endif data-id="{{$peg['nip']}}"><i class="ri-delete-bin-5-fill"></i></button></td>
                        </tr>
                        @endforeach
                    </tbody>
                </table>
            </div>
        </div>
    </div>
    <hr/>
    <div class="row">
        <div class="col-lg-12">
            <div class="hstack gap-2">
                <a href="{{ url('sdm/timakuntansibarang') }}" class="btn btn-outline-primary">Kembali</a>
                @if(!$readOnly)
                <button type="submit" class="btn btn-primary">
                    {{ $isNew ? 'Simpan' : 'Ubah' }}
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
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="modal-jabatan-pangkat" class="form-label">Jabatan dalam Tim</label>
                                    <input class="form-control" id="modal-jabatan_tim"/>
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-light" data-bs-dismiss="modal" onc>Tutup</button>
                    <button id="simpanPegawai" class="btn btn-primary">Simpan</button>
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
        $(function() {
            var isReadOnly = '{{ $readOnly }}';
            if(isReadOnly){
                $('input, select').prop('readonly', true);
            }
            const msPegawaiTable = $('#mspegawai-table').DataTable({
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ $controller . '/gridDataMsPegawai' }}",
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
                            return `
                            <button type="button" class="btn btn-info btn-icon waves-effect waves-light pilihPegawai " data-nip="${row.nip}" data-nama="${row.nama}" data-jabatan="${row.jabatan}" data-pangkat="${row.pangkat}">
                                <i class=" ri-checkbox-line"></i>
                            </button>
                        `;
                        },
                    }
                ]
            });

            $('#pegawai-table').on("click", ".btn_hapusjpn", function(){
                const idnya = $(this).data('id');
                console.log(idnya);
                $('#pegawai-table').find("tr[data-id='"+idnya+"']").remove();
            });

            $('#searchNip').on('click', function(){
                msPegawaiTable.ajax.reload();
                $('#modal-pegawai').modal('show');
            });

            $('#inputModal').on('hidden.bs.modal', function(){
                $('#modal-nip').val("");
                $('#modal-nama').val("");
                $('#modal-pangkat').val("");
                $('#modal-jabatan').val("");
                $('#modal-pengajuan_id').val("");
                $('#modal-jabatan_tim').val("");
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

            $('#simpanPegawai').on('click', function(){
                var tabel 	= $("#pegawai-table");
                var rwTbl	= tabel.find('tbody > tr:last');
                var rwNom	= parseInt(rwTbl.find("span.frmnojpn").data('rowCount'));
                var newId 	= (isNaN(rwNom))?1:parseInt(rwNom + 1);
                var nip	    = $('#modal-nip').val();
                var nama	= $('#modal-nama').val();
                var pangkat	= $('#modal-pangkat').val();
                var jabatan	= $('#modal-jabatan').val();
                var jabatan_tim	= $('#modal-jabatan_tim').val();
                var param	= nip+'#'+nama+'#'+pangkat+'#'+jabatan+'#'+jabatan_tim;
                var myvar = nip;
                const disabled = `{{ $canCreate ? '' : 'disabled' }}`;
                var html = '<tr data-id="'+myvar+'">'+
                        '<td class="text-center"><span class="frmnojpn" data-row-count="'+newId+'">'+newId+'</span><input type="hidden" name="jpnid[]" value="'+param+'" /></td>'+
                        '<td>'+nama+'<br />'+nip+'</td>'+
                        '<td>'+jabatan+'<br />'+pangkat+'</td>'+
                        '<td>'+jabatan_tim+'</td>'+
                        '<td class="text-center"><button type="button" class="btn btn-danger btn-icon waves-effect waves-light btn_hapusjpn" '+disabled+' data-id="'+nip+'"><i class="ri-delete-bin-5-fill"></i></button></td>'+
                    '</tr>';
                if(isNaN(rwNom)){
                    rwTbl.remove();
                    rwTbl = tabel.find('tbody');
                    rwTbl.append(html);
                } else{
                    rwTbl.after(html);
                }
                $('#inputModal').modal('hide');
            });
        })
    </script>
@endsection
