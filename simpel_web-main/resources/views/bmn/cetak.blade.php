<!DOCTYPE html>
<html>
<head>
    <style>
        /* Add your styling here */
        table {
            width: 100%;
            border-collapse: collapse;
        }
        th, td {
            border: 1px solid black;
            padding: 8px;
            text-align: left;
        }
        th {
            background-color: #f2f2f2;
        }
    </style>
</head>
<body>
    <h2 style="text-align: center;">SURAT IZIN PEMAKAIAN BMN</h2>
    <br/><br/>
    @foreach ($data as $dat)
    <div>{{$dat['nama_satker']}}</div>
    <table>
        <thead>
            <tr>
                <th>NIP</th>
                <th>NAMA PEGAWAI</th>
                <th>KODE/NAMA BARANG</th>
                <th>MERK</th>
                <th>NUP</th>
            </tr>
        </thead>
        <tbody>
            @foreach ($dat['items'] as $item)
                <tr>
                    <td>{{ $item['nip'] }}</td>
                    <td>{{ $item['nama'] }}</td>
                    <td>{{ $item['kode_barang'] }} - {{ $item['nama_barang'] }}</td>
                    <td>{{ $item['keterangan'] }}</td>
                    <td>{{ $item['nup'] }}</td>
                </tr>
            @endforeach
        </tbody>
    </table>
    @endforeach
</body>
</html>
