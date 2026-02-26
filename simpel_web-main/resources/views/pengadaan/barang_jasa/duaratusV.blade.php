{{-- Part Konsep HPS --}}
<div class="row" style="margin-bottom: 5px;">
    <div class="col-lg-12 mb-12">
        <div class="accordion accordion-flush" id="accordionHPS">

            <div class="accordion-item">
                <h2 class="accordion-header" id="flush-headingOne">
                    <button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target="#flush-hps" aria-expanded="false" aria-controls="flush-collapseOne">
                        1. Konsep HPS
                    </button>
                </h2>
                <div id="flush-hps" class="accordion-collapse collapse" aria-labelledby="flush-headingOne" data-bs-parent="#accordionHPS">
                    <div class="accordion-body">
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nomor HPS</label>
                                    <input type="text" class="form-control" id="no_hps" name="no_hps" value="{{ $hps['no_hps'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    @include('components.datepicker',[
                                        'value'=>$hps['tgl_hps']??'',
                                        'label'=>'Tanggal HPS',
                                        'name'=>'tgl_hps'
                                        ]
                                    )
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">NIP Penandatangan</label>
                                    <input type="text" class="form-control" id="nip_penandatangan_hps" name="nip_penandatangan_hps" value="{{ $hps['nip_penandatangan'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nama Penandatangan</label>
                                    <input type="text" class="form-control" id="nama_penandatangan_hps" name="nama_penandatangan_hps" value="{{ $hps['nama_penandatangan'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Pangkat Penandatangan</label>
                                    <input type="text" class="form-control" id="pangkat_penandatangan_hps" name="pangkat_penandatangan_hps" value="{{ $hps['pangkat_penandatangan'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <hr/>
                                <a class="btn btn-danger btn-sm hapusHps" title="Hapus"><i class="ri-delete-bin-5-fill" style="font-size:14px;"></i></a>
                                <a class="btn btn-success btn-sm" id="tambah-hps" title="Tambah Barang"><i class="ri-add-line" style="font-size:14px;"></i></a>
                                <br/>
                                <br/>
                                <table id="table_hps" class="table table-bordered">
                                    <thead class="table-light">
                                        <tr>
                                            <th rowspan="2" class="text-center" width="10%">#</th>
                                            <th rowspan="2" class="text-center" width="40%">Uraian Barang</th>
                                            <th colspan="2" class="text-center" width="20%">Volume</th>
                                            <th rowspan="2" class="text-center" width="15%">Harga Satuan <br/> (Rp)</th>
                                            <th rowspan="2" class="text-center" width="15%">Jumlah Harga <br/> (Rp)</th>
                                        </tr>
                                        <tr>
                                            <th class="text-center">Qty</th>
                                            <th class="text-center">Satuan</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                    @php
                                        $decodedData = json_decode($hps['barang'], true);
                                    @endphp
                                    @if(!empty($decodedData))
                                    @foreach($decodedData as $key => $value)
                                    <tr data-id="{{$key}}">
                                        <td class="text-center"><input type="checkbox" name="chk_del_tembusan[]" class="hRow form-check-input" id="chk_del_tembusan{{$key}}" value="{{$key}}"></td>
                                        <td><input type="text" class="form-control" id="uraian_barang_{{$key}}" name="uraian_barang[]" value="{{$value['uraian_barang']}}"></td>
                                        <td><input type="text" class="form-control angka qty" id="qty_barang_{{$key}}" name="qty_barang[]" value="{{$value['qty_barang']}}"></td>
                                        <td><input type="text" class="form-control" id="satuan_barang_{{$key}}" name="satuan_barang[]" value="{{$value['satuan_barang']}}"></td>
                                        <td><input type="text" class="form-control angka harga_satuan" id="harga_satuan_{{$key}}" name="harga_satuan[]" value="{{$value['harga_satuan']}}"></td>
                                        <td><input type="text" class="form-control angka harga_total" readonly id="harga_total_{{$key}}" name="harga_total[]" value="{{$value['harga_total']}}"></td>
                                    </tr>
                                    @endforeach
                                    @endif
                                    </tbody>
                                </table>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                            <hr/>
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Keterangan</label>
                                    <textarea name="keterangan_hps" class="form-control ckeditor" id="keterangan_hps" cols="30" rows="10">{{$hps['keterangan'] ?? ''}}</textarea>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="hstack gap-2">
                                 @if(!$readOnly)
                                <button type="button" class="btn btn-primary" id="simpan-hps">Simpan</button>
                                @endif
                                <a href="{{url($controller.'/cetakHps/'.$model['id'])}}" type="button" class="btn btn-warning" target="_blank">Cetak</a>
                            </div>
                        </div>

                    </div>

                </div>
            </div>

        </div>
    </div>
