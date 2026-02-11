@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
<div class="row">
    <div class="col-lg-12">
        <div class="card ">
            <div class="card-header">
                <div class="d-flex align-items-center">
                    <div class="flex-grow-1">
                        <h5 class="card-title mb-0">{{ $isNew ? 'Tambah' : 'Edit' }} User SIMPLE</h5>
                    </div>
                </div>
            </div>
            <div class="card-body p-4">
                <form action="{{$controller}}" method="POST" class="ajaxForm">
                    @csrf
                    @if (!$isNew)
                    <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                    @endif
                    <div class="row">
                        <div class="col-lg-6">
                            <div class="mb-3">
                                <label for="username" class="form-label">Username</label>
                                <input class="form-control" id="username" name="username" placeholder="Username" required value="{{ $model['username'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class=" col-lg-6">
                            <div class="mb-3">
                                <label for="name" class="form-label">Nama</label>
                                <input type="text" class="form-control" id="name" name="name" required value="{{ $model['name'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class=" col-lg-6">
                            <div class="mb-3">
                                <label for="email" class="form-label">Email</label>
                                <input type="email" class="form-control" id="email" name="email" required value="{{ $model['email'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    @if ($isNew)
                    <div class="row">
                        <div class=" col-lg-6">
                            <div class="mb-3">
                                <label for="password" class="form-label">Password</label>
                                <input type="password" class="form-control" id="password" name="password" required placeholder="Password" value="{{ $model['password'] ?? '' }}">
                            </div>
                        </div>
                    </div>
                    @endif
                    @if($canChangePassword)
                    @include('components.changePasswordModal', ['user_id'=>session('id')])
                    <button type="button" class="btn btn-secondary" data-bs-toggle="modal" data-bs-target="#changePasswordModal">Ubah Password</button>
                    @endif
                    <div class="row">
                        <div class="col-lg-12">
                            <div class="hstack gap-2 justify-content-end">
                                <a href="{{ url($controller) }}" class="btn btn-outline-primary">Kembali</a>
                                <button type="submit" class="btn btn-primary">
                                    {{ $isNew ? 'Simpan' : 'Ubah' }}
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
    $(function() {

    })
</script>
@endsection