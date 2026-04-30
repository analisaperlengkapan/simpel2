@extends('layout.auth')

@section('content')
    <div class="container mt-5">
        <div class="row justify-content-center">
            <div class="col-md-6">
                <div class="card">
                    <div class="card-header text-center">
                        <h4 class="mb-0">
                            <i class="fas fa-shield-alt me-2"></i>
                            Verifikasi Two-Factor Authentication
                        </h4>
                    </div>
                    <div class="card-body">
                        <div class="alert alert-info">
                            <i class="fas fa-info-circle me-2"></i>
                            <strong>Verifikasi Keamanan</strong><br>
                            Masukkan 6 digit kode dari aplikasi Google Authenticator untuk melanjutkan login.
                        </div>

                        <form method="POST" action="{{ route('2fa.verify') }}">
                            @csrf
                            <div class="mb-4">
                                <label for="otp" class="form-label">
                                    <i class="fas fa-key me-2"></i>
                                    Kode OTP (6 digit):
                                </label>
                                <input
                                    type="text"
                                    name="otp"
                                    id="otp"
                                    class="form-control form-control-lg text-center @error('otp') is-invalid @enderror"
                                    placeholder="123456"
                                    required
                                    autofocus
                                    maxlength="6"
                                    pattern="[0-9]{6}"
                                    inputmode="numeric"
                                >
                                @error('otp')
                                    <div class="invalid-feedback">{{ $message }}</div>
                                @enderror
                            </div>

                            <div class="d-grid gap-2">
                                <button class="btn btn-success btn-lg" type="submit">
                                    <i class="fas fa-check me-2"></i>
                                    Verifikasi dan Lanjutkan
                                </button>
                                <a href="/auth/login" class="btn btn-outline-secondary">
                                    <i class="fas fa-arrow-left me-2"></i>
                                    Kembali ke Login
                                </a>
                            </div>
                        </form>

                        <div class="mt-4 text-center">
                            <small class="text-muted">
                                <i class="fas fa-question-circle me-1"></i>
                                Tidak memiliki aplikasi Google Authenticator? 
                                <a href="https://play.google.com/store/apps/details?id=com.google.android.apps.authenticator2" target="_blank">
                                    Download di sini
                                </a>
                            </small>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <style>
        .form-control-lg {
            font-size: 1.25rem;
            letter-spacing: 0.5rem;
            font-weight: bold;
        }
        .card {
            border-radius: 15px;
            box-shadow: 0 4px 20px rgba(0,0,0,0.1);
        }
        .card-header {
            background: linear-gradient(135deg, #1e5631, #2d7d4a);
            color: white;
            border-radius: 15px 15px 0 0 !important;
        }
    </style>

    <script>
        // Auto-focus pada input OTP
        document.addEventListener('DOMContentLoaded', function() {
            const otpInput = document.getElementById('otp');
            if (otpInput) {
                otpInput.focus();
                
                // Auto-format input (hanya angka, maksimal 6 digit)
                otpInput.addEventListener('input', function(e) {
                    let value = e.target.value.replace(/\D/g, '');
                    if (value.length > 6) {
                        value = value.substring(0, 6);
                    }
                    e.target.value = value;
                });
            }
        });
    </script>
@endsection
