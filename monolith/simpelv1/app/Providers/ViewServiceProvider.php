<?php

namespace App\Providers;

use App\Helpers\MyHelper;
use App\Models\Master;
use App\Models\Notifikasi;
use App\Models\Pengguna\Pengguna;
use Illuminate\Support\Facades;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\ServiceProvider;
use Illuminate\View\View;

class ViewServiceProvider extends ServiceProvider
{
    /**
     * Register services.
     */
    public function register(): void
    {
        //
    }

    /**
     * Bootstrap services.
     */
    public function boot(): void
    {
        //
        Facades\View::composer('*', function (View $view) {
            $ms = new Master;
            $userData = \Session::get('userData');
            if (isset($_GET['active_route'])) {
                \Session::put('activeRoute', $_GET['active_route']);
            }

            $changeRole = $_GET['changeRole'] ?? null;
            // dd($changeRole);
            if ($changeRole) {
                $userRole = DB::table('user_role')->where(['user_id' => $userData['id'],  'ms_role_id' => $changeRole])->first();
                $role = Pengguna::isUserHasRole($userRole->id);
                if ($role) {
                    $role->satker_level = MyHelper::getSatkerLevel($role->ms_satker_id);
                    session()->put('userData.current_role', (array) $role);
                }
            }

            if (isset($_GET['notifId'])) {
                Notifikasi::where(['id' => $_GET['notifId']])->update(['is_read' => 1]);
            }

            if ($userData && getenv('APP_ENV') == 'development') {
                $view->with('userChangers', 1);
            }

            $unreadNotif = Notifikasi::getUnread();
            $menus = $ms->getMenus($userData['current_role']['ms_role_id'] ?? null);

            $logo = DB::table('ms_setting_qr as lg')->where('lg.id', '=', 1)->first();
            // print_r($logo);exit;

            $view->with('userData', \Session::get('userData'));
            $view->with('menus', $menus);
            $view->with('unreadNotif', $unreadNotif);
            $view->with('logo', $logo->logo_aplikasi);
            $view->with('logo_dark', $logo->logo_aplikasi_dark);
            $view->with('logo_kecil', $logo->logo_kecil);
            $view->with('nama_aplikasi', $logo->nama_aplikasi);
            $view->with('copyright', $logo->copyright);
        });
    }
}
