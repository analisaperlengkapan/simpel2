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
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Satker </label>
                                <select class="form-control selectTwo text-black" data-choices id="satker" name="satker">
                                        {!! $satkerOptions !!}
                                </select>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Jenis Pengadaan *</label>
                                <select class="form-control" data-choices data-choices-sorting-false name="jenis_pengadaan" id="jenis_pengadaan">
                                <option value="">Pilih</option>
                                @foreach ($jenisPengadaanOptions as $jns)
                                    <option {{ isset($model['jenis_pengadaan']) && $model['jenis_pengadaan'] == $jns['id'] ?'selected':'' }} value="{{ $jns['id'] }}">{{ $jns['text'] }}</option>
                                @endforeach
                                </select>
                            </div>
                        </div>
                    </div>

                    <div class="row lebih">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Konsep HPS</label>
                                <input type="file" class="form-control" id="konsep_hps" name="konsep_hps" value="{{ $model['konsep_hps'] ?? '' }}">
                                @if(isset($model['konsep_hps']))
                                <a href="{{ url($model['konsep_hps']) }}" download terget="_blank">
                                    <i class="ri-download-cloud-line"></i> Download File
                                </a>
                                @endif
                            </div>
                        </div>
                    </div>
                    <div class="row lebih">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Surat Keputusan Penyedia</label>
                                <input type="file" class="form-control" id="surat_keputusan_penyedia" name="surat_keputusan_penyedia" value="{{ $model['surat_keputusan_penyedia'] ?? '' }}">
                                @if(isset($model['surat_keputusan_penyedia']))
                                <a href="{{ url($model['surat_keputusan_penyedia']) }}" download terget="_blank">
                                    <i class="ri-download-cloud-line"></i> Download File
                                </a>
                                @endif
                            </div>
                        </div>
                    </div>
                    <div class="row lebih">
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Nomor SPK</label>
                                <input type="text" class="form-control" id="no_spk" name="no_spk" value="{{ $model['no_spk'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-4">
                            <div class="mb-3">
                                @include('components.datepicker',[
                                    'value'=>$model['tgl_spk']??'',
                                    'label'=>'Tanggal SPK',
                                    'name'=>'tgl_spk'
                                    ]
                                )
                            </div>
                        </div>
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Jangka Waktu Pelaksanaan</label>
                                <input type="text" class="form-control" id="jangka_waktu_pelaksanaan" name="jangka_waktu_pelaksanaan" value="{{ $model['jangka_waktu_pelaksanaan'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row all">
                        <!-- <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Jenis Kontrak *</label>
                                <select class="form-control" data-choices data-choices-sorting-false name="jenis_kontrak" id="jenis_kontrak">
                                    <option value="">Pilih Jenis</option>
                                    {!! $jenisOptions !!}
                                </select>
                            </div>
                        </div> -->
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="nip" class="form-label">No. Kontrak *</label>
                                <input type="hidden" id="isNew" name="isNew" value="{{ $isNew }}">
                                <input type="text" class="form-control" id="no_kontrak" name="no_kontrak" value="{{ $model['no_kontrak'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Nilai *</label>
                                <input type="text" class="form-control angka" id="nilai_kontrak" name="nilai_kontrak" value="{{ $model['nilai_kontrak'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-3">
                            <div class="mb-3">
                                @include('components.datepicker',[
                                    'value'=>$model['tgl_kontrak']??'',
                                    'label'=>'Tanggal Kontrak',
                                    'name'=>'tgl_kontrak'
                                    ]
                                )
                            </div>
                        </div>
                    </div>
                    <div class="row all" >
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="nip" class="form-label">File Kontrak *</label>
                                <input type="file" class="form-control" id="file_kontrak" name="file_kontrak" value="{{ $model['file_kontrak'] ?? '' }}">
                                @if(isset($model['file_kontrak']))
                                <a href="{{ url($model['file_kontrak']) }}" download terget="_blank">
                                    <i class="ri-download-cloud-line"></i> Download File
                                </a>
                                @endif
                            </div>
                        </div>
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Berita Acara Serah Terima</label>
                                <input type="file" class="form-control" id="bast" name="bast" value="{{ $model['bast'] ?? '' }}">
                                @if(isset($model['bast']))
                                <a href="{{ url($model['bast']) }}" download terget="_blank">
                                    <i class="ri-download-cloud-line"></i> Download File
                                </a>
                                @endif
                            </div>
                        </div>
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Berita Acara Pembayaran</label>
                                <input type="file" class="form-control" id="ba_pembayaran" name="ba_pembayaran" value="{{ $model['ba_pembayaran'] ?? '' }}">
                                @if(isset($model['ba_pembayaran']))
                                <a href="{{ url($model['ba_pembayaran']) }}" download terget="_blank">
                                    <i class="ri-download-cloud-line"></i> Download File
                                </a>
                                @endif
                            </div>
                        </div>
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Nodis Pengantar Kuitansi</label>
                                <input type="file" class="form-control" id="nodis_pengantar_kuitansi" name="nodis_pengantar_kuitansi" value="{{ $model['nodis_pengantar_kuitansi'] ?? '' }}">
                                @if(isset($model['nodis_pengantar_kuitansi']))
                                <a href="{{ url($model['nodis_pengantar_kuitansi']) }}" download terget="_blank">
                                    <i class="ri-download-cloud-line"></i> Download File
                                </a>
                                @endif
                            </div>
                        </div>
                    </div>
                    <hr/>
                    <div class="row">
                        <div class="col-lg-12">
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
        </div>
    </div>
</form>
@endsection

@section('js')
    <script>
        $('.lebih,.all').hide();
        $(function() {
            $('#jenis_pengadaan').on('change',function(){
                let val = $(this).val();
                if(val==1){
                    $('.all').show();
                    $('.lebih').show();
                }else{
                    $('.lebih').hide();
                    $('.all').show();
                }
            });
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
        })
    </script>
@endsection
