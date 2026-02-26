@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <form action="{{ $controller }}" method="POST" class="ajaxForm">
    <div class="row">
        <div class="col-lg-12">
            <div class="card ">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $isNew ? 'Tambah' : 'Edit' }} Pengajuan Kebutuhan BMN</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body p-4">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="nama" class="form-label">Tahun Anggaran</label>
                                    <select class="form-control" data-choices data-choices-search-false name="tahun" id="tahun">
                                        {!! $yearOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row" id="div-tanggal">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    @include('components.datepicker', [
                                        'value' => $model['tgl_mulai'] ?? '',
                                        'label' => 'Tanggal Mulai',
                                        'name' => 'tgl_mulai',
                                    ])
                                </div>
                            </div>
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    @include('components.datepicker', [
                                        'value' => $model['tgl_selesai'] ?? '',
                                        'label' => 'Tanggal Selesai',
                                        'name' => 'tgl_selesai',
                                    ])
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="nama" class="form-label">Nama Permintaan</label>
                                    <input class="form-control" id="nama" name="nama" placeholder="nama" value="{{ $model['nama'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="nama" class="form-label">Deskripsi</label>
                                    <textarea name="deskripsi" id="deskripsi" class="form-control" rows="5">{{ $model['deskripsi'] ?? '' }}</textarea>
                                </div>
                            </div>
                        </div>
                        <!-- <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="jenis_asset" class="form-label">Aset</label>
                                    <select class="form-control" data-choices data-choices-removeItem multiple name="id_jenis_asset[]" id="id_jenis_asset">
                                        <option value="">Pilih Aset</option>
                                        {!! $jenisAssetOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div> -->
                        <div class="row mt-3">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="pilihan_satker" class="form-label">Satker</label>
                                    <select class="form-control" data-choices data-choices-sorting-false id="pilihan_satker"
                                        name="pilihan_satker">
                                        <option value="semua"
                                            @php $selectedSatker = $model['pilihan_satker'] ?? null; @endphp
                                            {{ $selectedSatker == 'semua' ? 'selected' : '' }}>
                                            Semua Satker</option>
                                        <option value="sebagian" {{ $selectedSatker == 'sebagian' ? 'selected' : '' }}>
                                            Sebagian Satker</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row visually-hidden" id="div-pilihan-satker">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="satkers" class="form-label">Pilih Satker</label>
                                    <select class="form-control selectTwo text-black" multiple="multiple" name="satkers[]">
                                        {!! $satkerOptions !!}
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="mb-3">
                                    <label for="nama" class="form-label">Approval Daskrimti ?</label>
                                    <input {{ $model['is_appv_daskrimti']==1?'checked':'' }} type="checkbox" class="form-check-input" id="is_appv_daskrimti" name="is_appv_daskrimti" value="1">
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
                            <a class="btn btn-danger btn-sm hapusTembusan" title="Hapus"><i class="ri-delete-bin-5-fill" style="font-size:14px;"></i></a>
                            <a class="btn btn-success btn-sm" id="tambah-tembusan" title="Tambah Aset"><i class="ri-add-line" style="font-size:14px;"></i>Aset</a><br>
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
                            @if ($isNew)
                            <tr data-id="1">
                                <td class="text-center"></td>
                                <td>
                                    <select id="pilih_barang_1" name="asset_kode_barang[]" class="form-control selectTwo">
                                    @foreach($listBarang as $item)
                                        <option value="{{ $item->kode_barang }}">{{ $item->kode_barang.' - '.$item->nama_barang }}</option>
                                    @endforeach
                                    </select>
                                </td>
                                <td><textarea name="asset_keterangan[]" class="form-control" style="height:50px;"></textarea></td>
                            </tr>
                            @else
                            @foreach($asset as $index => $data)
                            <tr data-id="{{$index}}">
                                <td class="text-center">@if($index>0)<input type="checkbox" name="chk_del_tembusan[]" class="hRow form-check-input" id="chk_del_tembusan{{$index}}" value="{{$index}}">@endif</td>
                                <td>
                                    <select id="pilih_barang_{{$index}}" name="asset_kode_barang[]" class="form-control selectTwo">
                                    @foreach($listBarang as $item)
                                        <option {{ $item->kode_barang==$data['kode_barang']?'selected':'' }} value="{{ $item->kode_barang }}">{{ $item->kode_barang.' - '.$item->nama_barang }}</option>
                                    @endforeach
                                    </select>
                                </td>
                                <td><textarea name="asset_keterangan[]" class="form-control" style="height:50px;">{{$data['keterangan']}}</textarea></td>
                            </tr>
                            @endforeach
                            @endif
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <div class="row">
        <div class="col-lg-12">
            <div class="hstack gap-2 justify-content-start">
                <a href="{{ url($controller) }}" class="btn btn-outline-primary">Kembali</a>
                @if ($isNew)
                    <button type="submit" class="btn btn-primary">
                        {{ $isNew ? 'Simpan' : 'Ubah' }}
                    </button>
                @endif
            </div>
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
            var listBarang = @json($listBarang);
            var dataBarang = listBarang.map(function(item) {
                return {
                    id: item.kode_barang,
                    text: item.kode_barang+' - '+item.nama_barang
                };
            });
            /* START TEMBUSAN */
            $('#tambah-tembusan').click(function(){
                var tabel	= $('#table_tembusan > tbody').find('tr:last');
                var newId	= (tabel.length > 0)?parseInt(tabel.data('id'))+1:1;

                $('#table_tembusan').append(
                    '<tr data-id="'+newId+'">' +
                    '<td class="text-center"><input type="checkbox" name="chk_del_tembusan[]" class="hRow form-check-input" id="chk_del_tembusan'+newId+'" value="'+newId+'"></td>'+
                    '<td><select id="pilih_barang_'+newId+'" name="asset_kode_barang[]" class="form-control"></select></td>' +
                    '<td><textarea name="asset_keterangan[]" class="form-control" style="height:50px;"></textarea></td>' +
                    '</tr>'
                );
                $('#pilih_barang_'+newId).select2({
                    data : dataBarang
                });
                $('#table_tembusan').find("input[name='no_urut[]']").each(function(i,v){$(v).val(i+1);});
            });

            $(".hapusTembusan").click(function(){
                var tabel 	= $("#table_tembusan");
                tabel.find(".hRow:checked").each(function(k, v){
                    var idnya = $(v).val();
                    tabel.find("tr[data-id='"+idnya+"']").remove();
                });
                tabel.find("input[name='no_urut[]']").each(function(i,v){$(this).val(i+1);});
            });
            /* END TEMBUSAN */
        })
    </script>
@endsection
