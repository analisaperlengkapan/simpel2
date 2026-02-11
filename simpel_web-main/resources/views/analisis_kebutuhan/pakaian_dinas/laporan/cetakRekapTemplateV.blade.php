@include('components.cetakanCss')
<h2 style="padding:0px;margin:0px;text-align: center">{{ $header->nama }}</h2>
<h3 style="padding:0px;margin:0px;text-align: center">{{ $header->inst_nama }}</h3>
<p style="text-align: center">Periode: {{ MyHelper::dateFormatIndo($header->tgl_mulai) }} S/D
    {{ MyHelper::dateFormatIndo($header->tgl_selesai) }}
</p>
@foreach ($filter as $fil)
    <p style="text-align: left">
        <span>{{ $fil['label'] }}</span>
        <span style="margin-left: 1px">:</span>
        <span style="margin-left:2px">{{ $fil['value'] }}</span>
    </p>
@endforeach
@foreach ($pakaians as $pakaian)
    @php
        $ukurans = explode(',', $pakaian->ukurans);
        switch ($pakaian->subspesifikasi_gender) {
            case 'L':
                $genders = ['l'];
                break;
            case 'P':
                $genders = ['p'];
                break;
            default:
                $genders = ['l', 'p'];
                break;
        }
    @endphp
    @foreach ($genders as $gender)
        <div style="{{ $loop->parent->index > 0 || $loop->index > 0 ? 'page-break-before:always' : '' }}">
            <table style="width:100%;">
                <thead>
                    <tr>
                        <th class="text-center" width="5%" rowspan="2">No</th>
                        <th class="text-center" width="20%" rowspan="2">Satker</th>
                        <th class="text-center" colspan="{{ count($ukurans) }}">
                            {{ $pakaian->spesifikasi_nama }} - {{ strtoupper($gender) }}</th>
                        <th rowspan="2">Jumlah</th>
                    </tr>

                    <tr>
                        @foreach ($ukurans as $ukuran)
                            <th class="text-center">{{ $ukuran }}</th>
                        @endforeach
                    </tr>
                </thead>
                <tbody>
                    @foreach ($listSatker as $satker)
                        @php
                            $total = 0;
                        @endphp
                        <tr>
                            <td class="text-center">{{ $loop->index + 1 }}</td>
                            <td>{{ $satker->inst_nama }}</td>
                            @foreach ($ukurans as $ukuran)
                                @php
                                    $ukuranya = $dataUkuran[$pakaian->id][$satker->inst_satkerkd][$ukuran][$gender] ?? 0;
                                    $total = $total + $ukuranya;
                                @endphp
                                <td class="text-center">{{ $ukuranya }}</td>
                            @endforeach
                            <td class="text-center font-bold">{{ $total }}</td>
                        </tr>
                    @endforeach
                    <tr>
                        @php
                            $totalSummary = 0;
                        @endphp
                        <td colspan="2" class="font-bold text-center">Jumlah</td>
                        @foreach ($ukurans as $ukuran)
                            @php
                                $ukuranya = $dataSummaryPakaian[$pakaian->id][$ukuran][$gender] ?? 0;
                                $totalSummary = $totalSummary + $ukuranya;
                            @endphp
                            <td class="text-center font-bold">{{ $ukuranya }}</td>
                        @endforeach
                        <td class="text-center font-bold">{{ $totalSummary }}</td>
                    </tr>
                </tbody>
            </table>
        </div>
    @endforeach
@endforeach
