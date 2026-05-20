<?php

namespace App\Helpers;

use Illuminate\Support\Facades\DB;
use Mccarlosen\LaravelMpdf\Facades\LaravelMpdf;
use SimpleSoftwareIO\QrCode\Facades\QrCode;

class MyHelper
{
    public static function generateSelectOptions(array $arr)
    {
        $opts = '';

        $value = $arr['value'] ?? null;
        $text = $arr['text'] ?? null;
        $default = $arr['selected'] ?? null;
        $type = $arr['type'] ?? null;
        $textkode = $arr['textkode'] ?? null;
        $disableds = $arr['disableds'] ?? [];

        if ($type == 'raw') {
            $opts = [];
            if ($value) {
                foreach ($arr['data'] as $key => $dt) {
                    $opt = ['id' => $dt->{$value}, 'text' => $dt->{$text}];
                    if (in_array($opt['id'], $disableds)) {
                        $opt['disabled'] = true;
                    }
                    $opts[] = $opt;
                }
            } else {
                foreach ($arr['data'] as $key => $dt) {
                    $opt = ['id' => $dt, 'text' => $dt];
                    if (in_array($opt['id'], $disableds)) {
                        $opt['disabled'] = true;
                    }
                    $opts[] = $opt;
                }
            }
        } else {
            if ($value) {
                foreach ($arr['data'] as $key => $dt) {
                    if (is_array($default)) {
                        $selected = in_array($dt->{$value}, $default) ? 'selected' : '';
                    } else {
                        $selected = $dt->{$value} == $default ? 'selected' : '';
                    }
                    if ($textkode) {
                        $opts .= "<option {$selected} value='{$dt->{$value}}'>{$dt->{$textkode}} - {$dt->{$text}}</option>";
                    } else {
                        $opts .= "<option {$selected} value='{$dt->{$value}}'>{$dt->{$text}}</option>";
                    }
                }
            } else {
                foreach ($arr['data'] as $key => $dt) {
                    if (is_array($default)) {
                        $selected = in_array($dt, $default) ? 'selected' : '';
                    } else {
                        $selected = $dt == $default ? 'selected' : '';
                    }
                    $opts .= "<option {$selected} value='{$dt}'>{$dt}</option>";
                }
            }
        }

        return $opts;
    }

    public static function getPk($yearmonthdate, $table)
    {
        $query = DB::selectOne("select nextval('".$table."') as next_value");
        $hasil = $query->next_value;

        return $yearmonthdate.str_pad($hasil, 3, '0', STR_PAD_LEFT);
    }

    public static function toIdSatkerCms($kdSatker)
    {
        if ($kdSatker == '00') {
            $id_kejati = '00';
            $id_kejari = '00';
            $id_cabjari = '00';
        } else {
            $exp = explode('.', $kdSatker);
            $id_kejati = $exp[0] ?? null;
            if (! $id_kejati) {
                return ['id_kejati' => null, 'id_kejari' => null, 'id_cabjari' => null];
            }
            $id_kejari = $exp[1] ?? '00';
            $id_cabjari = $exp[2] ?? '00';
        }

        return ['id_kejati' => $id_kejati, 'id_kejari' => $id_kejari, 'id_cabjari' => $id_cabjari];
    }

    public static function generateQrCode($data)
    {
        $logoPath = public_path('assets/images/logo_kejaksaan.jpg');
        $qrCode = QrCode::format('png')
            ->size(150)
            ->errorCorrection('M')
            ->generate($data);

        return $qrCode;
    }

