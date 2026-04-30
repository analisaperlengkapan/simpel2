@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $judul }} Data Pengadaan</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/pengadaan/distribusi/pengisian" method="POST" class="ajaxForm" enctype="multipart/form-data">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nomor Kontrak *</label>
                                    <input type="hidden" id="isNew" name="isNew" value="{{ $isNew }}">
                                    <div class="input-group">
                                        <input type="text" class="form-control" id="no_kontrak" name="no_kontrak" value="{{ $model['no_kontrak'] ?? '' }}" readonly>
                                        <input type="hidden" class="form-control" id="id_kontrak" name="id_kontrak" value="{{ $model['id_kontrak'] ?? '' }}" readonly>
                                        <button class="input-group-text btn-dark btn" id="searchNip" type="button">
                                            <span class="">
                                                <i class="ri-search-line align-bottom me-1"></i>
                                                Cari
                                            </span>
                                        </button>
                                    </div>
                                </div>
                            </div>
                            <div class=" col-lg-4">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Tanggal Kontrak *</label>
                                    <input type="text" class="form-control" id="tgl_kontrak" name="tgl_kontrak" value="{{ $model['tgl_kontrak'] ?? '' }}" readonly>
                                </div>
                            </div>
                            <div class=" col-lg-4">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nilai Kontrak *</label>
                                    <input type="text" class="form-control" id="nilai_kontrak" name="nilai_kontrak" value="{{ $model['nilai_kontrak'] ?? '' }}" readonly>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nama Barang *</label>
                                    <input class="form-control" id="nm_barang" name="nm_barang" value="{{ $model['nm_barang'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-4">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Jumlah Barang *</label>
                                    <input type="text" class="form-control" id="jml_barang" name="jml_barang" value="{{ $model['jml_barang'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-4">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Nilai Barang *</label>
                                    <input type="number" class="form-control" id="nilai_barang" name="nilai_barang" value="{{ $model['nilai_barang'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-5">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Satker Tujuan *</label>
                                    <select class="form-control" data-choices data-choices-sorting-false name="kdsatker_tujuan" id="kdsatker_tujuan">
                                        <option value="">Pilih Satker</option>
                                        {!! $satkerOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row" >
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">File SPK *</label>
                                    <input type="file" class="form-control" id="file_spk" name="file_spk" value="{{ $model['file_spk'] ?? '' }}">
                                    @if(isset($model['file_spk']))
                                    <a href="{{ url($model['file_spk']) }}" download terget="_blank">
                                        <i class="ri-download-cloud-line"></i> Download File
                                    </a>
                                    @endif
                                </div>
                            </div>
                            <!-- @if(!request()->is('pengadaan/distribusi/konfirmasi-penerimaan'))
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">File BAST *</label>
                                    <input type="file" class="form-control" id="file_bast" name="file_bast" value="{{ $model['file_bast'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    <label for="name" class="form-label">File Foto *</label>
                                    <input type="file" class="form-control" id="file_foto" name="file_foto" value="{{ $model['file_foto'] ?? '' }}">
                                </div>
                            </div>
                            @endif -->
                        </div>
                        <div class="row">
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Apakah barang masuk gudang terlebih dahulu?</label>
                                    <select class="form-control" data-choices data-choices-sorting-false name="is_gudang" id="is_gudang">
                                        <option value="">Pilih</option>
                                        <option {{ $model && $model['is_gudang']==1? 'selected':'' }} value="1">Ya</option>
                                        <option {{ $model && $model['is_gudang']==2? 'selected':'' }} value="2">Tidak</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                        <hr/>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2">
                                    <a href="{{ url('pengadaan/distribusi/pengisian') }}" class="btn btn-outline-primary">Kembali</a>
                                    @if(!$readOnly)
                                    <button type="submit" class="btn btn-primary">
                                        {{ $isNew ? 'Simpan' : 'Ubah' }}
                                    </button>
                                    @endif
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>

    <div id="modal-kontrak" class="modal fade" tabindex="-1" aria-labelledby="myModalLabel" aria-hidden="true" style="display: none;">
        <div class="modal-dialog modal-lg">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title" id="myModalLabel">Daftar Kontrak</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"> </button>
                </div>
                <div class="modal-body">
                    <table class="table align-middle  mb-0 my-dt" id="kontrak-table" width="100%">
                        <thead class="table-light">
                            <tr>
                                <th scope="row">No Kontrak</th>
                                <th scope="col">Tanggal Kontrak</th>
                                <th scope="col">Nilai Kontrak</th>
                                <th scope="col">Uraian Kontrak</th>
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

            const msKontrak = $('#kontrak-table').DataTable({
                language: {
                    url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
                },
                ajax: {
                    url: "{{'/pengadaan/distribusi/gridDataKontrak'}}",
                    dataSrc: 'data',
                },
                columns: [{
                        data: 'no_kontrak',
                        render: (data, type, row, meta) => data
                    },
                    {
                        data: 'tanggal_kontrak',
                        render: (data, type, row) => data
                    },
                    {
                        data: 'nilai_kontrak',
                        render: (data, type, row) => data
                    },
                    {
                        data: 'uraian_kontrak',
                        render: (data, type, row) => data
                    },
                    {
                        data: 'id',
                        render: (data, type, row, meta) => {
                            const disabled = "";
                            return `
                            <button type="button" class="btn btn-info btn-icon waves-effect waves-light pilih ${disabled}" data-no_kontrak="${row.no_kontrak}" data-tgl_kontrak="${row.tanggal_kontrak}" data-nilai_kontrak="${row.nilai_kontrak}" data-id="${row.id_kontrak}">
                                <i class=" ri-checkbox-line"></i>
                            </button>
                        `;
                        },
                    }
                ],
                columnDefs:
                [
                    {
                        targets: 2,
                        render: $.fn.dataTable.render.number('.', '', 0, '')
                    },
                ],
            });

            $('#kontrak-table').on('click', '.pilih', function(){
                let no_kontrak = $(this).data('no_kontrak');
                let tgl_kontrak = $(this).data('tgl_kontrak');
                let nilai_kontrak = $(this).data('nilai_kontrak');
                let id = $(this).data('id');
                $('#no_kontrak').val(no_kontrak);
                $('#tgl_kontrak').val(tgl_kontrak);
                $('#nilai_kontrak').val(nilai_kontrak);
                $('#id_kontrak').val(id);
                $('#modal-kontrak').modal('hide');
            });

            $('#searchNip').on('click', function(){
                msKontrak.ajax.reload();
                $('#modal-kontrak').modal('show');
            });
        })
    </script>
@endsection
