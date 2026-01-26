<?php

namespace App\Http\Controllers;
use Illuminate\Routing\Controller;

use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use App\Models\Pengguna\Pengguna;
use PragmaRX\Google2FAQRCode\Google2FA;

class OtentikasiDuaFaktorController extends Controller
{
    /** Tampilkan QR untuk aktivasi 2FA */
    public function showSetupForm()
    {
        $user = Auth::user();
        if ($user->google2fa_secret) {
            return redirect('/dashboard')->with('status', '2FA sudah aktif.');
        }

        $secret = (new Google2FA())->generateSecretKey();
        session(['2fa:secret' => $secret]);

        $google2fa = new Google2FA();
        $QR_Image = $google2fa->getQRCodeInline(
            config('app.name'),
            $user->email,
            $secret
        );

        return view('auth.2fa_setup', compact('QR_Image'));
    }

    /** Aktifkan 2FA setelah scan dan input OTP */
    public function enable(Request $request)
    {
        $request->validate(['otp' => 'required|digits:6']);

        $user = Auth::user();
        $secretKey = session('2fa:secret');

        if (!$secretKey) {
            return redirect()->route('2fa.setup')
                ->withErrors(['otp' => 'Session kode 2FA hilang.']);
        }

        if (!(new Google2FA())->verifyKey($secretKey, $request->otp)) {
            return back()->withErrors(['otp' => 'Kode OTP salah.']);
        }

        // Simpan secret ke database
        $user->google2fa_secret = $secretKey;
        $user->save();

        // Log aktivitas aktivasi 2FA
        \App\Models\Pengguna\Aktivitas::create([
            'username'    => $user->username,
            'operation'   => 'AKTIVASI_2FA',
            'table'       => 'users',
            'ms_satker_id' => $user->ms_satker_id ?? null,
            'ms_satker_pusat_id' => $user->ms_satker_pusat_id ?? null,
            'pkey'        => $user->id,
            'keterangan'  => 'Aktivasi 2FA',
            'ip_address'  => $request->ip(),
            'user_agent'  => $request->userAgent(),
        ]);

        // Refresh user dan login ulang agar Auth::user() terupdate
        $updatedUser = $user->fresh();
        Auth::login($updatedUser);

        // Perbarui session userData dengan struktur yang konsisten
        $userInfo = Pengguna::setUserdata($updatedUser);
        $request->session()->put('userData', $userInfo);

        // Tandai sesi sudah terverifikasi
        session(['2fa_verified' => true]);

        return redirect('/dashboard')->with('status', '2FA berhasil diaktifkan.');
    }

    /** Tampilkan form verifikasi OTP */
    public function showVerifyForm()
    {
        if (!session('2fa:user:id')) {
            return redirect()->route('login')->withErrors(['otp' => 'Sesi 2FA tidak ditemukan. Silakan login ulang.']);
        }

        return view('auth.2fa_verify');
    }

    /** Proses verifikasi OTP */
    public function verify(Request $request)
    {
        $request->validate(['otp' => 'required|digits:6']);

        $userId = session('2fa:user:id');
        $remember = session('2fa:remember', false);

        if (!$userId) {
            return redirect()->route('login')->withErrors(['otp' => 'Sesi tidak valid. Silakan login ulang.']);
        }

        $user = Pengguna::find($userId);
        if (!$user || !$user->google2fa_secret) {
            return redirect()->route('login')->withErrors(['otp' => 'Akun tidak valid.']);
        }

        if (!(new Google2FA())->verifyKey($user->google2fa_secret, $request->otp)) {
            return back()->withErrors(['otp' => 'Kode OTP salah.']);
        }

        Auth::login($user, $remember);
        session(['2fa_verified' => true]);
        $request->session()->regenerate();

        // Perbarui session userData dengan struktur yang konsisten
        $userInfo = Pengguna::setUserdata($user);
        $request->session()->put('userData', $userInfo);

        session()->forget(['2fa:user:id', '2fa:remember']);

        return redirect()->intended('/dashboard');
    }

    /** Nonaktifkan 2FA */
    public function disable()
    {
        $user = Auth::user();
        $user->google2fa_secret = null;
        $user->save();
        session()->forget('2fa_verified');

        // Log aktivitas nonaktifkan 2FA
        \App\Models\Pengguna\Aktivitas::create([
            'username'    => $user->username,
            'operation'   => 'NONAKTIFKAN_2FA',
            'table'       => 'users',
            'ms_satker_id' => $user->ms_satker_id ?? null,
            'ms_satker_pusat_id' => $user->ms_satker_pusat_id ?? null,
            'pkey'        => $user->id,
            'keterangan'  => 'Nonaktifkan 2FA',
            'ip_address'  => request()->ip(),
            'user_agent'  => request()->userAgent(),
        ]);

        // Perbarui session userData
        $userInfo = Pengguna::setUserdata($user);
        session(['userData' => $userInfo]);

        return redirect()->route('profil')->with('status', '2FA berhasil dinonaktifkan.');
    }

    /** Nonaktifkan 2FA oleh superadmin untuk user lain */
    public function disableBySuperadmin($id, Request $request)
    {
        $admin = Auth::user();
        if (!$admin || empty($admin->is_superadmin) || $admin->is_superadmin != 1) {
            abort(403, 'Hanya superadmin yang boleh menonaktifkan 2FA user lain.');
        }
        $user = \App\Models\Pengguna\Pengguna::find($id);
        if (!$user) {
            return back()->with('error', 'User tidak ditemukan.');
        }
        $user->google2fa_secret = null;
        $user->save();
        // Log aktivitas nonaktifkan 2FA oleh superadmin
        \App\Models\Pengguna\Aktivitas::create([
            'username'    => $admin->username,
            'operation'   => 'NONAKTIFKAN_2FA_USER_LAIN',
            'table'       => 'users',
            'ms_satker_id' => $admin->ms_satker_id ?? null,
            'ms_satker_pusat_id' => $admin->ms_satker_pusat_id ?? null,
            'pkey'        => $user->id,
            'keterangan'  => 'Nonaktifkan 2FA user: '.$user->username,
            'ip_address'  => $request->ip(),
            'user_agent'  => $request->userAgent(),
        ]);
        return back()->with('status', '2FA user berhasil dinonaktifkan oleh superadmin.');
    }
}
