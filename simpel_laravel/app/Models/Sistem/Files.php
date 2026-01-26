<?php

namespace App\Models\Sistem;

// use Illuminate\Contracts\Auth\MustVerifyEmail;

use App\Helpers\MyHelper;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;

class Files extends Model
{
    protected $table = 'files';
    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'kategori',
        'kategori_slug',
        'pkey',
        'filename',
        'path',
        'created_by',
        'created_at',
        'filetype',
    ];
    static function isSingleFileUpload(Request $request, $params)
    {
        $fileKey = $params['fileKey'] ?? null;
        if (!$fileKey)
            return [];
        if (!$request->hasFile($params['fileKey'])) {
            return [];
        }

        if ($request->hasFile($fileKey) && is_array($request->file($fileKey)) === false) {
            return [$request->file($fileKey)];
        } else {
            return $request->file($fileKey);
        }
    }

    /*
     $params = [
        fileKey,
        dir,
        pkey,
        kategori,
     ]
    */

    static function upload(Request $request, $params)
    {

        $fileKey = $params['fileKey'] ?? null;
        $files = self::isSingleFileUpload($request, $params);
        if ($params['isRequired'] ?? false) {
            $validatorFieldName = count($files) > 1 ? "{$fileKey}.*" : "$fileKey";
            $rules[$validatorFieldName] = 'required|mimes:jpeg,png,pdf,jpg|max:50000';
            $msg["{$validatorFieldName}.required"] = $params['kategori'] . ' Harus diisi';
            $request->validate($rules, $msg);
        }
        if (empty($files))
            return null;
        foreach ($files as $file) {
            // $storedPath = $file->storePublicly($params['dir']);
            $storedPath = $file->move('uploads/' . $params['dir'], uniqid() . '.' . $file->getClientOriginalExtension());
            $newFile = [
                'filename' => $file->getClientOriginalName(),
                'path' => $storedPath->getPathname(),
                'pkey' => $params['pkey'],
                'kategori' => $params['kategori'],
                'kategori_slug' => MyHelper::generateSlug($params['kategori']),
                'filetype' => $file->getClientOriginalExtension(),
                'created_by' => session('userData.username'),
                'created_at' => date('Y-m-d H:i:s'),
            ];
            $insertedFiles[] = $newFile;
        }
        self::insert($insertedFiles);
        return count($insertedFiles) > 1 ? $insertedFiles : $insertedFiles[0];
    }

    static function getKategori()
    {
        $query = DB::table('files')->distinct()->select(['kategori', 'kategori_slug'])->get();
        return $query;
    }
    function getDataGrid($paging, $search = [], $isRaw = false)
    {
        $query = DB::table("files as a")->join('users as b', "a.created_by", '=', 'b.username');
        $query->select(["a.*", 'b.name']);
        // dd($query->paginate());
        if (!empty($search)) {
            foreach ($search as $field => $value) {
                $query->where($field, 'like', "%{$value}%");
            }
        }
        $query->orderByDesc('a.created_at');
        $total = $query->count();
        $data = $query->limit($paging['length'] ?? 10)->skip($paging['start'] ?? 0)->get();
        return ['total' => $total, 'data' => $data];
    }
    // select count(*) as unread from vw_notifikasi where ms_satker_id = '10.05' or username = 'superadmin' ;

}
