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
                    @csrf
                    @if (!$isNew)
                        <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                    @endif
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Satker </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['inst_nama'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Jenis Pengadaan</label>
                                <br/>
                                @foreach ($jenisPengadaanOptions as $jns)
                                    @if ($model['jenis_pengadaan'] == $jns['id'])
                                    <input type="text" class="form-control-plaintext" value="{{ $jns['text'] }}">
                                    @endif
                                @endforeach
                            </div>
                        </div>
                    </div>

                    <div class="row lebih">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Konsep HPS</label>
                                <br/>
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
                                <br/>
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
                                <input type="text" class="form-control-plaintext" value="{{ $model['no_spk'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Tanggal SPK</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['tgl_spk'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Jangka Waktu Pelaksanaan</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['jangka_waktu_pelaksanaan'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row all">
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Jenis Kontrak </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['jenis_kontrak'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="nip" class="form-label">No. Kontrak </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['no_kontrak'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Nilai</label>
                                <input type="text" class="form-control-plaintext angka" value="{{ $model['nilai_kontrak'] ?? '' }}">
                            </div>
                        </div>
                        <div class=" col-lg-3">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Tanggal Kontrak </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['tgl_kontrak'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row all" >
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="nip" class="form-label">File Kontrak</label>
                                <br/>
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
                                <br/>
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
                                <br/>
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
                                <br/>
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
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
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
                var val = '{{ $model["jenis_pengadaan"] }}';
                if(val==1){
                    $('.all').show();
                    $('.lebih').show();
                }else{
                    $('.lebih').hide();
                    $('.all').show();
                }
            }
        })
    </script>
@endsection