</div>

{{-- Part SKPPBJ --}}
<div class="row" style="margin-bottom: 5px;">
    <div class="col-lg-12 mb-12">
        <div class="accordion accordion-flush" id="accordionSKPPBJ">

            <div class="accordion-item">
                <h2 class="accordion-header" id="flush-headingOne">
                    <button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target="#flush-skppbj" aria-expanded="false" aria-controls="flush-collapseOne">
                        2. Surat Keputusan Penetapan Penyedia Barang/Jasa (SKPPBJ)
                    </button>
                </h2>
                <div id="flush-skppbj" class="accordion-collapse collapse" aria-labelledby="flush-headingOne" data-bs-parent="#accordionSKPPBJ">
                    <div class="accordion-body">
                        <div class="row">
                            <div class="col-lg-12">
                                <h5>Penandatangan SKPPBJ</h5>
                                <hr/>
                            </div>
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nama </label>
                                    <input type="text" class="form-control" id="nama_penandatangan_skppbj" name="nama_penandatangan_skppbj" value="{{ $skppbj['nama_penandatangan'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">NIP </label>
                                    <input type="text" class="form-control" id="nip_penandatangan_skppbj" name="nip_penandatangan_skppbj" value="{{ $skppbj['nip_penandatangan'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Pangkat </label>
                                    <input type="text" class="form-control" id="pangkat_penandatangan_skppbj" name="pangkat_penandatangan_skppbj" value="{{ $skppbj['pangkat_penandatangan'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Jabatan </label>
                                    <input type="text" class="form-control" id="jabatan_penandatangan_skppbj" name="jabatan_penandatangan_skppbj" value="{{ $skppbj['jabatan_penandatangan'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Alamat </label>
                                    <input type="text" class="form-control" id="alamat_skppbj" name="alamat_skppbj" value="{{ $skppbj['alamat'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    @include('components.datepicker',[
                                        'value'=>$skppbj['tgl_skppbj']??'',
                                        'label'=>'Tanggal SKPPBJ',
                                        'name'=>'tgl_skppbj'
                                        ]
                                    )
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">

                                <h5>List Penawaran Penyedia</h5>
                                <hr/>
                                <table id="table_skppbj" class="table table-bordered">
                                    <thead class="table-light">
                                        <tr>
                                            <th class="text-center" width="10%">#</th>
                                            <th class="text-center" width="60%">Nama Penyedia</th>
                                            <th class="text-center" width="20%">Harga Penawaran <br/> (Rp)</th>
                                            <th class="text-center" width="10%">Pemenang</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        @php
                                            $decodedSkppbj = json_decode($skppbj['penyedia'], true);
                                        @endphp
                                        <tr>
                                            <td class="text-center">1</td>
                                            <td class="">
                                                <input type="text" class="form-control" name="penyedia_skppbj[]" value="{{ isset($decodedSkppbj[0]['penyedia_skppbj']) ? $decodedSkppbj[0]['penyedia_skppbj'] : '' }}" />
                                            </td>
                                            <td class="">
                                                <input type="text" class="form-control angka" name="nilai_skppbj[]" value="{{ isset($decodedSkppbj[0]['nilai_skppbj']) ? $decodedSkppbj[0]['nilai_skppbj'] : '' }}"/>
                                            </td>
                                            <td class="">
                                                <input type="radio" class="form-check-input" name="pemenang_skppbj[]" @if(isset($decodedSkppbj[0]['pemenang_skppbj']) && $decodedSkppbj[0]['pemenang_skppbj'] == 1) checked @endif/>
                                            </td>
                                        </tr>
                                        <tr>
                                            <td class="text-center">2</td>
                                            <td class="">
                                                <input type="text" class="form-control" name="penyedia_skppbj[]" value="{{ isset($decodedSkppbj[1]['penyedia_skppbj']) ? $decodedSkppbj[1]['penyedia_skppbj'] : '' }}"/>
                                            </td>
                                            <td class="">
                                                <input type="text" class="form-control angka" name="nilai_skppbj[]" value="{{ isset($decodedSkppbj[1]['nilai_skppbj']) ? $decodedSkppbj[1]['nilai_skppbj'] : '' }}"/>
                                            </td>
                                            <td class="">
                                                <input type="radio" class="form-check-input" name="pemenang_skppbj[]" @if(isset($decodedSkppbj[1]['pemenang_skppbj']) && $decodedSkppbj[1]['pemenang_skppbj'] == 1) checked @endif/>
                                            </td>
                                        </tr>
                                        <tr>
                                            <td class="text-center">3</td>
                                            <td class="">
                                                <input type="text" class="form-control" name="penyedia_skppbj[]" value="{{ isset($decodedSkppbj[2]['penyedia_skppbj']) ? $decodedSkppbj[2]['penyedia_skppbj'] : '' }}" />
                                            </td>
                                            <td class="">
                                                <input type="text" class="form-control angka" name="nilai_skppbj[]" value="{{ isset($decodedSkppbj[2]['nilai_skppbj']) ? $decodedSkppbj[2]['nilai_skppbj'] : '' }}" />
                                            </td>
                                            <td class="">
                                                <input type="radio" class="form-check-input" name="pemenang_skppbj[]" @if(isset($decodedSkppbj[2]['pemenang_skppbj']) && $decodedSkppbj[2]['pemenang_skppbj'] == 1) checked @endif/>
                                            </td>
                                        </tr>
                                    </tbody>
                                </table>
                            </div>
                        </div>
                        <div class="row">
                            <div class="hstack gap-2">
                                    @if(!$readOnly)
                                <button type="button" class="btn btn-primary" id="simpan-skppbj">Simpan</button>
                                @endif
                                <a href="{{url($controller.'/cetakSkppbj/'.$model['id'])}}" type="button" class="btn btn-warning" target="_blank">Cetak</a>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    </div>