    public static function generateLabelBankAsset($model)
    {
        $data = 'Kode Barang : '.$model['kode_barang']."\n".'Nama Barang : '.$model['nm_barang']."\n".'Nama Satker : '.$model['nm_satker'];
        $qrCodeImage = self::generateQrCode($data);
        $logoPath = public_path('assets/images/web-bg.png');
        $pdfContent = '<html><style> table.myFormat tr td { font-size: 8px; } table { margin-left: auto; margin-right: auto;}, table, th, td {border: 1px solid black;border-collapse: collapse;} </style><body>';
        $pdfContent .= '<table class="myFormat" style="border: 1px solid black;border-collapse: collapse;" width="100%"><tbody>';
        $pdfContent .= '<tr>';
        $pdfContent .= '<td width="20%"><center><img src="'.$logoPath.'" width="30"/></center></td>';
        $pdfContent .= '<td colspan="2"><center><b>'.$model['nm_satker'].'<br/><br/><br/>'.$model['kdsatker_keu'].'</center></b></td>';
        $pdfContent .= '</tr>';
        $pdfContent .= '<tr><td colspan="2">';
        $pdfContent .= $model['kode_barang'].'<br/>'.$model['nm_barang'].'<br/>'.($model['nup'] ? 'NUP : '.$model['nup'] : '').'<br/><br/>'.($model['merk'] ? 'Merk : '.$model['merk'] : '');
        $pdfContent .= '</td>';
        $pdfContent .= '<td width="50%">';
        $pdfContent .= '<center><img src="data:image/png;base64,'.base64_encode($qrCodeImage).'" width="100"/></center>';
        $pdfContent .= '</td></tr>';
        $pdfContent .= '</tbody></table>';
        $pdfContent .= '</body></html>';
        $pdf = LaravelMpdf::loadHTML($pdfContent, [
            'title' => 'Label',
            'format' => 'A8-L',
            'orientation' => 'L',
            'margin_left' => 1,
            'margin_right' => 1,
            'margin_top' => 5,
            'margin_bottom' => 1,
        ]);

        return $pdf;
    }

    public static function insertUpdateHooks($data, $isCreate = true)
    {
        $currentRole = session('userData.current_role');
        $satkerCms = MyHelper::toIdSatkerCms($currentRole['ms_satker_id']);
        $data['id_kejati'] = $satkerCms['id_kejati'];
        $data['id_kejari'] = $satkerCms['id_kejari'];
        $data['id_cabjari'] = $satkerCms['id_cabjari'];
        $data['id_satker_keu'] = $currentRole['ms_satker_id_keu'] ?? null;
        if ($isCreate) {
            $data['created_by'] = session('userData.username');
        } else {
            $data['updated_by'] = session('userData.username');
        }

        return $data;
    }

    public static function getTingkatRole($satkerId)
    {
        if ($satkerId == '00') {
            return 'KEJAGUNG';
        }
        if (strlen($satkerId) == 2) {
            return 'KEJATI';
        } else {
            return 'KEJARI';
        }
    }

    public static function dateFormat($date, $tipe = 'NORMAL')
    {
        if (! $date) {
            return '-';
        }

        $exp = explode(' ', $date);
        $tanggal = $exp[0];
        $waktu = $exp[1] ?? null;
        [$tahun, $bulan, $tanggal] = explode('-', $tanggal);
        $tgl = "{$tanggal}-{$bulan}-{$tahun}";
        if ($waktu) {
            [$jam, $menit] = explode(':', $waktu);

            return "{$tgl} {$jam}:{$menit}";
        }

        return $tgl;
    }

    public static function getShortMonth(?int $key = null)
    {
        $months = [
            [
                'id' => 1,
                'textId' => '01',
                'text' => 'Januari',
                'shortText' => 'Jan',
            ],
            [
                'id' => 2,
                'textId' => '02',
                'text' => 'Februari',
                'shortText' => 'Feb',
            ],
            [
                'id' => 3,
                'textId' => '03',
                'shortText' => 'Mar',
                'text' => 'Maret',
            ],
            [
                'id' => 4,
                'textId' => '04',
                'shortText' => 'Apr',
                'text' => 'April',
            ],
            [
                'id' => 5,
                'textId' => '05',
                'shortText' => 'Mei',
                'text' => 'Mei',
            ],
            [
                'id' => 6,
                'textId' => '06',
                'shortText' => 'Jun',
                'text' => 'Juni',
            ],
            [
                'id' => 7,
                'textId' => '07',
                'shortText' => 'Jul',
                'text' => 'Juli',
            ],
            [
                'id' => 8,
                'textId' => '08',
                'shortText' => 'Agu',
                'text' => 'Agustus',
            ],
            [
                'id' => 9,
                'textId' => '09',
                'shortText' => 'Sep',
                'text' => 'September',
            ],
            [
                'id' => 10,
                'textId' => '10',
                'shortText' => 'Oct',
                'text' => 'Oktober',
            ],
            [
                'id' => 11,
                'textId' => '11',
                'shortText' => 'Nop',
                'text' => 'Nopember',
            ],
            [
                'id' => 12,
                'textId' => '12',
                'shortText' => 'Des',
                'text' => 'Desember',
            ],
        ];

        if (! $key) {
            return $months;
        }

        return $months[$key - 1];
    }

