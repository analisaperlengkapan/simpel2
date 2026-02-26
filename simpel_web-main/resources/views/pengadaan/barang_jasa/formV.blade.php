@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
<form action="{{$controller}}" method="POST" class="ajaxForm" enctype="multipart/form-data">
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }} Pengadaan Barang dan Jasa</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    @csrf
                    @if (!$isNew)
                        <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                    @endif
                    <div class="row">
                        <div class="col-lg-8">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Nama Pengadaan *</label>
                                <input type="text" class="form-control" id="nama_pengadaan" name="nama_pengadaan" value="{{ $model['nama_pengadaan'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-4">
                            @if ($isNew)
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Jenis Pengadaan *</label>
                                    <select class="form-control" data-choices data-choices-sorting-false name="jenis_pengadaan" id="jenis_pengadaan">
                                    <option value="">Pilih</option>
                                    @foreach ($jenisPengadaanOptions as $jns)
                                        <option {{ isset($model['jenis_pengadaan']) && $model['jenis_pengadaan'] == $jns['id'] ?'selected':'' }} value="{{ $jns['id'] }}">{{ $jns['text'] }}</option>
                                    @endforeach
                                    </select>
                                </div>
                            @endif
                            @if (!$isNew)
                                <div class="mb-3">
                                    <input type="hidden" class="form-control" id="jenis_pengadaan" name="jenis_pengadaan" value="{{ $model['jenis_pengadaan'] ?? '' }}">

                                    <label for="nip" class="form-label">Nama Pengadaan *</label>
                                    <input type="text" class="form-control" readonly id="" name="" value="{{ $model['jenis_pengadaan'] == '1' ? 'Pengadaan Lebih Dari 200Jt' : 'Pengadaan s.d 200Jt' }}">
                                </div>
                            @endif
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Kode Barang *</label>
                                <input type="text" class="form-control" id="kode_barang" name="kode_barang" value="{{ $model['kode_barang'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Nilai Pengadaan *</label>
                                <input type="text" class="form-control angka" id="nilai" name="nilai" value="{{ $model['nilai'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Data Anggaran *</label>
                                <div class="input-group">
                                    <input type="text" readonly class="form-control" id="anggaran_text" name="anggaran_text" value="{{ $model['kode_anggaran'] ?? '' }}"/>
                                    <input type="hidden" readonly class="form-control" id="kode_anggaran"/>
                                    <button class="input-group-text btn-dark btn" id="searchAnggaran" type="button">
                                        <span class="">
                                            <i class="ri-search-line align-bottom me-1"></i>
                                            Cari
                                        </span>
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>

                </div>
            </div>
        </div>
    </div>

    @if (!$isNew)

        @if ($model['jenis_pengadaan'] == '2')
            @include('pengadaan.barang_jasa.duaratusV', ['model' => $model, 'hps' => $hps, 'skppbj' => $skppbj])
        @elseif ($model['jenis_pengadaan'] == '2')
            @include('pengadaan.barang_jasa.duaratuslebihV', ['model' => $model, 'hps' => $hps, 'skppbj' => $skppbj])
        @endif

    @endif

    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-body">
                    <div class="hstack gap-2">
                        <a href="{{ url($controller) }}" class="btn btn-outline-primary">Kembali</a>
                        @if(!$readOnly)
                        <button type="submit" class="btn btn-primary">
                            {{ $isNew ? 'Simpan' : 'Ubah' }}
                        </button>
                        @endif
                    </div>
                </div>
            </div>
        </div>
    </div>

</form>

<div id="modal-anggaran" class="modal fade" tabindex="-1" aria-labelledby="myModalLabel" aria-hidden="true" style="display: none;">
        <div class="modal-dialog modal-lg">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title" id="myModalLabel">Daftar Anggaran</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"> </button>
                </div>
                <div class="modal-body">
                    <table class="table align-middle  mb-0 my-dt" id="msanggaran-table" width="100%">
                        <thead class="table-light">
                            <tr>
                                <th scope="col">Kode Anggaran</th>
                                <th scope="col">Uraian </th>
                                <th scope="col">Total Anggaran </th>
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
    <script src="https://cdn.ckeditor.com/4.14.1/standard/ckeditor.js"></script>
@endsection

@section('js')
    <script>
        //$('.lebih,.all').hide();
        $(function() {
            // $('#jenis_pengadaan').on('change',function(){
            //     let val = $(this).val();
            //     if(val==1){
            //         $('.all').show();
            //         $('.lebih').show();
            //     }else{
            //         $('.lebih').hide();
            //         $('.all').show();
            //     }
            // });
            var isReadOnly = '{{ $readOnly }}';
            if(isReadOnly){
                $('input, select').prop('readonly', true);
            }

            $('.angka').autoNumeric('init', {
                aSep : '.',
                aDec: ',',
                mDec: '0'
            });
            var isNew = '{{ $isNew }}';
            if(!isNew){
                $('#jenis_pengadaan').trigger('change');
            }

            const anggaranTable = $('#msanggaran-table').DataTable({
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ $controller . '/gridDataAnggaran' }}",
                    type: 'POST',
                    dataSrc: 'data'
                },
                columns: [
                    {
                        data: 'kode_anggaran',
                    },
                    {
                        data: 'uraian_subkomponen',
                    },
                    {
                        data: 'total',
                    },
                    {
                        data: 'kode_anggaran',
                        render: (data, type, row, meta) => {
                            return `
                            <button data-kode="${data}" type="button" class="btn btn-info btn-icon waves-effect waves-light pilih">
                                <i class=" ri-checkbox-line"></i>
                            </button>
                        `;
                        },
                    }
                ],
                columnDefs: [
                    {
                        targets: 2,
                        className: 'dt-body-right',
                        render: $.fn.dataTable.render.number('.', '.', 0, '')
                    }
                ]
            });

            $('#searchAnggaran').on('click', function(){
                anggaranTable.ajax.reload();
                $('#modal-anggaran').modal('show');
            });

            $('#msanggaran-table').on('click','.pilih',function(){
                const kode = $(this).data('kode');
                $('#anggaran_text').val(kode);
                $('#modal-anggaran').modal('hide');
            });

        })
    </script>
@endsection
