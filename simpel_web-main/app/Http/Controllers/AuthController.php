<?php

namespace App\Http\Controllers;

use App\Helpers\MyHelper;
use App\Helpers\RecaptchaHelper;
use App\Models\Master;
use App\Models\Master\MsSatker;
use App\Models\MsSatker as ModelsMsSatker;
use App\Models\Pengguna\Level;
use App\Models\Pengguna\Pengguna;
use App\Models\Pengguna\Aktifitas;
use App\Models\Suport\NotifikasiManual;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\RateLimiter;
use Illuminate\Support\Str;
use Tymon\JWTAuth\Facades\JWTAuth;

class AuthController extends Controller
{
    public function index()
    {
        return view('auth.login', [
            'recaptcha_site_key' => config('app.recaptcha_site_key'),
        ]);
    }

    public function login(Request $request)
    {

       // 1) Validasi input + reCAPTCHA jika diaktifkan
        $rules = [
            'username' => 'required|string',
            'password' => 'required|string',
        ];
        if (config('app.recaptcha_enabled', true)) {
            $rules['g-recaptcha-response'] = 'required';
        }
        $messages = [
            'g-recaptcha-response.required' => 'Captcha harus diisi',
        ];
        $request->validate($rules, $messages);

        // 2) Rate Limiting per username + IP
        $username    = (string) $request->username;
        $key         = Str::lower("login|{$username}|{$request->ip()}");
        $maxAttempts = 5;
        $decaySecs   = 60;

        if (RateLimiter::tooManyAttempts($key, $maxAttempts)) {
            $secs = RateLimiter::availableIn($key);
       
            Log::warning('Login diblokir karena terlalu banyak percobaan', [
                'ip'       => $request->ip(),
                'username' => $request->username,
                'time'     => now(),
                'seconds'  => $secs,
            ]);

            return back()->withErrors(["username" => "Terlalu banyak percobaan. Tunggu $secs detik."])
                         ->onlyInput('username');
        }
        RateLimiter::hit($key, $decaySecs);

        // 3) Verifikasi reCAPTCHA jika diaktifkan
        if (config('app.recaptcha_enabled')) {
            $cap = RecaptchaHelper::verify($request->input('g-recaptcha-response'));
            if (! $cap['success'] || $cap['score'] < 0.5) {
                return back()->withErrors(['captcha' => 'Verifikasi reCAPTCHA gagal.'])->onlyInput('username');
            }
        }

        // 4) Cek data pegawai atau insert default password
        $check = $this->checkPegawaiExist($request->username);
        if ($check === 2) {
            return back()->withErrors(['username' => 'NIP/Password Salah!'])->onlyInput('username');
        } elseif ($check === 0) {
            $request->merge(['password' => config('constants.default_password')]);
        }

        // 5) Attempt login
        if (Auth::attempt($request->only('username', 'password'), $request->has('remember'))) {
            RateLimiter::clear($key);
            $user = Auth::user();

            // Catat aktivitas login
            Aktifitas::create([
                'username'    => $user->username,
                'operation'   => 'LOGIN',
                'table'       => 'users',
                'ms_satker_id' => $user->ms_satker_id ?? null,
                'ms_satker_pusat_id' => $user->ms_satker_pusat_id ?? null,
                'pkey'        => $user->id,
                'keterangan'  => 'Login ke sistem',
                'ip_address'  => $request->ip(),
                'user_agent'  => $request->userAgent(),
            ]);

            // 2FA jika aktif
            if (! empty($user->google2fa_secret)) {
                session([
                    '2fa:user:id' => $user->id,
                    '2fa:remember' => $request->has('remember'),
                    '2fa:pending' => true,
                ]);
                Auth::logout();
                return redirect()->route('2fa.index');
            }

            // Simpan session userData
            $userInfo = Pengguna::setUserdata($user);
            $request->session()->regenerate();
            $request->session()->put('userData', $userInfo);

            // Flash notifikasi
            if ($notif = NotifikasiManual::getRoleNotif()) {
                $request->session()->flash('notifMessage', $notif);
            }

            return redirect()->intended('/dashboard');
        }

        // Gagal login
        sleep(1);
        Aktifitas::create([
            'username'    => $request->username,
            'operation'   => 'LOGIN_GAGAL',
            'table'       => 'users',
            'ms_satker_id' => null,
            'ms_satker_pusat_id' => null,
            'pkey'        => null,
            'keterangan'  => 'Login gagal: NIP/Password Salah',
            'ip_address'  => $request->ip(),
            'user_agent'  => $request->userAgent(),
        ]);
        Log::warning('Login gagal', [
            'ip'       => $request->ip(),
            'username' => $request->username,
            'time'     => now(),
        ]);
        return back()->withErrors(['username' => 'NIP/Password Salah!'])->onlyInput('username');
    }

