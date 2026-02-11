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
            text-align: center;
        }
    </style>
</head>
<body>
    <h2 style="text-align: center;">{{strtoupper($model['nama']).' TAHUN '.$model['tahun']}}</h2>
    <br/><br/>
    <h3>Daftar Barang Disetujui</h3>
    <table>
        <thead>
            <tr>
                <th>Satker</th>
                <th>Kode Barang</th>
                <th>Nama Barang</th>
                <th>Jml Pengajuan</th>
                <th>Jml Disetujui</th>
                <th>Alasan Pengadaan</th>
                <th>Prioritas</th>
            </tr>
        </thead>
        <tbody>
            @foreach ($data as $item)
                <tr>
                    <td>{{ $item->satker }}</td>
                    <td>{{ $item->kode_barang }}</td>
                    <td>{{ $item->nm_barang }}</td>
                    <td>{{ $item->jumlah }}</td>
                    <td>{{ $item->jml_setuju }}</td>
                    <td>{{ $item->alasan }}</td>
                    <td>{{ $item->prioritas }}</td>
                </tr>
            @endforeach
        </tbody>
    </table>
    <div style="page-break-before: always;"></div>
    <h3>Daftar Barang Ditolak</h3>
    <table>
        <thead>
            <tr>
                <th>Satker</th>
                <th>Kode Barang</th>
                <th>Nama Barang</th>
                <th>Jml Pengajuan</th>
                <th>Jml Ditolak</th>
                <th>Keterangan</th>
            </tr>
        </thead>
        <tbody>
            @foreach ($data as $item)
                <tr>
                    <td>{{ $item->satker }}</td>
                    <td>{{ $item->kode_barang }}</td>
                    <td>{{ $item->nm_barang }}</td>
                    <td>{{ $item->jumlah }}</td>
                    <td>{{ $item->jml_tolak }}</td>
                    <td>{{ $item->keterangan }}</td>
                </tr>
            @endforeach
        </tbody>
    </table>
</body>
</html>