    public static function dateFormatIndo($completeDate, $short = false)
    {
        if (! $completeDate) {
            return '';
        }
        $exp = explode(' ', $completeDate);
        $date = $exp[0];
        $time = $exp[1] ?? null;
        $timeText = '';
        if ($time) {
            $timeText = substr($time, 0, 5);
        }
        $exDate = explode('-', $date);
        $month = $short ? self::getShortMonth($exDate[1])['shortText'] : self::getShortMonth($exDate[1])['text'];

        return "{$exDate[2]}-{$month}-{$exDate[0]} {$timeText}";
    }

    public static function getSatkerLevel($msSatkerId)
    {
        if ($msSatkerId == '00') {
            return 'KEJAGUNG';
        } elseif (strlen($msSatkerId) > 2) {
            return 'KEJARI';
        } else {
            return 'KEJATI';
        }
    }

    public static function generateLogData($operation, $model)
    {

        $role = session('userData.current_role');
        $newLog = [
            'username' => session('userData.username'),
            'ms_satker_id' => $role['ms_satker_id'],
            'ms_satker_pusat_id' => $role['ms_satker_pusat_id'],
            'operation' => $operation,
            'pkey' => $model->id,
            'table' => $model->getTable(),
            'keterangan' => $model::tableKet,
            'ip_address' => request()->ip(),
            'user_agent' => request()->header('User-Agent'),
        ];

        return $newLog;
    }

    public static function generateAssetpdf($view, $selectedColumns, $data, $defColumns, $judul)
    {
        $filename = str_replace(' ', '_', strtolower($judul)).'.pdf';

        // Handle different data formats
        $rows = [];
        if (isset($data['data'])) {
            if (is_object($data['data']) && method_exists($data['data'], 'toArray')) {
                $rows = $data['data']->toArray();
            } else {
                $rows = $data['data'];
            }
        }

        $pdf = LaravelMpdf::chunkLoadView('<html-separator/>', $view, [
            'headers' => $selectedColumns,
            'rows' => $rows,
            'judul' => $judul,
            'defColumns' => $defColumns,
        ], [], [
            'title' => $judul,
            'format' => 'A4-L',
            'orientation' => 'L',
        ]);

        return $pdf->stream($filename);
    }

    public static function generateTahun($start = null, $end = null)
    {
        $tahunEnd = $end ?? date('Y');
        $tahunMulai = $start ?? $tahunEnd - 5;
        $tahuns = [];
        for ($i = (int) $tahunEnd; $i >= $tahunMulai; $i--) {
            $tahuns[] = (string) $i;
        }

        return $tahuns;
    }

    public static function generateSlug($inputString)
    {
        $urlFriendlyString = str_replace(' ', '-', strtolower($inputString));

        $urlFriendlyString = preg_replace('/[^a-z0-9-_]+/', '-', $urlFriendlyString);

        $urlFriendlyString = trim($urlFriendlyString, '-_');

        return $urlFriendlyString;
    }

