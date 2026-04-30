@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
<form action="/suport/helpdesk/saveKomentar" method="POST" class="ajaxForm" enctype="multipart/form-data"> 
 <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">{{ $kategoriJudul }} Bantuan</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form action="/suport/helpdesk" method="POST" class="ajaxForm" enctype="multipart/form-data">
                        @csrf

                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        <div class="row">
                            <div class="table-responsive">
                                <table class="table table-borderless mb-0">
                                    <tbody>
                                        <tr>
                                            <th class="ps-0" width="20%" scope="row">Judul</th>
                                            <td width="5%">:</td>
                                            <td class="">
                                                {{ $model['judul'] ?? '-' }}
                                            </td>
                                        </tr>
                                        <tr>
                                            <th class="ps-0" width="20%" scope="row">Topik/Kategori</th>
                                            <td width="5%">:</td>
                                            <td class="">
                                                {{ $model['topik'] ?? '-' }}
                                            </td>
                                        </tr>
                                        <tr>
                                            <th class="ps-0" width="20%" scope="row">Deskripsi</th>
                                            <td width="5%">:</td>
                                            <td class="">
                                                {{ $model['deskripsi'] ?? '-' }}
                                            </td>
                                        </tr>
                                        <tr>
                                            <th class="ps-0" width="20%" scope="row">Tanggal Pengajuan</th>
                                            <td width="5%">:</td>
                                            <td class="">
                                                {{ $model['tgl_pengajuan'] ?? '-' }}
                                            </td>
                                        </tr>
                                    </tbody>
                                </table>
                            </div>
                        </div>						                   
                </div>
            </div>
        </div>
		<div class="col-lg-12">
			<div class="card">
				<div class="card-header">
					<div class="d-flex align-items-center">
						<div class="flex-grow-1">
							<h5 class="card-title mb-0">Aktifitas Komentar/Catatan</h5>
						</div>
					</div>
				</div>
				<div class="card-body">
					<table class="table align-middle mb-0 my-dt">
						<thead class="table-light">
							<tr>
								<th scope="row">Waktu</th>
								<th scope="col">Nama</th>
								<th scope="col">Jabatan / Pangkat</th>
								<th scope="col">Role</th>
								<th scope="col">Komentar</th>
							</tr>
						</thead>
						<tbody>
							@forelse ($aktifitasHistories as $history)
                                    <tr class="text-center">
                                        <td>{{ MyHelper::dateFormat($history->created_at) }}</td>
                                        <td>{{ $history->nama_user }}</td>
                                        <td class="text-left">{{ $history->jabatan }} <br> {{ $history->pangkat }}</td>
                                        <td>{{ $history->role }}</td>
                                        <td>{{ $history->komentar }}</td>
                                    </tr>
                                @empty
                                    <tr>
                                        <td colspan="6" class="text-center">Belum ada Data</td>
                                    </tr>
                                @endforelse
						</tbody>
					</table>
				</div>
			</div>
		</div>
		<div class="col-lg-12">
			<div class="card">
				<div class="card-header">
					<div class="d-flex align-items-center">
						<div class="flex-grow-1">
							<h5 class="card-title mb-0">Aksi</h5>
						</div>
					</div>
				</div>
				<div class="card-body p-4">
					<div class="row">
						<div class="mb-3 col-lg-6">
							<select class="form-control" data-choices data-choices-search name="ms_aktifitas_id">
								<option value="Close">Tutup Komentar (Close)</option>
								<option value="Open">Open Komentar (Open)</option>
							</select>
						</div>
					</div>

					<div class="row">
						<div class="mb-3 col-lg-6">
							<textarea name="komentar" class="form-control" rows="5" placeholder="Tuliskan Komentar"></textarea>
						</div>
					</div>

				</div>
			</div>
			<div class="col-lg-12 mb-4">
                <div class="hstack gap-2 justify-content-left">
                    <a href="{{ url('suport/helpdesk') }}" class="btn btn-outline-primary">Kembali</a>
                        <button type="submit" class="btn btn-primary">
                            Simpan
                        </button>
                </div>
            </div>
		</div>
    </div>
</form>
@endsection

@section('js')
    <script>
        
    </script>
@endsection
