<div class="row">
    <div class="col-12">
        <div class="card shadow-lg rounded-4 mb-4 border-0">
            <div class="card-header bg-white border-0 pb-0">
                <h4 class="mb-0 fw-bold" style="color:#1e5631;letter-spacing:1px;">REKAPITULASI INSTALASI JARINGAN</h4>
            </div>
            <div class="card-body">
                <div class="table-responsive">
                    <table class="table table-bordered table-hover align-middle mb-0" style="background:#fff7e0;">
                        <thead class="sticky-top" style="background:#f9b233;color:#1e5631;">
                            <tr>
                                <th>JENIS ASET</th>
                                <th width="15%">KUANTITAS</th>
                                <th width="20%">NILAI ASET</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td align="center" colspan="3">
                                    <span class="badge rounded-pill px-3 py-2 fs-6" style="background:#f9b233;color:#1e5631;">KONDISI INSTALASI JARINGAN</span>
                                </td>
                            </tr>
                            @foreach ($kondisi_instalasi_jaringan as $rows)
                                <tr>
                                    <td> {{ $rows->judul }} </td>
                                    <td align="right"><span class="fw-bold fs-5">{{ number_format($rows->total,0,",",".") }}</span></td>
                                    <td align="right"><span class="fw-bold fs-5">Rp. {{ number_format($rows->total_nilai_perolehan,0,",",".") }}</span></td>
                                </tr>
                            @endforeach
                            <tr>
                                <td align="center" colspan="3">
                                    <span class="badge rounded-pill px-3 py-2 fs-6" style="background:#f9b233;color:#1e5631;">KELOMPOK INSTALASI JARINGAN</span>
                                </td>
                            </tr>
                            @foreach ($kelompok_instalasi_jaringan as $rows)
                                <tr>
                                    <td> {{ $rows->judul }} </td>
                                    <td align="right"><span class="fw-bold fs-5">{{ number_format($rows->total,0,",",".") }}</span></td>
                                    <td align="right"><span class="fw-bold fs-5">Rp. {{ number_format($rows->total_nilai_perolehan,0,",",".") }}</span></td>
                                </tr>
                            @endforeach
                            <tr>
                                <td align="center" colspan="3">
                                    <span class="badge rounded-pill px-3 py-2 fs-6" style="background:#f9b233;color:#1e5631;">SUB KELOMPOK INSTALASI JARINGAN</span>
                                </td>
                            </tr>
                            @foreach ($subkelompok_instalasi_jaringan as $rows)
                                <tr>
                                    <td> {{ $rows->judul }} </td>
                                    <td align="right"><span class="fw-bold fs-5">{{ number_format($rows->total,0,",",".") }}</span></td>
                                    <td align="right"><span class="fw-bold fs-5">Rp. {{ number_format($rows->total_nilai_perolehan,0,",",".") }}</span></td>
                                </tr>
                            @endforeach
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    </div>
</div>

        <h5 class="card-title mb-0">REKAPITULASI INSTALASI JARINGAN</h5>
        <br/>
        <table class="display table table-bordered dt-responsive my-dt" style="width:100%">
            <thead>
                <tr>
                    <th>JENIS ASET</th>
                    <th width="15%">KUANTITAS</th>
                    <th width="20%">NILAI ASET</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td align="center" colspan="3" style="background-color:antiquewhite;">KONDISI INSTALASI JARINGAN</td>
                </tr>
                @foreach ($kondisi_instalasi_jaringan as $rows)
                    <tr>
                        <td> {{ $rows->judul }} </td>
                        <td align="right"> {{ number_format($rows->total,0,",",".") }}</td>
                        <td align="right">Rp. {{ number_format($rows->total_nilai_perolehan,0,",",".") }} </td>
                    </tr>
                @endforeach
                <tr>
                    <td align="center" colspan="3" style="background-color:antiquewhite;">KELOMPOK INSTALASI JARINGAN</td>
                </tr>
                @foreach ($kelompok_instalasi_jaringan as $rows)
                    <tr>
                        <td> {{ $rows->judul }} </td>
                        <td align="right"> {{ number_format($rows->total,0,",",".") }} </td>
                        <td align="right">Rp. {{ number_format($rows->total_nilai_perolehan,0,",",".") }} </td>
                    </tr>
                @endforeach
                <tr>
                    <td align="center" colspan="3" style="background-color:antiquewhite;">SUB KELOMPOK INSTALASI JARINGAN</td>
                </tr>
                @foreach ($subkelompok_instalasi_jaringan as $rows)
                    <tr>
                        <td> {{ $rows->judul }} </td>
                        <td align="right"> {{ number_format($rows->total,0,",",".") }} </td>
                        <td align="right">Rp. {{ number_format($rows->total_nilai_perolehan,0,",",".") }} </td>
                    </tr>
                @endforeach
            </tbody>
        </table>

    </div>
</div>