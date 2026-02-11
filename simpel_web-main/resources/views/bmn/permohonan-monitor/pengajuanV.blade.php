@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{$kategoriJudul}}</h5>
                        </div>
                        <!-- @if ($canCreate)
                            <div class="flex-shrink-0">
                                <a href="#" type="button" data-bs-toggle="modal" data-bs-target="#myModal"
                                    class="btn btn-success btn-label waves-effect waves-light"><i
                                        class="ri-add-line label-icon align-middle fs-16 me-2"></i>
                                    Tambah
                                </a>
                            </div>
                        @endif -->
                    </div>
                </div>
                <div class="card-body">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt"
                        style="width:100%">
                        <thead>
                            <tr>
                                <th>Nomor Permohonan @include('components.dtFilterInput',['index'=>0,'column' => 'Nama'])</th>
                                <th>Tanggal Permohonan @include('components.datepicker',['index'=>1,'name'=>'','className'=>'column-filter'])</th>
                                <th>Kategori @include('components.dtFilterInput',['index'=>2,'column'=>'kategori'])</th>
                                <th>Satker @include('components.dtFilterInput',['index' => 3,'column' => 'Satker'])</th>
                                <th>Aktifitas @include('components.dtFilterInput',['index' => 4,'column' => 'Aktifitas'])</th>
                                <th>File Persetujuan</th>
                                <th>Aksi</th>
                            </tr>
                        </thead>
                    </table>
                </div>
            </div>
        </div>
    </div>
    <div id="myModal" class="modal fade" tabindex="-1" data-bs-focus="false" aria-labelledby="myModalLabel" aria-hidden="true" style="display: none;">
        <div class="modal-dialog">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title" id="myModalLabel">Form Monitoring</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"> </button>
                </div>
                <div class="modal-body">
                <form action="{{ $controller }}" method="POST" class="ajaxForm" enctype="multipart/form-data">
                        @csrf
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Pengajuan Surat Keputusan Penghapusan BMN</label>
                                    <div class="input-group">
                                        <input type="text" class="form-control" id="no_pengajuan" name="no_pengajuan" value="{{ $model['no_pengajuan'] ?? '' }}" readonly>
                                        <input type="hidden" class="form-control" id="pengajuan_id" name="pengajuan_id" value="{{ $model['pengajuan_id'] ?? '' }}" readonly>
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
                        <div class="row mt-3">
                            <div class="col-lg-12">
                                <div class="hstack gap-2 justify-content-start">
                                    <button type="button" data-bs-dismiss="modal" class="btn btn-outline-primary">Kembali</a>
                                    <button type="submit" class="btn btn-primary"> Simpan </button>
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
            </div><!-- /.modal-content -->
        </div><!-- /.modal-dialog -->
    </div><!-- /.modal -->

    <div id="modal-pengajuan" class="modal fade" tabindex="-1" aria-labelledby="myModalLabel" aria-hidden="true" style="display: none;">
        <div class="modal-dialog modal-lg">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title" id="myModalLabel">Daftar Permohonan Penerbitan Persetujuan Penghapusan BMN</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"> </button>
                </div>
                <div class="modal-body">
                    <table class="table align-middle  mb-0 my-dt" id="permohonan-table" width="100%">
                        <thead class="table-light">
                            <tr>
                                <th scope="row">No Surat Permohonan</th>
                                <th scope="col">Tanggal Surat Permohonan</th>
                                <th scope="col">Kategori</th>
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
    <style>
        #{{ $tableId }} thead th {
            background-color: #405189;
            color: #ffffff;
            text-align: center;
            text-transform: uppercase;
        }
    </style>
@endsection

