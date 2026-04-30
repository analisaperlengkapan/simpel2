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

        // Return logout response with event marker so client-side
        // listeners can broadcast the logout to other tabs via localStorage.
        return response()->redirectTo('/auth/login')
            ->header('X-Logout-Event', '1');
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

    /**
     * OAuth Callback Handler dari Portal
     * 
     * Menerima JWT token dari Portal, validate via Authenc gRPC,
     * kemudian create Laravel session
     */
    public function oauthCallback(Request $request)
    {
        try {
            // Prefer POST body / Authorization header over the query
            // string for the JWT itself: query parameters get persisted
            // in nginx access logs, browser history, and Referer headers.
            // We still fall back to `?token=` for backwards compatibility
            // with the existing Portal redirect flow, but new integrations
            // should POST the token.
            $token = $request->input('token')
                ?? $request->bearerToken()
                ?? $request->query('token');

            if (!$token) {
                return redirect()->route('login')
                    ->with('error', 'Token tidak ditemukan');
            }

            // Verify the OAuth `state` parameter against the value
            // stashed in session by TokenToOAuthMiddleware. This blocks
            // login-CSRF / session-fixation attacks where an attacker
            // crafts `/auth/oauth-callback?token={attacker_jwt}` and
            // tricks a victim into clicking it: the victim's session
            // either has no `oauth_state` (they never started a flow)
            // or has a different one (they started a different flow),
            // so the callback refuses to log them in.
            $expectedState = $request->session()->pull('oauth_state');
            $providedState = $request->input('state') ?? $request->query('state');
            if (!$expectedState || !$providedState || !hash_equals((string) $expectedState, (string) $providedState)) {
                Log::warning('OAuth callback rejected: invalid state', [
                    'ip'        => $request->ip(),
                    'user_agent' => $request->userAgent(),
                    'has_expected' => (bool) $expectedState,
                    'has_provided' => (bool) $providedState,
                ]);
                return redirect()->route('login')
                    ->with('error', 'Sesi OAuth tidak valid, silakan login ulang');
            }

            // Validasi JWT token via Authenc gRPC
            // TODO: Implement gRPC call ke Authenc::verify($token)
            // Untuk sekarang, decode JWT secara manual (production harus via gRPC)
            $claims = $this->verifyJwtToken($token);

            if (!$claims) {
                return redirect()->route('login')
                    ->with('error', 'Token tidak valid');
            }

            // Cari atau buat user di database
            $username = $claims['sub'] ?? null;
            if (!$username) {
                return redirect()->route('login')
                    ->with('error', 'Username tidak ditemukan di token');
            }

            $user = Pengguna::where('username', $username)->first();

            if (!$user) {
                // Auto-create user dari token jika belum ada
                $pegawai = Master::getPegawaiByNip($username);
                if (!$pegawai) {
                    return redirect()->route('login')
                        ->with('error', 'User tidak ditemukan di sistem');
                }

                $data = [
                    'username' => $username,
                    'name' => $claims['name'] ?? $pegawai->nama,
                    'email' => $pegawai->pns_mail,
                    'pangkat' => $pegawai->pangkat ?? 'N/A',
                    'jabatan' => $pegawai->jabatan ?? 'N/A',
                    'ms_satker_id' => $pegawai->inst_satkerkd,
                    'satker' => $pegawai->satker,
                    'ms_satker_pusat_id' => $pegawai->mapped_unit_kerja,
                    'foto' => MyHelper::getFotoMysimkari($pegawai->foto),
                    'password' => Hash::make(Str::random(32)),
                ];

                // Wrap user creation + role assignment in a transaction so we
                // never end up with an orphaned user without a role (which
                // would let subsequent OAuth attempts log them in directly,
                // permanently bypassing role assignment).
                DB::beginTransaction();
                try {
                    $user = Pengguna::create($data);

                    // Assign default role
                    $satker = ModelsMsSatker::where('inst_satkerkd', $pegawai->inst_satkerkd)->first();
                    (new Level())->delInsertUserRole($user->id, [
                        'ms_role_id' => config('constants.pelaksana_satker_role_id'),
                        'ms_satker_id' => $pegawai->inst_satkerkd,
                        'ms_satker_pusat_id' => $pegawai->mapped_unit_kerja,
                        'ms_satker_id_keu' => $satker->kdsatker_keu ?? null,
                        'user_id' => $user->id,
                    ]);
                    DB::commit();
                } catch (\Throwable $e) {
                    DB::rollBack();
                    throw $e;
                }
            }

            // Buat session
            Auth::login($user);

            // Log activity (catat aktivitas login OAuth sebelum kemungkinan
            // redirect ke 2FA, agar selalu tercatat)
            Aktifitas::create([
                'username' => $user->username,
                'operation' => 'LOGIN_OAUTH',
                'table' => 'users',
                'ms_satker_id' => $user->ms_satker_id ?? null,
                'ms_satker_pusat_id' => $user->ms_satker_pusat_id ?? null,
                'pkey' => $user->id,
                'keterangan' => 'Login via Portal OAuth',
                'ip_address' => $request->ip(),
                'user_agent' => $request->userAgent(),
            ]);

            // Enforce 2FA jika user mengaktifkannya (parity dengan flow
            // `login()` di atas). Tanpa ini, OAuth callback bisa menjadi
            // jalan pintas yang melewati 2FA — sekaligus membuat user
            // dengan 2FA aktif terjebak di middleware Ensure2FAIsVerified
            // tanpa session keys yang dibutuhkan TwoFAController.
            if (! empty($user->google2fa_secret)) {
                session([
                    '2fa:user:id' => $user->id,
                    '2fa:remember' => false,
                    '2fa:pending' => true,
                    'auth_token' => $token,
                ]);
                Auth::logout();
                return redirect()->route('2fa.index');
            }

            $userInfo = Pengguna::setUserdata($user);
            $request->session()->regenerate();
            $request->session()->put('userData', $userInfo);
            $request->session()->put('auth_token', $token);

            return redirect()->intended('/dashboard');

        } catch (\Throwable $th) {
            Log::error('OAuth callback error', [
                'error' => $th->getMessage(),
                'trace' => $th->getTraceAsString(),
            ]);

            return redirect()->route('login')
                ->with('error', 'Terjadi kesalahan saat processing token');
        }
    }

    /**
     * Temporary JWT verification (should be replaced with gRPC call to Authenc)
     */
    private function verifyJwtToken(string $token): ?array
    {
        try {
            // In production, this should call Authenc gRPC service
            // For now, using JWT facade for basic verification
            $claims = JWTAuth::getPayload($token);
            return $claims ? $claims->toArray() : null;
        } catch (\Throwable $th) {
            Log::warning('JWT token verification failed', [
                'error' => $th->getMessage(),
            ]);
            return null;
        }
    }
}
