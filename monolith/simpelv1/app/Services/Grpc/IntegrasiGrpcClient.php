<?php

namespace App\Services\Grpc;

use Illuminate\Http\Client\PendingRequest;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;
use Throwable;

/**
 * Integrasi Gateway Client
 *
 * Talks to the Integrasi service via the K8s Sidecar / Rust Gateway Proxy
 * over HTTP/REST rather than direct gRPC. See `monolith/simpelv1/AGENTS.md`
 * for the rationale (php-fpm cannot afford to open mTLS gRPC channels
 * per request).
 *
 * Class name retained as `IntegrasiGrpcClient` for backwards compatibility.
 */
class IntegrasiGrpcClient
{
    private string $baseUrl;

    private float $timeout;

    public function __construct()
    {
        // Resolve via `config()` only — see AuthencGrpcClient::__construct
        // for why env() must not be called at runtime from a service class.
        $this->baseUrl = rtrim(
            (string) config('services.gateway.integrasi.url', 'http://127.0.0.1:8082'),
            '/'
        );
        $this->timeout = (float) config('services.gateway.timeout', 5.0);
    }

    private function http(): PendingRequest
    {
        return Http::timeout($this->timeout)->acceptJson();
    }

    /**
     * Fetch asset data from external system (MonSAKTI)
     */
    public function fetchAssetFromMonSAKTI(string $assetId): ?array
    {
        try {
            $response = $this->http()->get(
                "{$this->baseUrl}/v1/monsakti/assets/".rawurlencode($assetId)
            );
            if (! $response->successful()) {
                return null;
            }
            $data = $response->json();

            return is_array($data) ? $data : null;
        } catch (Throwable $th) {
            Log::warning("Failed to fetch asset from MonSAKTI: {$th->getMessage()}");

            return null;
        }
    }

    /**
     * Get list of employees from MySIMKARI with filtering and pagination
     */
    public function getEmployees(array $params = []): ?array
    {
        try {
            $response = $this->http()->get("{$this->baseUrl}/v1/mysimkari/employees", $params);
            if (! $response->successful()) {
                return null;
            }

            return $response->json();
        } catch (Throwable $th) {
            Log::warning("Failed to get employees from MySIMKARI: {$th->getMessage()}");

            return null;
        }
    }

    /**
     * Fetch employee data from MySIMKARI
     */
    public function fetchEmployeeFromMySIMKARI(string $nip): ?array
    {
        try {
            $response = $this->http()->get(
                "{$this->baseUrl}/v1/mysimkari/employees/".rawurlencode($nip)
            );
            if (! $response->successful()) {
                // Fallback to searching if exact match fails
                $search = $this->getEmployees(['nip_filter' => $nip, 'per_page' => 1]);
                if (! empty($search['items'])) {
                    return $search['items'][0];
                }

                return null;
            }
            $data = $response->json();

            return is_array($data) ? $data : null;
        } catch (Throwable $th) {
            Log::warning("Failed to fetch employee from MySIMKARI: {$th->getMessage()}");

            return null;
        }
    }

    /**
     * SIMAN Asset Categories (matching integrasi.proto enum)
     */
    public const SIMAN_CAT_TANAH = 1;

    public const SIMAN_CAT_GEDUNG_BANGUNAN = 2;

    public const SIMAN_CAT_ALAT_BESAR = 3;

    public const SIMAN_CAT_ANGKUTAN_BERMOTOR = 4;

    public const SIMAN_CAT_ALAT_PERSENJATAAN = 5;

    public const SIMAN_CAT_TAK_BERWUJUD = 6;

    public const SIMAN_CAT_TETAP_LAINNYA = 7;

    public const SIMAN_CAT_BANGUNAN_AIR = 8;

    public const SIMAN_CAT_INSTALASI_JARINGAN = 9;

    public const SIMAN_CAT_JALAN_JEMBATAN = 10;

    public const SIMAN_CAT_KDP = 11;

    public const SIMAN_CAT_KHUSUS_TIK = 12;

    public const SIMAN_CAT_NON_TIK = 13;

    public const SIMAN_CAT_RUMAH = 14;

    public const SIMAN_CAT_TETAP_RENOVASI = 15;

    /**
     * Get list of assets from SIMAN with filtering and pagination
     */
    public function getAssets(int $category, array $params = []): ?array
    {
        try {
            $params['category'] = $category;
            $response = $this->http()->get("{$this->baseUrl}/v1/siman/assets", $params);
            if (! $response->successful()) {
                return null;
            }

            return $response->json();
        } catch (Throwable $th) {
            Log::warning("Failed to get assets from SIMAN: {$th->getMessage()}");

            return null;
        }
    }

    /**
     * Fetch inventory data from SIMAN
     */
    public function fetchInventoryFromSIMAN(string $inventoryId): ?array
    {
        try {
            $response = $this->http()->get(
                "{$this->baseUrl}/v1/siman/inventory/".rawurlencode($inventoryId)
            );
            if (! $response->successful()) {
                return null;
            }
            $data = $response->json();

            return is_array($data) ? $data : null;
        } catch (Throwable $th) {
            Log::warning("Failed to fetch inventory from SIMAN: {$th->getMessage()}");

            return null;
        }
    }

    /**
     * Sync asset changes back to MonSAKTI
     */
    public function syncAssetToMonSAKTI(array $assetData): bool
    {
        try {
            $response = $this->http()
                ->asJson()
                ->post("{$this->baseUrl}/v1/monsakti/assets/sync", $assetData);

            return $response->successful();
        } catch (Throwable $th) {
            Log::warning("Failed to sync asset to MonSAKTI: {$th->getMessage()}");

            return false;
        }
    }

    /**
     * Check integration health
     */
    public function health(): bool
    {
        try {
            return $this->http()->get("{$this->baseUrl}/healthz")->successful();
        } catch (Throwable $th) {
            Log::warning("Integrasi gateway health check failed: {$th->getMessage()}");

            return false;
        }
    }
}