</div>

{{-- Part SPK --}}
<div class="row" style="margin-bottom: 5px;">
    <div class="col-lg-12 mb-12">
        <div class="accordion accordion-flush" id="accordionSPK">

            <div class="accordion-item">
                <h2 class="accordion-header" id="flush-headingOne">
                    <button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target="#flush-spk" aria-expanded="false" aria-controls="flush-collapseOne">
                        3. Surat Perintah Kerja (SPK)
                    </button>
                </h2>
                <div id="flush-spk" class="accordion-collapse collapse" aria-labelledby="flush-headingOne" data-bs-parent="#accordionSPK">
                    <div class="accordion-body">
                        <div class="row">
                            <div class="col-lg-8">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nomor SPK *</label>
                                    <input type="text" class="form-control" id="no_spk" name="no_spk" value="{{ $spk['no_spk'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-4">
                                <div class="mb-3">
                                    @include('components.datepicker',[
                                        'value'=>$spk['tgl_spk']??'',
                                        'label'=>'Tanggal SPK',
                                        'name'=>'tgl_spk'
                                        ]
                                    )
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-8">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nomor Surat Permintaan Penawaran *</label>
                                    <input type="text" class="form-control" id="no_permintaan_spk" name="no_permintaan_spk" value="{{ $spk['no_permintaan'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-4">
                                <div class="mb-3">
                                    @include('components.datepicker',[
                                        'value'=>$spk['tgl_permintaan']??'',
                                        'label'=>'Tanggal',
                                        'name'=>'tgl_permintaan_spk'
                                        ]
                                    )
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-8">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nomor Berita Acara Negoisasi *</label>
                                    <input type="text" class="form-control" id="no_ba_spk" name="no_ba_spk" value="{{ $spk['no_ba'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-4">
                                <div class="mb-3">
                                    @include('components.datepicker',[
                                        'value'=>$spk['tgl_ba']??'',
                                        'label'=>'Tanggal',
                                        'name'=>'tgl_ba_spk'
                                        ]
                                    )
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-4">
                                <div class="mb-3">
                                    @include('components.datepicker',[
                                        'value'=>$spk['tgl_mulai']??'',
                                        'label'=>'Tanggal Mulai Pelaksanaan',
                                        'name'=>'tgl_mulai_spk'
                                        ]
                                    )
                                </div>
                            </div>
                            <div class=" col-lg-4">
                                <div class="mb-3">
                                    @include('components.datepicker',[
                                        'value'=>$spk['tgl_selesai']??'',
                                        'label'=>'Tanggal Selesai Pelaksanaan',
                                        'name'=>'tgl_selesai_spk'
                                        ]
                                    )
                                </div>
                            </div>
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nama Penandatangan Penyedia *</label>
                                    <input type="text" class="form-control" id="nama_penyedia_spk" name="nama_penyedia_spk" value="{{ $spk['nama_penyedia'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-12">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Keterangan SPK *</label>
                                    <textarea class="form-control ckeditor" id="keterangan_spk" name="keterangan_spk" >{{ $spk['keterangan'] ?? '' }}</textarea>
                                </div>
                            </div>
                            <div class=" col-lg-12">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Instruksi Kepada Penyedia *</label>
                                    <textarea class="form-control" id="instruksi_spk" name="instruksi_spk" >{{ $spk['instruksi'] ?? '' }}</textarea>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="hstack gap-2">
                                 @if(!$readOnly)
                                <button type="button" class="btn btn-primary" id="simpan-spk">Simpan</button>
                                @endif
                                <a href="{{url($controller.'/cetakSpk/'.$model['id'])}}" type="button" class="btn btn-warning" target="_blank">Cetak</a>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    </div>
