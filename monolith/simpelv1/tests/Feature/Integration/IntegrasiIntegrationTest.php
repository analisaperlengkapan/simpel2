<?php

namespace Tests\Feature\Integration;

use App\Services\Grpc\IntegrasiGrpcClient;
use Illuminate\Support\Facades\Http;
use Tests\TestCase;

class IntegrasiIntegrationTest extends TestCase
{
    private const GW = 'http://integrasi-gw.test';

    protected function setUp(): void
    {
        parent::setUp();
        config(['services.gateway.integrasi.url' => self::GW]);
        config(['services.gateway.timeout' => 5.0]);
    }

    public function test_get_employees_success(): void
    {
        Http::fake([
            self::GW . '/v1/mysimkari/employees*' => Http::response([
                'items' => [['nip' => '123', 'nama' => 'User']],
                'pagination' => ['total_items' => 1]
            ], 200)
        ]);

        $result = (new IntegrasiGrpcClient)->getEmployees(['nip_filter' => '123']);

        $this->assertIsArray($result);
        $this->assertCount(1, $result['items']);
        $this->assertEquals('123', $result['items'][0]['nip']);
    }

    public function test_get_assets_success(): void
    {
        Http::fake([
            self::GW . '/v1/siman/assets*' => Http::response([
                'items' => [['id' => 'A1', 'nama_barang' => 'Asset 1']],
                'pagination' => ['total_items' => 1]
            ], 200)
        ]);

        $client = new IntegrasiGrpcClient();
        $result = $client->getAssets($client::SIMAN_CAT_TANAH, ['kode_satker' => 'S1']);

        $this->assertIsArray($result);
        $this->assertCount(1, $result['items']);
        $this->assertEquals('A1', $result['items'][0]['id']);
    }

    public function test_fetch_employee_fallback_works(): void
    {
        // 404 for exact match, but search returns it
        Http::fake([
            self::GW . '/v1/mysimkari/employees/123' => Http::response([], 404),
            self::GW . '/v1/mysimkari/employees?nip_filter=123&per_page=1' => Http::response([
                'items' => [['nip' => '123', 'nama' => 'User']]
            ], 200)
        ]);

        $result = (new IntegrasiGrpcClient)->fetchEmployeeFromMySIMKARI('123');

        $this->assertIsArray($result);
        $this->assertEquals('123', $result['nip']);
    }
}