    protected function checkPegawaiExist(string $nip): int
    {
        // 1 = sudah ada, 0 = baru insert (default pwd), 2 = tidak ditemukan
        if (Pengguna::where('username', $nip)->exists()) {
            return 1;
        }
        $pegawai = Master::getPegawaiByNip($nip);
        if (empty($pegawai)) {
            return 2;
        }

        DB::beginTransaction();
        try {
            $data = [
                'username'           => $pegawai->peg_nip_baru,
                'name'               => $pegawai->nama,
                'email'              => $pegawai->pns_mail,
                'pangkat'            => $pegawai->pangkat,
                'jabatan'            => $pegawai->jabatan,
                'ms_satker_id'       => $pegawai->inst_satkerkd,
                'satker'             => $pegawai->satker,
                'ms_satker_pusat_id' => $pegawai->mapped_unit_kerja,
                'foto'               => MyHelper::getFotoMysimkari($pegawai->foto),
                'password'           => config('constants.default_password'),
            ];
            $new = Pengguna::create($data);

            // assign role default
            $satker = ModelsMsSatker::where('inst_satkerkd', $pegawai->inst_satkerkd)->first();
            (new Level())->delInsertUserRole($new->id, [
                'ms_role_id'         => config('constants.pelaksana_satker_role_id'),
                'ms_satker_id'       => $pegawai->inst_satkerkd,
                'ms_satker_pusat_id' => $pegawai->mapped_unit_kerja,
                'ms_satker_id_keu'   => $satker->kdsatker_keu ?? null,
                'user_id'            => $new->id,
            ]);

            DB::commit();
            return 0;
        } catch (\Throwable $e) {
            DB::rollBack();
            return 2;
        }
    }

    public function logout(Request $request)
    {
        $user = Auth::user();
        // Catat aktivitas logout
        if ($user) {
            Aktifitas::create([
                'username'    => $user->username,
                'operation'   => 'LOGOUT',
                'table'       => 'users',
                'ms_satker_id' => $user->ms_satker_id ?? null,
                'ms_satker_pusat_id' => $user->ms_satker_pusat_id ?? null,
                'pkey'        => $user->id,
                'keterangan'  => 'Logout dari sistem',
                'ip_address'  => $request->ip(),
                'user_agent'  => $request->userAgent(),
            ]);
        }
        Auth::logout();
        $request->session()->invalidate();
        $request->session()->regenerateToken();
        return redirect('/auth/login');
    }

    public function changeRole(int $roleId)
    {
        $role = Pengguna::isUserHasRole($roleId);
        if (!$role) {
            return $this->resError('Tidak Memiliki Akses');
        }

        $role->satker_level = MyHelper::getSatkerLevel($role->ms_satker_id);
        session()->put('userData.current_role', (array) $role);
        return $this->resSuccess();
    }

    public function changeRoleMobile(int $roleId)
    {
        $role = Pengguna::isUserHasRole($roleId);
        if (!$role) {
            return response()->json([
                'status' => 'error',
                'message' => 'Unauthorized',
            ], 401);
        }

        $role->satker_level = MyHelper::getSatkerLevel($role->ms_satker_id);
        $master = new Master();
        $menus = $master->getMenus($role->ms_role_id);
        return response()->json([
            'status' => 'success',
                'role' => $role,
                'menu' => $menus,
        ], 200);
    }