</div>

{{-- Part Ringkasan Kontrak --}}
<div class="row" style="margin-bottom: 5px;">
    <div class="col-lg-12 mb-12">
        <div class="accordion accordion-flush" id="accordionRINGKASAN">
            <div class="accordion-item">
                <h2 class="accordion-header" id="flush-headingOne">
                    <button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target="#flush-ringkasan" aria-expanded="false" aria-controls="flush-collapseOne">
                        4. Ringkasan Kontrak
                    </button>
                </h2>
                <div id="flush-ringkasan" class="accordion-collapse collapse" aria-labelledby="flush-headingOne" data-bs-parent="#accordionRINGKASAN">
                    <div class="accordion-body">

                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nomor DIPA *</label>
                                    <input type="text" class="form-control" id="no_dipa_ringkasan" name="no_dipa_ringkasan" value="{{ $ringkasan['no_dipa'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    @include('components.datepicker',[
                                        'value'=>$ringkasan['tgl_dipa']??'',
                                        'label'=>'Tanggal',
                                        'name'=>'tgl_dipa_ringkasan'
                                        ]
                                    )
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Cara Pembayaran</label>
                                    <input type="text" class="form-control" id="cara_pembayaran_ringkasan" name="cara_pembayaran_ringkasan" value="{{ $ringkasan['cara_pembayaran'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Alamat Kantor Penyedia</label>
                                    <input type="text" class="form-control" id="alamat_kantor_ringkasan" name="alamat_kantor_ringkasan" value="{{ $ringkasan['alamat_penyedia'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nama Bank Rek. Penyedia</label>
                                    <input type="text" class="form-control" id="nama_bank_ringkasan" name="nama_bank_ringkasan" value="{{ $ringkasan['nama_bank'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Kantor Cabang Bank</label>
                                    <input type="text" class="form-control" id="kantor_cabang_ringkasan" name="kantor_cabang_ringkasan" value="{{ $ringkasan['kantor_bank'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">No Rekening</label>
                                    <input type="text" class="form-control" id="no_rekening_ringkasan" name="no_rekening_ringkasan" value="{{ $ringkasan['no_rek'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">NPWP</label>
                                    <input type="text" class="form-control" id="npwp_ringkasan" name="npwp_ringkasan" value="{{ $ringkasan['npwp'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Ketentuan Sanksi</label>
                                    <input type="text" class="form-control" id="ketentuan_sanksi_ringkasan" name="ketentuan_sanksi_ringkasan" value="{{ $ringkasan['sanksi'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="hstack gap-2">
                                 @if(!$readOnly)
                                <button type="button" class="btn btn-primary" id="simpan-ringkasan">Simpan</button>
                                @endif
                                <a href="{{url($controller.'/cetakRingkasan/'.$model['id'])}}" type="button" class="btn btn-warning" target="_blank">Cetak</a>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    </div>
</div>

{{-- Part Kontrak --}}
<div class="row" style="margin-bottom: 5px;">
    <div class="col-lg-12 mb-12">
        <div class="accordion accordion-flush" id="accordionKONTRAK">

            <div class="accordion-item">
                <h2 class="accordion-header" id="flush-headingOne">
                    <button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target="#flush-kontrak" aria-expanded="false" aria-controls="flush-collapseOne">
                        5. Kontrak
                    </button>
                </h2>
                <div id="flush-kontrak" class="accordion-collapse collapse" aria-labelledby="flush-headingOne" data-bs-parent="#accordionKONTRAK">
                    <div class="accordion-body">

                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nomor Kontrak *</label>
                                    <input type="text" class="form-control" id="no_kontrak" name="no_kontrak" value="{{ $kontrak['no_kontrak'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    @include('components.datepicker',[
                                        'value'=>$kontrak['tgl_kontrak']??'',
                                        'label'=>'Tanggal Kontrak',
                                        'name'=>'tgl_kontrak'
                                        ]
                                    )
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="hstack gap-2">
                                 @if(!$readOnly)
                                <button type="button" class="btn btn-primary" id="simpan-kontrak">Simpan</button>
                                @endif
                                <a href="{{url($controller.'/cetakKontrak/'.$model['id'])}}" type="button" class="btn btn-warning" target="_blank">Cetak</a>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    </div>
</div>

