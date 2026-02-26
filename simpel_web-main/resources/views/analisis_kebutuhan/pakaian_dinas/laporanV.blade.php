@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card ">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Laporan Pengajuan Pakaian Dinas</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body p-4">
                    <form action="{{ $controller }}" method="POST">
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="tahun" class="form-label">Pilih Tahun</label>
                                    <select class="form-control" required id="tahun" name="tahun">
                                        <option value="">Pilih Tahun</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="pengajuan_id" class="form-label">Pilih Pengadaan</label>
                                    <select class="form-control" required id="pengajuan_id" name="pengajuan_id">
                                        <option value="">Pilih Pengadaan</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="jenis_laporan" class="form-label">Jenis Laporan</label>
                                    <select class="form-control" id="jenis_laporan" name="jenis_laporan">
                                        <option value="rekap">Rekap</option>
                                        <option value="daftar">Daftar</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="jenis_file" class="form-label">Tipe File</label>
                                    <select class="form-control" id="jenis_file" name="jenis_file">
                                        <option value="pdf">PDF</option>
                                        <option value="excel">Excel</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row" id="div-wilayah">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="kejati_id" class="form-label">Wilayah</label>
                                    <select class="form-control" id="kejati_id" name="kejati_id">
                                        <option value="">Pilih</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="ms_satker_id" class="form-label">Satuan Kerja</label>
                                    <select class="form-control" required id="ms_satker_id" name="ms_satker_id">
                                        <option value="">Pilih</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div id="div-filter">
                            <div class="row">
                                <div class=" col-lg-6">
                                    <div class="mb-3">
                                        <label for="filter[jenis]" class="form-label">Status Pegawai</label>
                                        <select class="form-control filter" id="filter[jenis]" name="filter[jenis]">
                                            <option value="">Semua Pegawai</option>
                                            <option value="0">Jaksa</option>
                                            <option value="1">TU</option>
                                        </select>
                                    </div>
                                </div>
                            </div>
                            <div class="row">
                                <div class=" col-lg-6">
                                    <div class="mb-3">
                                        <label for="filter[eselon]" class="form-label">Eselon</label>
                                        <select class="form-control filter" id="filter[eselon]" name="filter[eselon]">
                                            <option value="">Semua Eselon</option>
                                            <option value="non">Non Eselon</option>
                                            <option value="I">I</option>
                                            <option value="II">II</option>
                                            <option value="III">III</option>
                                            <option value="IV">IV</option>
                                            <option value="V">V</option>
                                        </select>
                                    </div>
                                </div>
                            </div>
                            <div class="row">
                                <div class=" col-lg-6">
                                    <div class="mb-3">
                                        <label for="filter[jenis_kelamin]" class="form-label">Jenis Kelamin</label>
                                        <select class="form-control filter" id="filter[jenis_kelamin]"
                                            name="filter[jenis_kelamin]">
                                            <option value="">Semua</option>
                                            <option value="L">Laki-Laki</option>
                                            <option value="P">Perempuan</option>
                                        </select>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="row mt-3">
                            <div class="col" id="spec-list">
                            </div>
                            <div class="row mt-3">
                                <div class="col-lg-12">
                                    <div class="hstack gap-2 justify-content-start">
                                        <a href="{{ url($controller) }}" class="btn btn-outline-primary">Kembali</a>
                                        <button type="submit" class="btn btn-primary">
                                            Cetak
                                        </button>
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
        const satkers = {{ Js::from($satkers) }};
        const tahuns = {{ Js::from($tahuns) }};
        const pusats = {{ Js::from($pusats) }};
        const wilayahs = {{ Js::from($wilayahs) }};
        const pengadaans = {{ Js::from($pengadaans) }};
        const url = `{{ $controller }}`

        $(function() {
            $('#jenis_laporan, #jenis_file, #tahun, .filter')
                .select2();
            $('#pengajuan_id').select2({
                data: pengadaans
            })

            $('#ms_satker_id').select2({
                data: satkers
            })

            $('#tahun').select2({
                data: tahuns
            })

            if (wilayahs.length < 1) {
                $('#div-wilayah').toggleClass('visually-hidden', true);
            } else {
                $('#kejati_id').select2({
                    data: wilayahs
                })
            }

            if (`{{ $isPusat }}` == 0) {
                $('#div-eselon').toggleClass('visually-hidden', true);
            }

            $('#kejati_id').on('change', function() {
                const id = $(this).val();
                let filtered = satkers.filter(satker => satker.id.substr(0, id.length) == id)
                if (id == '00') {
                    filtered = pusats;
                }
                $('#ms_satker_id').empty().select2({
                    data: [{
                        id: 'all',
                        text: 'SEMUA SATKER'
                    }, ...filtered]
                }).trigger('change');
            })

            $('#tahun').on('change', function() {
                const tahun = $(this).val();
                $.get(`${url}/getPengadaanByTahun/${tahun}`)
                    .done(res => {
                        $('#pengajuan_id').empty().select2({
                            data: res
                        }).trigger('change');
                    })
            })

            $('form').on('submit', function(e) {
                e.preventDefault();
                const data = $(this).serializeFormJSON();
                const qParams = $.param(data);
                const cetakUrl = `${url}/cetak?${qParams}`
                window.open(cetakUrl, '_blank');
            })
        })
    </script>
@endsection