    public function changePassword(Request $request)
    {
        $request->validate([
            'password_old' => 'required',
            'password_new' => ['required', 'min:6'],
            'user_id' => ['required'],
        ]);

        $user = DB::table('users')->where(['id' => $request->input('user_id')])->first();
        if (!$user) {
            return $this->resError('User Tidak ditemukan');
        }

        if (!Hash::check($request->input('password_old'), $user->password)) {
            return $this->resError('Password Salah');
        }

        $password = Hash::make($request->input('password_new'));
        Pengguna::where(['id' => $request->input('user_id')])->update(['password' => $password]);

        // Log aktivitas ganti password
        \App\Models\Pengguna\Aktifitas::create([
            'username'    => $user->username,
            'operation'   => 'GANTI_PASSWORD',
            'table'       => 'users',
            'ms_satker_id' => $user->ms_satker_id ?? null,
            'ms_satker_pusat_id' => $user->ms_satker_pusat_id ?? null,
            'pkey'        => $user->id,
            'keterangan'  => 'Ganti password',
            'ip_address'  => $request->ip(),
            'user_agent'  => $request->userAgent(),
        ]);

        return $this->resSuccess('Berhasil merubah Password', ['type' => 'redirect', 'url' => url('pengguna/profil')]);
    }

    public function resetPassword(Request $request)
    {
        try {
            $user = Pengguna::where('username', $request->input('nip'))->first();
            if (!$user) {
                return $this->resError('User tidak ditemukan');
            }

            $password = Hash::make(config('constants.default_password'));
            $user->update(['password' => $password]);

            // Log aktivitas reset password
            \App\Models\Pengguna\Aktifitas::create([
                'username'    => $user->username,
                'operation'   => 'RESET_PASSWORD',
                'table'       => 'users',
                'ms_satker_id' => $user->ms_satker_id ?? null,
                'ms_satker_pusat_id' => $user->ms_satker_pusat_id ?? null,
                'pkey'        => $user->id,
                'keterangan'  => 'Reset password oleh admin',
                'ip_address'  => $request->ip(),
                'user_agent'  => $request->userAgent(),
            ]);

            return $this->resSuccess('Berhasil mereset Password', ['type' => 'redirect', 'url' => url('pengguna/profil')]);
        } catch (\Throwable $th) {
            return $this->resError('Gagal Reset password');
        }
    }

    public function loginApi(Request $request)
    {
        $request->validate([
            'username' => 'required|string',
            'password' => 'required|string',
        ]);
        $credentials = $request->only('username', 'password');
        if (!$token = JWTAuth::attempt($credentials)) {
            return response()->json([
                'status' => 'error',
                'message' => 'Unauthorized',
            ], 401);
        }

        $user = JWTAuth::user();

        if ($request->has('firebase_token')) {
            Pengguna::where(['username' => $credentials['username']])->update(['firebase_token' => $request->input('firebase_token')]);
        }

        $userData = Pengguna::setUserdata($user);
        $master = new Master();
        $menus = $master->getMenus($userData['current_role']['ms_role_id']);

        return response()->json([
            'status' => 'success',
            'user' => $userData,
            'menu' => $menus,
            'authorisation' => [
                'token' => $token,
                'type' => 'bearer',
            ]
        ]);
    }

    public function logoutApi()
    {
        $user = JWTAuth::user();
        Pengguna::where(['id' => $user->id])->update(['firebase_token' => null]);
        auth()->guard('api')->logout();

        return response()->json(['message' => 'Logged out successfully']);
    }

    public function getAuthenticatedUser()
    {
        return response()->json(session('userData'));
    }

    public function changeUser(Request $request)
    {
        $selected = Pengguna::where('username', $request->input('username'))->first();
        $userInfo = Pengguna::setUserdata($selected);
        $request->session()->put('userData', $userInfo);
        return $this->resSuccess();
    }
}