{{-- Part BAST --}}
<div class="row" style="margin-bottom: 5px;">
    <div class="col-lg-12 mb-12">
        <div class="accordion accordion-flush" id="accordionBAST">

            <div class="accordion-item">
                <h2 class="accordion-header" id="flush-headingOne">
                    <button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target="#flush-bast" aria-expanded="false" aria-controls="flush-collapseOne">
                        6. BAST
                    </button>
                </h2>
                <div id="flush-bast" class="accordion-collapse collapse" aria-labelledby="flush-headingOne" data-bs-parent="#accordionBAST">
                    <div class="accordion-body">

                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nomor BAST *</label>
                                    <input type="text" class="form-control" id="no_bast" name="no_bast" value="{{ $bast['no_bast'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    @include('components.datepicker',[
                                        'value'=>$bast['tgl_bast']??'',
                                        'label'=>'Tanggal BAST',
                                        'name'=>'tgl_bast'
                                        ]
                                    )
                                </div>
                            </div>
                        </div>

                        <div class="col-lg-12">
                            <h5>Pejabat Mengetahui</h5>
                            <hr/>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Nama </label>
                                <input type="text" class="form-control" id="nama_mengetahui_bast" name="nama_mengetahui_bast" value="{{ $bast['nama_pejabat'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="nip" class="form-label">NIP </label>
                                <input type="text" class="form-control" id="nip_mengetahui_bast" name="nip_mengetahui_bast" value="{{ $bast['nip_pejabat'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Pangkat </label>
                                <input type="text" class="form-control" id="pangkat_mengetahui_bast" name="pangkat_mengetahui_bast" value="{{ $bast['pangkat_pejabat'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Jabatan </label>
                                <input type="text" class="form-control" id="jabatan_mengetahui_skppbj" name="jabatan_mengetahui_skppbj" value="{{ $bast['jabatan_pejabat'] ?? '' }}">
                            </div>
                        </div>
                        <!-- <div class="col-lg-9">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Alamat </label>
                                <input type="text" class="form-control" id="alamat_mengetahui_skppbj" name="alamat_mengetahui_skppbj" value="{{ $model['alamat_mengetahui_skppbj'] ?? '' }}">
                            </div>
                        </div> -->

                        <div class="row">
                            <div class="hstack gap-2">
                                 @if(!$readOnly)
                                <button type="button" class="btn btn-primary" id="simpan-bast">Simpan</button>
                                @endif
                                <a href="{{url($controller.'/cetakBast/'.$model['id'])}}" type="button" class="btn btn-warning" target="_blank">Cetak</a>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    </div>
</div>

{{-- Part BAPP --}}
<div class="row" style="margin-bottom: 5px;">
    <div class="col-lg-12 mb-12">
        <div class="accordion accordion-flush" id="accordionBAPP">

            <div class="accordion-item">
                <h2 class="accordion-header" id="flush-headingOne">
                    <button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target="#flush-bapp" aria-expanded="false" aria-controls="flush-collapseOne">
                        7. BAPP
                    </button>
                </h2>
                <div id="flush-bapp" class="accordion-collapse collapse" aria-labelledby="flush-headingOne" data-bs-parent="#accordionBAPP">
                    <div class="accordion-body">

                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nomor BAPP *</label>
                                    <input type="text" class="form-control" id="no_bapp" name="no_bapp" value="{{ $bast['no_bapp'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    @include('components.datepicker',[
                                        'value'=>$bast['tgl_bapp']??'',
                                        'label'=>'Tanggal BAPP',
                                        'name'=>'tgl_bapp'
                                        ]
                                    )
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nomor Keputusan Jaksa Agung RI *</label>
                                    <input type="text" class="form-control" id="no_kep_ja_bapp" name="no_kep_ja_bapp" value="{{ $bast['no_kepja'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    @include('components.datepicker',[
                                        'value'=>$bast['tgl_kepja']??'',
                                        'label'=>'Tanggal Keputusan JA',
                                        'name'=>'tgl_kep_ja_bapp'
                                        ]
                                    )
                                </div>
                            </div>
                        </div>

                        <div class="row">
                            <div class="hstack gap-2">
                                 @if(!$readOnly)
                                <button type="button" class="btn btn-primary" id="simpan-bapp">Simpan</button>
                                @endif
                                <a href="{{url($controller.'/cetakBapp/'.$model['id'])}}" type="button" class="btn btn-warning" target="_blank">Cetak</a>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    </div>
</div>

