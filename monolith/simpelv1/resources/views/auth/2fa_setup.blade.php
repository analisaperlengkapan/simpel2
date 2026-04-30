@extends('layout.main')

@section('content')
    <div class="container mt-5">
        <div class="row justify-content-center">
            <div class="col-md-8">
                <div class="card">
                    <div class="card-header">
                        <h4 class="mb-0">
                            <i class="fas fa-shield-alt me-2"></i>
                            Setup Google Authenticator
                        </h4>
                    </div>
                    <div class="card-body">
                        <div class="alert alert-info">
                            <i class="fas fa-info-circle me-2"></i>
                            <strong>Langkah-langkah setup 2FA:</strong>
                            <ol class="mb-0 mt-2">
                                <li>Download aplikasi Google Authenticator di smartphone Anda</li>
                                <li>Scan QR code di bawah ini dengan aplikasi Google Authenticator</li>
                                <li>Masukkan kode 6 digit yang muncul di aplikasi</li>
                                <li>Klik tombol "Verifikasi dan Aktifkan 2FA"</li>
                            </ol>
                        </div>

                        <div class="text-center mb-4">
                            <h6 class="mb-3">Scan QR Code berikut:</h6>
                            <div class="qr-container p-3 border rounded bg-light">
                                {!! $QR_Image !!}
                            </div>
                            <small class="text-muted mt-2 d-block">
                                Jika QR code tidak muncul, silakan refresh halaman
                            </small>
                        </div>

                        <form method="POST" action="{{ route('2fa.setup.post') }}">
                            @csrf
                            <div class="mb-3">
                                <label for="otp" class="form-label">
                                    <i class="fas fa-key me-2"></i>
                                    Masukkan kode dari Google Authenticator (6 digit):
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
                                <button class="btn btn-primary btn-lg" type="submit">
                                    <i class="fas fa-check me-2"></i>
                                    Verifikasi dan Aktifkan 2FA
                                </button>
                                <a href="/dashboard" class="btn btn-outline-secondary">
                                    <i class="fas fa-arrow-left me-2"></i>
                                    Kembali ke Dashboard
                                </a>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <style>
        .qr-container {
            display: inline-block;
            background: white;
            padding: 20px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        .qr-container img {
            max-width: 200px;
            height: auto;
        }
        .form-control-lg {
            font-size: 1.25rem;
            letter-spacing: 0.5rem;
            font-weight: bold;
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
