@extends('layout.auth')

@section('content')
<style>
    body {
        background: #fff7e0 !important;
        min-height: 100vh;
    }
    .login-card {
        border-radius: 18px;
        box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.10);
        background: #fff;
        border: 2.5px solid #f9b233;
        color: #1e5631;
    }
    .login-logo {
        width: 80px;
        margin-bottom: 10px;
    }
    .login-title {
        font-weight: bold;
        font-size: 1.3rem;
        color: #1e5631;
    }
    .login-subtitle {
        font-size: 1rem;
        color: #f9b233;
        margin-bottom: 18px;
        font-weight: 500;
    }
    .input-group-text {
        background: #fff;
        border: none;
        color: #1e5631;
    }
    .form-control {
        background: #fff;
        color: #1e5631;
        border: 1.5px solid #f9b233;
    }
    .form-control:focus {
        box-shadow: 0 0 0 2px #f9b23355;
        border-color: #1e5631;
        background: #fff;
        color: #1e5631;
    }
    .btn-login {
        background: #1e5631;
        border: none;
        color: #fff;
        font-weight: bold;
        box-shadow: 0 2px 8px 0 #1e563155;
    }
    .btn-login:active, .btn-login:focus {
        background: #f9b233;
        color: #1e5631;
    }
    .spinner-border {
        width: 1.2rem;
        height: 1.2rem;
        margin-right: 8px;
        color: #1e5631;
    }
    .alert-danger {
        background: #fff3cd;
        color: #856404;
        border-color: #ffeeba;
    }
</style>
<div class="d-flex align-items-center justify-content-center" style="min-height: 100vh;">
    <div class="col-11 col-sm-8 col-md-6 col-lg-4">
        <div class="card login-card p-4">
            <div class="text-center">
                <img src="{{ url('/assets/images/logo_kejaksaan.png') }}" class="login-logo" alt="Logo Kejaksaan" />
                <div class="login-title mt-2">SIMPEL KEJAKSAAN RI</div>
                <div class="login-subtitle">Sistem Informasi Manajemen Perlengkapan</div>
            </div>
            @if ($errors->any())
                <div class="alert alert-danger mt-2" role="alert">
                    <strong>{{ $errors->first() }}</strong>
                </div>
            @endif
            <form action="/auth/login" method="POST" class="mt-3" id="loginForm" autocomplete="off">
                @csrf
                <div class="mb-3 input-group">
                    <span class="input-group-text"><i class="ri-user-3-line"></i></span>
                    <input type="text" name="username" class="form-control" id="username"
                        placeholder="Username / NIP" required autofocus>
                </div>
                <div class="mb-3 input-group">
                    <span class="input-group-text"><i class="ri-lock-2-line"></i></span>
                    <input type="password" name="password" class="form-control" id="password-input"
                        placeholder="Password" required>
                    <button class="btn btn-outline-secondary" type="button" id="togglePassword">
                        <i class="ri-eye-off-line" id="toggleIcon"></i>
                    </button>
                </div>
                <input type="hidden" name="g-recaptcha-response" id="g-recaptcha-response">
                <button class="btn btn-login w-100 mt-2" type="submit" id="loginBtn">
                    <span id="loginSpinner" class="spinner-border spinner-border-sm d-none"></span>
                    Login
                </button>
            </form>
        </div>
    </div>
</div>
<script src="https://www.google.com/recaptcha/api.js?render={{ $recaptcha_site_key }}"></script>
<script>
    grecaptcha.ready(function () {
        grecaptcha.execute('{{ $recaptcha_site_key }}', { action: 'login' }).then(function (token) {
            document.getElementById('g-recaptcha-response').value = token;
        });
    });
    document.getElementById('togglePassword').addEventListener('click', function () {
        const pwd = document.getElementById('password-input');
        const icon = document.getElementById('toggleIcon');
        if (pwd.type === 'password') {
            pwd.type = 'text';
            icon.classList.remove('ri-eye-off-line');
            icon.classList.add('ri-eye-line');
        } else {
            pwd.type = 'password';
            icon.classList.remove('ri-eye-line');
            icon.classList.add('ri-eye-off-line');
        }
    });
    document.getElementById('loginForm').addEventListener('submit', function(e) {
        const btn = document.getElementById('loginBtn');
        const spinner = document.getElementById('loginSpinner');
        spinner.classList.remove('d-none');
        btn.setAttribute('disabled', 'disabled');
        const token = document.getElementById('g-recaptcha-response').value;
        if (!token) {
            e.preventDefault();
            spinner.classList.add('d-none');
            btn.removeAttribute('disabled');
            alert('Mohon tunggu captcha selesai dimuat. Coba lagi dalam beberapa detik.');
        }
    });
</script>
@endsection