@section('js')
    <script>
        const tableId = `{{ $tableId }}`;
        const kategori = @json($kategori);
        const jenis = @json($jenis);
        $(function() {
            const dt = $('#' + tableId).DataTable({
                serverSide: true,
                processing: true,
                deferRender: true,
                ordering: false,
                dom: dtLayout,
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ $controller . '/gridData' }}",
                    dataSrc: 'data',
                },
                columns: [
                    {
                        data: 'no_surat_permohonan'
                    },
                    {
                        data: 'tgl_surat_permohonan',
                        render :(data, type, row) =>{
                            return dateFormatIndo(row.tgl_surat_permohonan);
                        }
                    },
                    {
                        data: 'kategori',
                        render :(data, type, row) =>{
                            return kategori[data]+(data==2?"<br/>("+jenis[row.jenis]+")":'');
                        }
                    },
                    {
                        data: 'inst_nama'
                    },
                    {
                        data: 'aktifitas'
                    },
                    {
                        data: 'pengajuan_id',
                        render: (data, type, row) => {
                            const url = `{{ '/bmn/penghapusan/penghapusansk/downloadZip/' . '${data}' }}`;
                            if(data){
                                return `<a href="${url}"><i class="ri-download-cloud-line"></i>Download</a>`;
                            }else{
                                return '';
                            }
                        }
                    },
                    {
                        data: 'id',
                        render: (data, type, row) => {
                            const url = `{{ $controller . '/' . '${data}' }}`;
                            return `
                                <div class="d-flex justify-content-center gap-2">
                                    @include('components.updateBtn', [
                                        'url' => '${url}',
                                    ])
                                </div>`;
                        },
                    }
                ]
            });

            const msKontrak = $('#permohonan-table').DataTable({
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{ $controller . '/gridDataPermohonan' }}",
                    dataSrc: 'data',
                },
                columns: [{
                        data: 'no_surat_permohonan',
                        render: (data, type, row, meta) => data
                    },
                    {
                        data: 'tgl_surat_permohonan',
                        render :(data, type, row) =>{
                            return dateFormatIndo(row.tgl_surat_permohonan);
                        }
                    },
                    {
                        data: 'kategori',
                        render: (data, type, row) => data
                    },
                    {
                        data: 'id',
                        render: (data, type, row, meta) => {
                            const disabled = "";
                            return `
                            <button type="button" class="btn btn-info btn-icon waves-effect waves-light pilih ${disabled}" data-no_surat_permohonan="${row.no_surat_permohonan}" data-id="${row.id}">
                                <i class=" ri-checkbox-line"></i>
                            </button>
                        `;
                        },
                    }
                ],
            });

            // $('#simpan').on('click', function(){
            //     let id = $('#id').val();
            //     let kategori = $('#kategori').val();
            //     let tgl_pengajuan = $('#tgl_pengajuan').val();
            //     let nama = $('#nama').val();

            //     let data = {
            //         id,kategori,tgl_pengajuan,nama
            //     };

            //     $.ajax({
            //         method: "POST",
            //         url: `{{ $controller }}`,
            //         data: data,
            //         success: function(){
            //             notify({
            //                 type: "success",
            //                 message: "Data Berhasil Disimpan",
            //             });
            //         },
            //         error: showError,
            //     }).done(function( msg ) {
            //         dt.ajax.reload();
            //         $('#myModal').modal('hide');
            //     });
            // });

            $('#myModal').on('hidden.bs.modal', function(){
                $('#id').val("");
                $('#tgl_pengajuan').val("");
                $('#tgl_pengajuan').flatpickr().clear();
            });

            $('#searchNip').on('click', function(){
                msKontrak.ajax.reload();
                $('#modal-pengajuan').modal('show');
            });

            $('#permohonan-table').on('click', '.pilih', function(){
                let no_surat_permohonan = $(this).data('no_surat_permohonan');
                let pengajuan_id = $(this).data('id');
                $('#no_pengajuan').val(no_surat_permohonan);
                $('#pengajuan_id').val(pengajuan_id);
                $('#modal-pengajuan').modal('hide');
            });
        })
    </script>
@endsection
