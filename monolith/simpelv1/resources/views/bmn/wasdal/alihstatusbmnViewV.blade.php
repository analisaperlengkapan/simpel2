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
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Satuan Kerja </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['nama_satker'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Tahun Anggaran</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['tahun_anggaran'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">No. SK </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['no_sk'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <!-- <label for="kdsatker_keu" class="form-label">Tgl. SK</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['tgl_sk'] ?? '-' }}" readonly/> -->
                                @include('components.datepicker', [
                                    'value' => $model['tgl_sk'] ?? '',
                                    'label' => 'Tgl. SK',
                                    'name' => 'tgl_sk',
                                    'className' => 'form-control-plaintext',
                                ])
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Jenis Aset </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['jenis_aset'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Total BMN </label>
                                <input type="text" class="form-control-plaintext angka" value="{{ $model['total_bmn'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Nilai Penetapan</label>
                                <input type="text" class="form-control-plaintext angka" value="{{ $model['nilai_penetapan'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-12">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Uraian </label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['uraian_keputusan'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Korwil</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['korwil'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Es. 1</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['es1'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">KPKNL</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['kpknl'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Kanwil</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['kanwil'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Nama Penerbit SK</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['nama_penerbit_sk'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Nama Pihak Lain</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['nama_pihak_lain'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Alamat Pihak Lain</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['alamat_pihak_lain'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">NIP Penandatangan</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['nip_penandatangan'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Nama Penandatangan</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['nama_penandatangan'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Jabatan</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['jabatan_ttd'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Tipe Pemohon</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['tipe_pemohon'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Kode Pemohon</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['kode_pemohon'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                        <div class="col-lg-4">
                            <div class="mb-3">
                                <label for="kdsatker_keu" class="form-label">Nama Pemohon</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['nama_pemohon'] ?? '-' }}" readonly/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-lg-3">
                            <div class="mb-3">
                                <!-- <label for="kdsatker_keu" class="form-label">Tgl. Tarik Data SIMAN</label>
                                <input type="text" class="form-control-plaintext" value="{{ $model['tgl_tarik'] ?? '-' }}" readonly/> -->

                                @include('components.datepicker', [
                                    'value' => $model['tgl_tarik'] ?? '',
                                    'label' => 'Tgl. Tarik Data SIMAN',
                                    'name' => 'tgl_tarik',
                                    'className' => 'form-control-plaintext'
                                ])
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Detail Data Aset</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <table id="tb-monsakti-traset" class="display table table-bordered dt-responsive" style="width:100%">
                        <thead>
                            <tr>
                                <th class="text-center">Kode Aset</th>
                                <th class="text-center">NUP</th>
                                <th class="text-center">Nama Aset</th>
                                <th class="text-center">Tgl Perolehan</th>
                                <th class="text-center">Nilai Perolehan</th>
                                <th class="text-center">Nilai Buku</th>
                                <th class="text-center">Kuantitas</th>
                                <th class="text-center">Nilai Persetujuan</th>
                            </tr>
                        </thead>
                        <tbody>
                            @foreach($detail as $data)
                                <tr>
                                    <td class="text-center">{{$data->kode_barang}}</td>
                                    <td class="text-center">{{$data->nup}}</td>
                                    <td>{{$data->nama_barang}}</td>
                                    <td width="15%" class="text-center">{{$data->tgl_perolehan}}</td>
                                    <td class="angka" style="text-align: right;">{{$data->nilai_perolehan}}</td>
                                    <td class="angka" style="text-align: right;">{{$data->nilai_buku}}</td>
                                    <td class="angka" style="text-align: right;">{{$data->kuantitas}}</td>
                                    <td class="angka" style="text-align: right;">{{$data->nilai_persetujuan}}</td>
                                </tr>
                            @endforeach
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
        <!--end col-->
    </div>
    
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-body">
                    <div class="hstack gap-2">
                        <a href="{{ url($controller) }}" class="btn btn-outline-primary">Kembali</a>
                    </div>
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
        })
    </script>
@endsection
