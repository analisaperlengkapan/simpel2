@include('components.cetakanCss')
<h2 style="padding:0px;margin:0px;text-align: center">{{ $header->nama }}</h2>
<h3 style="padding:0px;margin:0px;text-align: center">{{ $header->inst_nama }}</h3>
<p style="text-align: center">Periode: {{ MyHelper::dateFormatIndo($header->tgl_mulai) }} S/D
    {{ MyHelper::dateFormatIndo($header->tgl_selesai) }}
</p>

@php
    $sep = '';
    if (isset($isExcel) && $isExcel) {
        $sep = "'";
    }
@endphp
@foreach ($filter as $fil)
    <p style="text-align: left">
        <span>{{ $fil['label'] }}</span>
        <span style="margin-left: 1px">:</span>
        <span style="margin-left:2px">{{ $fil['value'] }}</span>
    </p>
@endforeach
@foreach ($listSatker as $satker)
    <div style="{{ $loop->index > 0 ? 'page-break-before:always' : '' }}">
        <h3 syle="padding: 0px;margin:0px;text-align: cente">{{ $satker->inst_nama }}</h3>
        <table style="width:100%;">
            <thead>
                <tr>
                    <th>No</th>
                    <th>NIP</th>
                    <th>Nama</th>
                    <th>Jabatan</th>
                    <th>Golongan</th>
                    <th>Status</th>
                    <th>Gender</th>
                    <th>Busana Mulsimah</th>
                    @foreach ($pakaians as $pakaian)
                        <th>{{ $pakaian->spesifikasi_nama }}</th>
                    @endforeach
                </tr>
                <thead>
                <tbody>
                    @php
                        // $kdSatker = $isPusat ? $satker->inst_satkerkd : $satker->
                        $dataSatker = $dataPerSatker[$satker->inst_satkerkd] ?? [];
                    @endphp
                    @forelse ($dataSatker as $pegawai)
                        @php
                            $pegawaiUkuran = $mappedUkurans[$pegawai->id] ?? [];
                        @endphp
                        <tr>
                            <td class="text-center">{{ $loop->index + 1 }}</td>
                            <td>{{ $sep }}{{ $pegawai->nip }}</td>
                            <td class="text-left">{{ $pegawai->nama }}</td>
                            <td class="text-left">{{ $pegawai->jabatan }}</td>
                            <td class="text-center">{{ $pegawai->gol_kd ?? '-' }}</td>
                            <td class="text-center">{{ $pegawai->jenis == 0 ? 'J' : 'T' }}</td>
                            <td class="text-center">{{ $pegawai->jenis_kelamin }}</td>
                            <td class="text-center">{{ $pegawai->with_hijab == 1 ? 'Y' : 'T' }}</td>
                            @foreach ($pakaians as $pakaian)
                                <td class="text-center">{{ $pegawaiUkuran[$pakaian->id] ?? '-' }}</td>
                            @endforeach
                        </tr>
                    @empty
                        <tr>
                            <td colspan="11" style="text-align: center">Data Kosong</td>
                        </tr>
                    @endforelse
                </tbody>
        </table>
    </div>
@endforeach
