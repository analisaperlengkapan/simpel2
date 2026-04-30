@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
<div class="row">
    <div class="col-lg-8">
        <div class="card">
            <div class="card-header">
                <h5 class="card-title mb-0">Detail Log API</h5>
            </div>
            <div class="card-body">
                <dl class="row mb-0">
                    <dt class="col-sm-4">Waktu</dt>
                    <dd class="col-sm-8">{{ $log->created_at }}</dd>
                    <dt class="col-sm-4">Aplikasi</dt>
                    <dd class="col-sm-8">{{ $log->application->name ?? '-' }}</dd>
                    <dt class="col-sm-4">Endpoint</dt>
                    <dd class="col-sm-8">{{ $log->endpoint->name ?? '-' }}</dd>
                    <dt class="col-sm-4">Method</dt>
                    <dd class="col-sm-8">{{ $log->request_method }}</dd>
                    <dt class="col-sm-4">Path</dt>
                    <dd class="col-sm-8">{{ $log->request_path }}</dd>
                    <dt class="col-sm-4">Status</dt>
                    <dd class="col-sm-8">{{ $log->status_code }}</dd>
                    <dt class="col-sm-4">IP</dt>
                    <dd class="col-sm-8">{{ $log->requester_ip }}</dd>
                    <dt class="col-sm-4">User Agent</dt>
                    <dd class="col-sm-8">{{ $log->user_agent }}</dd>
                </dl>
                <hr/>
                <div class="mb-3">
                    <label class="form-label">Request Body</label>
                    <pre class="bg-light p-2 border rounded" style="min-height:60px;">{{ $log->request_body }}</pre>
                </div>
                <div class="mb-3">
                    <label class="form-label">Response Body</label>
                    <pre class="bg-light p-2 border rounded" style="min-height:60px;">{{ $log->response_body }}</pre>
                </div>
                <a href="{{ url('/pengaturan/api-log') }}" class="btn btn-outline-primary">Kembali</a>
            </div>
        </div>
    </div>
</div>
@endsection 