{{-- Part Nota Dinas Pengantar Kuitansi --}}
<div class="row" style="margin-bottom: 25px;">
    <div class="col-lg-12 mb-12">
        <div class="accordion accordion-flush" id="accordionKUITANSI">

            <div class="accordion-item">
                <h2 class="accordion-header" id="flush-headingOne">
                    <button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target="#flush-kuitansi" aria-expanded="false" aria-controls="flush-collapseOne">
                        8. Nota Dinas Pengantar Kuitansi
                    </button>
                </h2>
                <div id="flush-kuitansi" class="accordion-collapse collapse" aria-labelledby="flush-headingOne" data-bs-parent="#accordionKUITANSI">
                    <div class="accordion-body">

                        <div class="row">
                            <div class="col-lg-9">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nomor Nota Dinas *</label>
                                    <input type="text" class="form-control" id="no_nodis" name="no_nodis" value="{{ $nodis['no_nodis'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-3">
                                <div class="mb-3">
                                    @include('components.datepicker',[
                                        'value'=>$model['tgl_nodis']??'',
                                        'label'=>'Tanggal Nota Dinas',
                                        'name'=>'tgl_nodis'
                                        ]
                                    )
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Yth</label>
                                    <input type="text" class="form-control" id="yth_nodis" name="yth_nodis" value="{{ $nodis['yth'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Selaku</label>
                                    <input type="text" class="form-control" id="selaku_nodis" name="selaku_nodis" value="{{ $nodis['selaku'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Dari</label>
                                    <input type="text" class="form-control" id="dari_nodis" name="dari_nodis" value="{{ $nodis['dari'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Sifat</label>
                                    <input type="text" class="form-control" id="sifat_nodis" name="sifat_nodis" value="{{ $nodis['sifat'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Lampiran</label>
                                    <input type="text" class="form-control" id="lampiran_nodis" name="lampiran_nodis" value="{{ $nodis['lampiran'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-4">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Hal</label>
                                    <input type="text" class="form-control" id="hal_nodis" name="hal_nodis" value="{{ $nodis['hal'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="col-lg-12">
                            <h5>Pejabat Penandatangan</h5>
                            <hr/>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Nama </label>
                                <input type="text" class="form-control" id="nama_mengetahui_nodis" name="nama_mengetahui_nodis" value="{{ $nodis['nama_pejabat'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="nip" class="form-label">NIP </label>
                                <input type="text" class="form-control" id="nip_mengetahui_nodis" name="nip_mengetahui_nodis" value="{{ $nodis['nip_pejabat'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Pangkat </label>
                                <input type="text" class="form-control" id="pangkat_mengetahui_nodis" name="pangkat_mengetahui_nodis" value="{{ $nodis['pangkat_pejabat'] ?? '' }}">
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="nip" class="form-label">Jabatan </label>
                                <input type="text" class="form-control" id="jabatan_mengetahui_nodis" name="jabatan_mengetahui_nodis" value="{{ $nodis['jabatan_pejabat'] ?? '' }}">
                            </div>
                        </div>
                        <div class="row">
                            <div class="hstack gap-2">
                                 @if(!$readOnly)
                                <button type="button" class="btn btn-primary" id="simpan-nodis">Simpan</button>
                                @endif
                                <a href="{{url($controller.'/cetakNodis/'.$model['id'])}}" type="button" class="btn btn-warning" target="_blank">Cetak</a>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    </div>
</div>

