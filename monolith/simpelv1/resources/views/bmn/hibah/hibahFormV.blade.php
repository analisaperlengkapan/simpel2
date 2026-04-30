@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
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
                    <form action="/bmn/hibah/hibah" method="POST" class="ajaxForm" enctype="multipart/form-data">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <!--<div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Kode Satker *</label>
                                    <select class="form-control" data-choices data-choices-sorting-false name="kdsatker_keu" id="kdsatker_keu">
                                        <option value="">Pilih Satker</option>
                                        {!! $satkerOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>-->
                        <div class="row">
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Jenis Hibah *</label>
                                    <select class="form-control" data-choices data-choices-sorting-false name="jenis_hibah" id="jenis_hibah">
                                        <option value="">Pilih Jenis</option>
                                        {!! $jenisOptions !!}
                                    </select>
                                </div>
                            </div>
                            <!--<div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nomor Register *</label>
                                    <input type="hidden" id="isNew" name="isNew" value="{{ $isNew }}">
                                    <input type="text" class="form-control" id="no_register" name="no_register" value="{{ $model['no_register'] ?? '' }}">
                                </div>
                            </div>-->
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="kdsatker_keu" class="form-label">Bentuk Hibah *</label>
                                    <select class="form-control" data-choices data-choices-sorting-false name="kategori" id="kategori">
                                        <option value="">Pilih Bentuk</option>
                                        {!! $kategoriOptions !!}
                                    </select>
                                </div>
                            </div>
                            <div class=" col-lg-2">
                                <div class="mb-3">
                                    <!--<label for="name" class="form-label">Tanggal Register *</label>
                                    <input type="date" class="form-control" id="tgl_register" name="tgl_register" value="{{ $model['tgl_register'] ?? '' }}">-->
                                    @include('components.datepicker',[
                                        'value'=>$model['tgl_register'] ?? '',
                                        'label'=>'Tanggal ',
                                        'name'=>'tgl_register'
                                        ]
                                    )
                                </div>
                            </div>
                            <div class="col-lg-3">
                                    <div class="mb-3">
                                        <label for="nip" class="form-label">Nilai Hibah</label>
                                        <!--<input class="form-control" id="nilai" name="nilai" value="{{ $model['nilai'] ?? '' }}">-->
                                        <input type="text" class="form-control angka" id="nilai" name="nilai" value="{{ $model['nilai'] ?? '' }}">
                                    </div>
                            </div>

                        </div>
                        <div class="row" >
                        <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Hibah Dari</label>
                                    <input class="form-control" id="hibah_ke" name="hibah_ke" value="{{ $model['hibah_ke'] ?? '' }}">
                                </div>
                            </div>
                            <!-- <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">File Lampiran *</label>
                                    <input type="file" class="form-control" id="file_sk" name="file_sk" value="{{ $model['file_sk'] ?? '' }}">
                                    @if(isset($model['file_sk']))
                                    <a href="{{ url($model['file_sk']) }}" download terget="_blank">
                                        <i class="ri-download-cloud-line"></i> Download File
                                    </a>
                                    @endif
                                </div>
                            </div> -->
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="card card-default">
                                    @if(!$readOnly)
                                    <div class="card-header with-border">
                                        <div class="row">
                                            <div class="col-sm-12">
                                                <a class="btn btn-danger hapusTembusan" title="Hapus"><i class="ri-delete-bin-line"></i></a>
                                                <a class="btn btn-success" id="tambah-tembusan" title="Tambah Tembusan"><i class="fa fa-plus jarak-kanan"></i>Tambah File</a><br>
                                            </div>
                                        </div>
                                    </div>
                                    @endif
                                    <div class="card-body">
                                        <div class="table-responsive">
                                            <table id="table_tembusan" class="display table table-bordered dt-responsive my-dt">
                                                <thead>
                                                    <tr>
                                                        <th width="10%"></th>
                                                        <th>Nomor</th>
                                                        <th>Tanggal</th>
                                                        <th>File</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    @if(empty($modelSk))
                                                    <tr data-id="1">
                                                        <td class="text-center"><input type="checkbox" name="chk_del_tembusan[]" class="hRow" id="chk_del_tembusan1" value="1"></td>
                                                        <td><input type="text" name="no_sk[]" class="form-control" value=""/></td>
                                                        <td>
                                                            <input name="id_sk[]" type="hidden" value="">
                                                            @include('components.datepicker',[
                                                                'value'=> '',
                                                                'name'=>'tgl_sk[]'
                                                                ]
                                                            )
                                                        </td>
                                                        <td><input type="file" name="file_sk[]" class="form-control" /></td>
                                                    </tr>
                                                    @endif
                                                    @foreach($modelSk as $key => $value)
                                                    <tr data-id="{{ $key+1 }}">
                                                        <td class="text-center"><input type="checkbox" name="chk_del_tembusan[]" class="hRow" id="chk_del_tembusan{{ $key+1 }}" value="{{ $key+1 }}"></td>
                                                        <td><input type="text" name="no_sk[]" class="form-control" value="{{ $value['no'] }}"/></td>
                                                        <td>
                                                            <input name="id_sk[]" type="hidden" value="{{ $value['id'] }}">
                                                            @include('components.datepicker',[
                                                                'value'=>$value['tgl'] ?? '',
                                                                'name'=>'tgl_sk[]'
                                                                ]
                                                            )
                                                        </td>
                                                        <td>
                                                            <input type="file" name="file_sk[]" class="form-control" />
                                                            @if(isset($value['file']))
                                                            <a href="{{ url($value['file']) }}" download terget="_blank">
                                                                <i class="ri-download-cloud-line"></i> Download File
                                                            </a>
                                                            @endif
                                                        </td>
                                                    </tr>
                                                    @endforeach
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                </div>
                                <!-- <div class="mb-3">
                                    <label for="nip" class="form-label">File SK *</label>
                                    <input type="file" class="form-control filepond filepond-input-multiple" id="file_sk" name="file_sk" multiple value="{{ $model['file_sk'] ?? '' }}">
                                    @if(isset($model['file_sk']))
                                    <a href="{{ url($model['file_sk']) }}" download terget="_blank">
                                        <i class="ri-download-cloud-line"></i> Download File
                                    </a>
                                    @endif
                                </div> -->
                            </div>
                        </div>
                        <hr/>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2">
                                    <a href="{{ url('bmn/hibah/hibah') }}" class="btn btn-outline-primary">Kembali</a>
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
@endsection

@section('js')
    <script>
        $(function() {
            $('.angka').autoNumeric('init', {
                aSep : '.',
                aDec: ',',
                mDec: '0'
            });
            var isReadOnly = '{{ $readOnly }}';
            if(isReadOnly){
                $('input, select').prop('readonly', true);
            }
            $('#tambah-tembusan').click(function(){
                var tabel	= $('#table_tembusan > tbody').find('tr:last');
                var newId	= (tabel.length > 0)?parseInt(tabel.data('id'))+1:1;
                $('#table_tembusan').append(
                    '<tr data-id="'+newId+'">' +
                    '<td class="text-center"><input type="checkbox" name="chk_del_tembusan[]" class="hRow" id="chk_del_tembusan'+newId+'" value="'+newId+'"></td>'+
                    '<td><input type="text" name="no_sk[]" class="form-control" /></td>' +
                    `<td><div class="form-icon right">
                            <input name="id_sk[]" type="hidden">
                            <input data-provider="flatpickr" class="form-control" id="flatpickr${newId}" name="tgl_sk[]">
                            <i class="ri-calendar-2-fill"></i>
                        </div></td>`+
                    '<td><input type="file" name="file_sk[]" class="form-control" /></td>' +
                    '</tr>'
                );
                $('#table_tembusan').find(`#flatpickr${newId}`).flatpickr({
                    altFormat: "d-F-Y",
                    altInput: true,
                    parseDate:true
                });
            });
            $(".hapusTembusan").click(function(){
                var tabel 	= $("#table_tembusan");
                tabel.find(".hRow:checked").each(function(k, v){
                    var idnya = $(v).val();
                    tabel.find("tr[data-id='"+idnya+"']").remove();
                });
            });
        })
    </script>
@endsection
