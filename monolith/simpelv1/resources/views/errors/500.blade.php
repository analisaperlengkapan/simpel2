@extends('layout.error')
@section('content')
    <div class="mt-n4">
        <h1 class="display-1 fw-medium">500</h1>
        <h3 class="text-muted">Ups Terjadi Kesalahan, Hubungi Admin!</h3>

        <a href="{{ url('/') }}" class="btn btn-success"><i class="mdi mdi-home me-1"></i>Back to
            home</a>
    </div>
@endsection