<script>
    /* START HPS */
    $('#tambah-hps').click(function(){
        var tabel	= $('#table_hps > tbody').find('tr:last');
        var newId	= (tabel.length > 0)?parseInt(tabel.data('id'))+1:1;

        $('#table_hps').append(
            '<tr data-id="'+newId+'">' +
            '<td class="text-center"><input type="checkbox" name="chk_del_tembusan[]" class="hRow form-check-input" id="chk_del_tembusan'+newId+'" value="'+newId+'"></td>'+
            '<td><input type="text" class="form-control" id="uraian_barang_'+newId+'" name="uraian_barang[]"></td>' +
            '<td><input type="text" class="form-control angka" id="qty_barang_'+newId+'" name="qty_barang[]"></td>' +
            '<td><input type="text" class="form-control" id="satuan_barang_'+newId+'" name="satuan_barang[]"></td>' +
            '<td><input type="text" class="form-control angka" id="harga_satuan_'+newId+'" name="harga_satuan[]"></td>' +
            '<td><input type="text" class="form-control angka" readonly id="harga_total_'+newId+'" name="harga_total[]"></td>' +
            '</tr>'
        );
        $('#table_hps').find("input[name='no_urut[]']").each(function(i,v){$(v).val(i+1);});
    });

    $('.qty, .harga_satuan').on('input', function() {
        var row = $(this).closest('tr');
        var qty = parseFloat(row.find('.qty').val());
        var hargaSatuan = parseFloat(row.find('.harga_satuan').val());
        var total = qty * hargaSatuan;
        if (!isNaN(total)) {
            row.find('.harga_total').val(total.toFixed(0));
        }
    });

    $(".hapusHps").click(function(){
        var tabel 	= $("#table_hps");
        tabel.find(".hRow:checked").each(function(k, v){
            var idnya = $(v).val();
            tabel.find("tr[data-id='"+idnya+"']").remove();
        });
        tabel.find("input[name='no_urut[]']").each(function(i,v){$(this).val(i+1);});
    });

    $('#simpan-hps').on('click', function(){
        const id = $('#id').val();
        const no_hps = $('#no_hps').val();
        const tgl_hps = $('#tgl_hps').val();
        const nip_penandatangan = $('#nip_penandatangan_hps').val();
        const nama_penandatangan = $('#nama_penandatangan_hps').val();
        const pangkat_penandatangan = $('#pangkat_penandatangan_hps').val();
        const keterangan = CKEDITOR.instances.keterangan_hps.getData();
        var barang = [];
        $('#table_hps tbody tr').each(function(index, tr) {
            var rowData = {
                'uraian_barang': $(tr).find('[name="uraian_barang[]"]').val(),
                'qty_barang': $(tr).find('[name="qty_barang[]"]').val(),
                'satuan_barang': $(tr).find('[name="satuan_barang[]"]').val(),
                'harga_satuan': $(tr).find('[name="harga_satuan[]"]').val(),
                'harga_total': $(tr).find('[name="harga_total[]"]').val()
            };
            barang.push(rowData);
        });
        var data = {
            id,no_hps,tgl_hps,nip_penandatangan,nama_penandatangan,pangkat_penandatangan,keterangan,barang
        };
        $.ajax({
            url: `{{ $controller. '/saveHps' }}`,
            method: 'POST',
            data: data,
            success: function(response) {
                notify({
                    type: "success",
                    message: "Data Berhasil Disimpan",
                });
            },
            error: showError,
        });
    });
    /* END HPS */

    /* START SKPPBJ */
    $('#simpan-skppbj').on('click', function(){
        const id = $('#id').val();
        const nama_penandatangan = $('#nama_penandatangan_skppbj').val();
        const nip_penandatangan = $('#nip_penandatangan_skppbj').val();
        const pangkat_penandatangan = $('#pangkat_penandatangan_skppbj').val();
        const jabatan_penandatangan = $('#jabatan_penandatangan_skppbj').val();
        const alamat = $('#alamat_skppbj').val();
        const tgl_skppbj = $('#tgl_skppbj').val();
        var penyedia = [];
        $('#table_skppbj tbody tr').each(function(index, tr) {
            var rowData = {
                'penyedia_skppbj': $(tr).find('[name="penyedia_skppbj[]"]').val(),
                'nilai_skppbj': $(tr).find('[name="nilai_skppbj[]"]').val(),
                'pemenang_skppbj': ($(tr).find('[name="pemenang_skppbj[]"]').prop("checked") == true ? '1' : '0'),
            };
            penyedia.push(rowData);
        });
        var data = {
            id,nama_penandatangan,nip_penandatangan,pangkat_penandatangan,jabatan_penandatangan,alamat,tgl_skppbj,penyedia
        };
        $.ajax({
            url: `{{ $controller. '/saveSkppbj' }}`,
            method: 'POST',
            data: data,
            success: function(response) {
                notify({
                    type: "success",
                    message: "Data Berhasil Disimpan",
                });
            },
            error: showError,
        });
    });
    /* END SKPPBJ */

    /* START SPK */
    $('#simpan-spk').on('click', function(){
        const id = $('#id').val();
        const no_spk = $('#no_spk').val();
        const no_permintaan = $('#no_permintaan_spk').val();
        const tgl_permintaan = $('#tgl_permintaan_spk').val();
        const no_ba = $('#no_ba_spk').val();
        const tgl_ba = $('#tgl_ba_spk').val();
        const tgl_mulai = $('#tgl_mulai_spk').val();
        const tgl_spk = $('#tgl_spk').val();
        const tgl_selesai = $('#tgl_selesai_spk').val();
        const nama_penyedia = $('#nama_penyedia_spk').val();
        const keterangan = CKEDITOR.instances.keterangan_spk.getData();
        const instruksi = $('#instruksi_spk').val();
        var data = {
            id,no_spk,no_permintaan,tgl_permintaan,no_ba,tgl_ba,tgl_mulai,tgl_spk,tgl_selesai,nama_penyedia,keterangan,instruksi
        };
        $.ajax({
            url: `{{ $controller. '/saveSpk' }}`,
            method: 'POST',
            data: data,
            success: function(response) {
                notify({
                    type: "success",
                    message: "Data Berhasil Disimpan",
                });
            },
            error: showError,
        });
    });
    /* END SPK */

    /* START RINGKASAN */
    $('#simpan-ringkasan').on('click', function(){
        const id = $('#id').val();
        const no_dipa = $('#no_dipa_ringkasan').val();
        const tgl_dipa = $('#tgl_dipa_ringkasan').val();
        const cara_pembayaran = $('#cara_pembayaran_ringkasan').val();
        const alamat_penyedia = $('#alamat_kantor_ringkasan').val();
        const nama_bank = $('#nama_bank_ringkasan').val();
        const kantor_bank = $('#kantor_cabang_ringkasan').val();
        const no_rek = $('#no_rekening_ringkasan').val();
        const npwp = $('#npwp_ringkasan').val();
        const sanksi = $('#ketentuan_sanksi_ringkasan').val();
        var data = {
            id,no_dipa,tgl_dipa,cara_pembayaran,alamat_penyedia,nama_bank,kantor_bank,no_rek,npwp,sanksi
        };
        $.ajax({
            url: `{{ $controller. '/saveRingkasan' }}`,
            method: 'POST',
            data: data,
            success: function(response) {
                notify({
                    type: "success",
                    message: "Data Berhasil Disimpan",
                });
            },
            error: showError,
        });
    });
    /* END RINGKASAN */

    /* START KONTRAK */
    $('#simpan-kontrak').on('click', function(){
        const id = $('#id').val();
        const no_kontrak = $('#no_kontrak').val();
        const tgl_kontrak = $('#tgl_kontrak').val();
        var data = {
            id,no_kontrak,tgl_kontrak
        };
        $.ajax({
            url: `{{ $controller. '/saveKontrak' }}`,
            method: 'POST',
            data: data,
            success: function(response) {
                notify({
                    type: "success",
                    message: "Data Berhasil Disimpan",
                });
            },
            error: showError,
        });
    });
    /* END KONTRAK */

    /* START BAST */
    $('#simpan-bast').on('click', function(){
        const id = $('#id').val();
        const no_bast = $('#no_bast').val();
        const tgl_bast = $('#tgl_bast').val();
        const nama_pejabat = $('#nama_mengetahui_bast').val();
        const nip_pejabat = $('#nip_mengetahui_bast').val();
        const pangkat_pejabat = $('#pangkat_mengetahui_bast').val();
        const jabatan_pejabat = $('#jabatan_mengetahui_skppbj').val();
        var data = {
            id,no_bast,tgl_bast,nama_pejabat,nip_pejabat,pangkat_pejabat,jabatan_pejabat
        };
        $.ajax({
            url: `{{ $controller. '/saveBast' }}`,
            method: 'POST',
            data: data,
            success: function(response) {
                notify({
                    type: "success",
                    message: "Data Berhasil Disimpan",
                });
            },
            error: showError,
        });
    });
    /* END BAST */

    /* START BAPP */
    $('#simpan-bapp').on('click', function(){
        const id = $('#id').val();
        const no_bapp = $('#no_bapp').val();
        const tgl_bapp = $('#tgl_bapp').val();
        const no_kepja = $('#no_kep_ja_bapp').val();
        const tgl_kepja = $('#tgl_kep_ja_bapp').val();
        var data = {
            id,no_bapp,tgl_bapp,no_kepja,tgl_kepja
        };
        $.ajax({
            url: `{{ $controller. '/saveBapp' }}`,
            method: 'POST',
            data: data,
            success: function(response) {
                notify({
                    type: "success",
                    message: "Data Berhasil Disimpan",
                });
            },
            error: showError,
        });
    });
    /* END BAPP */

    /* START NODIS */
    $('#simpan-nodis').on('click', function(){
        const id = $('#id').val();
        const no_nodis = $('#no_nodis').val();
        const tgl_nodis = $('#tgl_nodis').val();
        const yth = $('#yth_nodis').val();
        const selaku = $('#selaku_nodis').val();
        const dari = $('#dari_nodis').val();
        const sifat = $('#sifat_nodis').val();
        const lampiran = $('#lampiran_nodis').val();
        const hal = $('#hal_nodis').val();
        const nama_pejabat = $('#nama_mengetahui_nodis').val();
        const nip_pejabat = $('#nip_mengetahui_nodis').val();
        const pangkat_pejabat = $('#pangkat_mengetahui_nodis').val();
        const jabatan_pejabat = $('#jabatan_mengetahui_nodis').val();
        var data = {
            id,no_nodis,tgl_nodis,tgl_nodis,yth,selaku,dari,sifat,lampiran,hal,nama_pejabat,nip_pejabat,pangkat_pejabat,jabatan_pejabat
        };
        $.ajax({
            url: `{{ $controller. '/saveNodis' }}`,
            method: 'POST',
            data: data,
            success: function(response) {
                notify({
                    type: "success",
                    message: "Data Berhasil Disimpan",
                });
            },
            error: showError,
        });
    });
    /* END NODIS */
</script>

