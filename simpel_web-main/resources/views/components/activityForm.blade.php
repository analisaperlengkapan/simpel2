<div class="col-lg-12">
    <div class="card">
        <div class="card-header">
            <div class="d-flex align-items-center">
                <div class="flex-grow-1">
                    <h5 class="card-title mb-0">Aktifitas Pengajuan</h5>
                </div>
            </div>
        </div>
        <div class="card-body">
            <table class="table align-middle mb-0 my-dt" id="myActivityTbl">
                <thead class="table-light">
                    <tr>
                        <th scope="row">Waktu</th>
                        <th scope="col">Nama</th>
                        <th scope="col">Jabatan / Pangkat</th>
                        <th scope="col">Role</th>
                        <th scope="col">Aktifitas</th>
                        <th scope="col">Komentar</th>
                    </tr>
                </thead>
                <tbody>
                    @forelse ($aktifitasHistories as $history)
                        <tr class="text-center">
                            <td>{{ MyHelper::dateFormat($history->created_at) }}</td>
                            <td>{{ $history->nama }}</td>
                            <td class="text-left">{{ $history->jabatan }} <br> {{ $history->pangkat }}</td>
                            <td>{{ $history->role }}</td>
                            <td>{{ $history->nama_aktifitas }}</td>
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
@if ($aktifitas->canChange)
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
                            @foreach ($aktifitasOptions as $act)
                                <option value="{{ $act->id }}">{{ $act->nama }}</option>
                            @endforeach
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
    </div>
@endif
<script>
    $(function() {
        $('#myActivityTbl').DataTable({
            pageLength: 5,
            order: []
        });
    })
</script>