    public static function paginate($totalData, $pageSize = 15, $currentPage = 1)
    {
        $totalPages = ceil($totalData / $pageSize);

        // Ensure the current page is within bounds
        if ($currentPage < 1) {
            $currentPage = 1;
        } elseif ($currentPage > $totalPages) {
            $currentPage = $totalPages;
        }

        $paginationLinks = '<div class="align-items-center py-4 row text-center justify-content-center text-sm-start">
                                <div class="col-sm-auto">
                                    <ul
                                        class="pagination pagination-separated pagination-sm justify-content-center justify-content-sm-start mb-0">';

        // <li class="page-item">
        //     <a href="#" class="page-link">1</a>
        // </li>
        // <li class="page-item active">
        //     <a href="#" class="page-link">2</a>
        // </li>
        // <li class="page-item">
        //     <a href="#" class="page-link">3</a>
        // </li>
        // <li class="page-item">
        //     <a href="#" class="page-link">→</a>
        // </li>

        // Generate the "Previous" link
        // if ($currentPage > 1) {
        //     $prevPage = $currentPage - 1;
        //     // $paginationLinks .= '<a href="?page=' . $prevPage . '">Previous</a>';
        //     $paginationLinks .=
        //         "<li class='page-item'>
        //             <a href='?page={$prevPage}' class='page-link'>←</a>
        //         </li>";
        // }

        // Generate the numbered page links (up to 5 pages)
        for ($i = max(1, $currentPage - 2); $i <= min($currentPage + 2, $totalPages); $i++) {
            // if ($i == $currentPage) {
            //     $paginationLinks[] = '<span>' . $i . '</span>';
            // } else {
            //     $paginationLinks[] = '<a href="?page=' . $i . '">' . $i . '</a>';
            // }
            $link = "?page={$i}";
            $active = '';
            if ($i == $currentPage) {
                $link = '#';
                $active = 'active';
            }
            $paginationLinks .=
                "<li class='page-item'>
                    <a href='{$link}' class='page-link {$active}'>{$i}</a>
                </li>";
        }

        // Generate the "Next" link
        // if ($currentPage < $totalPages) {
        //     $nextPage = $currentPage + 1;
        //     $paginationLinks .=
        //         "<li class='page-item'>
        //             <a href='?page={$nextPage}' class='page-link'>→</a>
        //         </li>";
        // }

        $paginationLinks .= ' </ul>
                                </div>
                            </div>';
        $paginationInfo = [
            'currentPage' => $currentPage,
            'pageSize' => $pageSize,
            'totalData' => $totalData,
            'totalPages' => $totalPages,
            // 'paginationElms' => $paginationLinks,
        ];

        // dd($paginationInfo);
        //
        return $paginationLinks;
    }

    // Example usage:
    // $totalData = 100; // Total number of data items
    // $pageSize = 10; // Number of items per page
    // $currentPage = isset($_GET['page']) ? intval($_GET['page']) : 1; // Current page, can be obtained from a query parameter

    // $pagingInfo = paginate($totalData, $pageSize, $currentPage);

    // // Access the generated pagination links
    // foreach ($pagingInfo['paginationLinks'] as $link) {
    //     echo $link . " ";
    // }

    //     }
    public static function isAdmin()
    {
        return session('userData.current_role.ms_role_id') == config('constants.admin_biro_lengkap_role_id');
    }

    public static function isSuperAdmin()
    {
        return session('userData.current_role.ms_role_id') == config('constants.superadmin_role_id');
    }

    public static function isValidatorPusat()
    {
        return session('userData.current_role.ms_role_id') == config('constants.validator_pusat_role_id');
    }

    public static function isValidatorWilayah()
    {
        return session('userData.current_role.ms_role_id') == config('constants.validator_wilayah_role_id');
    }

    public static function isPelaksanaSatker()
    {

        return session('userData.current_role.ms_role_id') == config('constants.pelaksana_satker_role_id');
    }

    public static function getFotoMysimkari($foto)
    {
        $baseFoto = config('constants.mysimkari_foto');

        if (empty($foto)) {
            return '';
        }

        // ✅ Cegah URL double
        if (str_starts_with($foto, 'http://') || str_starts_with($foto, 'https://')) {
            return $foto;
        }

        return "{$baseFoto}{$foto}";
    }

    public static function money2int($duit)
    {
        return str_replace('.', '', $duit);
    }

    public static function int2money($duit)
    {
        return number_format($duit, 0, '', ',');
    }
}
