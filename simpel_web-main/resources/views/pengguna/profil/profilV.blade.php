@extends('layout.main')

@section('content')
    @include('components.breadcums', $breadcums)

    <div class="row">
        <div class="col-lg-12">
            <div class="card mb-4">
                <div class="card-header">
                    <h5 class="mb-0">Profil Saya</h5>
                </div>
                <div class="card-body p-4">
                    <div class="row">
                        <!-- Kolom Data Profil -->
                        <div class="col-lg-6">
                            <table class="table table-borderless mb-0">
                                <tbody>
                                    @foreach ([
                                        ['label'=>'NIP','field'=>'username'],
                                        ['label'=>'Nama','field'=>'name'],
                                        ['label'=>'Pangkat','field'=>'pangkat'],
                                        ['label'=>'Jabatan','field'=>'jabatan'],
                                        ['label'=>'Satker','field'=>'inst_nama'],
                                        ['label'=>'Email','field'=>'email'],
                                    ] as $item)
                                        <tr>
                                            <th width="25%">{{ $item['label'] }}</th>
                                            <td width="5%">:</td>
                                            <td>{{ $model[$item['field']] ?? '-' }}</td>
                                        </tr>
                                    @endforeach
                                    <tr>
                                        <th>Password</th>
                                        <td>:</td>
                                        <td>
                                            @include('components.changePasswordModal', ['user_id'=>$model['id']])
                                            <button class="btn btn-secondary" data-bs-toggle="modal"
                                                data-bs-target="#changePasswordModal">
                                                Ubah Password
                                            </button>
                                        </td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>

                        <!-- Kolom Foto -->
                        <div class="col-lg-6 text-center">
                            <img src="{{ asset(session('userData.foto','assets/images/no-pic.jpg')) }}"
                                 class="rounded-circle avatar-xl img-thumbnail" alt="Foto Profil">
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Card 2FA -->
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <h5 class="mb-0">Two-Factor Authentication (2FA)</h5>
                </div>
                <div class="card-body p-4">
                    @if(session('status'))
                        <div class="alert alert-success">{{ session('status') }}</div>
                    @endif

                    @if(! auth()->user()->google2fa_secret)
                        <p>2FA saat ini <strong>non-aktif</strong>. Aktifkan untuk menambah keamanan akun Anda.</p>
                        <a href="{{ route('2fa.setup') }}" class="btn btn-warning">Setup 2FA</a>
                    @else
                        <p>2FA saat ini <strong>aktif</strong>. Jika ingin mematikan, klik tombol di bawah.</p>
                        <form method="POST" action="{{ route('2fa.disable') }}">
                            @csrf
                            <button type="submit" class="btn btn-danger">
                                Nonaktifkan 2FA
                            </button>
                        </form>
                    @endif
                </div>
            </div>
        </div>
    </div>
@endsection
