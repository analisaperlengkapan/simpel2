<?php

namespace App\Models;

// use Illuminate\Contracts\Auth\MustVerifyEmail;

use App\Models\Pengguna\Pengguna;
use GuzzleHttp\Client;
use GuzzleHttp\Psr7\Request;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\DB;

class Notifikasi extends Model
{
    protected $table = 'notifikasi';

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'dari_username',
        'dari_ms_satker_id',
        'dari_ms_role_id',
        'target_username',
        'target_role_id',
        'isi',
        'judul',
        'url',
        'is_read',
    ];

    protected static function boot()
    {
        parent::boot();

        static::creating(function ($data) {
            // Perform actions after a new post is created.
            // For example, you can log the creation or send notifications.
            // $post is the instance of the created Post model.
            // You can access its attributes like $post->title, $post->content, etc.
        });

        static::created(function ($data) {
            // Perform actions after a new post is created.
            // For example, you can log the creation or send notifications.
            // $post is the instance of the created Post model.
            // You can access its attributes like $post->title, $post->content, etc.
        });
    }

    public static function getUnread()
    {
        $currentRole = session('userData.current_role');
        if (! $currentRole) {
            return 0;
        }
        $query = DB::table('vw_notifikasi')->where(['is_read' => 0, 'target_username' => session('userData.username')])->count();

        return $query;
    }

    public static function getNotifs()
    {
        $currentRole = session('userData.username');

        return DB::table('vw_notifikasi')->where('target_username', session('userData.username'))->orderByDesc('created_at')->limit(10)->get();
    }
    // select count(*) as unread from vw_notifikasi where ms_satker_id = '10.05' or username = 'superadmin' ;

    public static function sendMobileNotif($params)
    {
        $client = new Client;
        $headers = [
            'Authorization' => getenv('FIREBASE_TOKEN'),
            'Content-Type' => 'application/json',
        ];
        // "to" => "dxZI4VqCT_yrHXDyDsMNRI:APA91bFlbLEi_NPGG7llmAb0M0-RskIs-4OAYZfIgLRm0tFdiposuy_sosFEp2vSwF8PEObf4nnGjadZ0t6GXC-OHwNpkgV0wxn68H_ua-ZuNuakcjV8L8bv5c9O6LWV-GJuLV9pG-8c",
        // "data" => [
        //     "title" => "Title Notification",
        //     "message" => "Message Body Notification",
        //     "id" => 12345678
        // ],
        $body = [
            'registration_ids' => $params['tokens'],
            'data' => [
                'title' => $params['title'],
                'message' => $params['isi'],
                'id' => uniqid(),
            ],
            'notification' => [
                'body' => $params['isi'],
                'title' => $params['title'],
            ],
        ];
        $request = new Request('POST', 'https://fcm.googleapis.com/fcm/send', $headers, json_encode($body));
        $client->send($request);
        // echo $res->getBody();
    }

    /**
     *url = Url pas di klik
     *judul
     *isi (optional)
     *target  = role / username,
     *targetValue = klo role ms_role_id nya | atau usernamenya,
     *targetSatker = array optional
     *targetSatkerPusat = array optional
     */
    public static function sendNotif(array $params)
    {
        $currentRole = session('userData.current_role');
        $notifs = [];
        $userTokens = [];

        $baseNotif = [
            'created_at' => date('Y-m-d H:i:s'),
            'dari_ms_role_id' => $currentRole['ms_role_id'],
            'dari_username' => session('userData.username'),
            'dari_ms_satker_id' => $currentRole['ms_satker_id'],
            'dari_ms_satker_pusat_id' => $currentRole['ms_satker_pusat_id'],
            'url' => $params['url'],
            'judul' => $params['judul'],
            'isi' => $params['isi'],
        ];

        if ($params['target'] == 'role') {
            $users = Pengguna::getUserByRole($params['targetValue'], $params['targetSatker'] ?? []);
        } else {
            $users = Pengguna::where('username', '=', $params['targetValue'])->get();
        }

        foreach ($users as $user) {
            $newNotif = $baseNotif;
            $newNotif['target_username'] = $user->username;
            $newNotif['target_role_id'] = $params['targetValue'] ?? null;
            if (! empty($user->firebase_token)) {
                $userTokens[] = $user->firebase_token;
            }
            $notifs[] = $newNotif;
        }

        Notifikasi::insert($notifs);
        if (! empty($userTokens)) {
            self::sendMobileNotif(['title' => $params['judul'], 'isi' => $params['isi'], 'tokens' => $userTokens]);
        }

    }
}
