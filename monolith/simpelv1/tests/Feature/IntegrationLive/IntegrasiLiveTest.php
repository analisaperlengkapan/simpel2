<?php

namespace Tests\Feature\IntegrationLive;

use App\Services\Grpc\IntegrasiGrpcClient;
use Tests\TestCase;

/**
 * simpelv1 ↔ layanan-integrasi — LIVE (F-GW PR-C).
 *
 * Runs the real IntegrasiGrpcClient against a LIVE gateway → integrasi gRPC.
 * Skips unless INTEGRASI_GATEWAY_URL is set.
 *
 * Seed: seed-multisatker.sql (integrasi.siman_aset) + seed-simpelv1-live.sql
 * (integrasi.mysimkari_pegawai). NOTE the SIMAN assertion uses the "Tanah" asset
 * E2E-B-2: the gateway's by-id inventory scan only queries the Tanah /
 * GedungBangunan / AlatBesar / AngkutanBermotor categories, so the seeded
 * "Peralatan dan Mesin" rows (E2E-A-x and E2E-C-x) are intentionally NOT
 * reachable by id.
 */
class IntegrasiLiveTest extends TestCase
{
    protected function setUp(): void
    {
        parent::setUp();
        $gw = getenv('INTEGRASI_GATEWAY_URL');
        if ($gw === false || $gw === '') {
            $this->markTestSkipped('INTEGRASI_GATEWAY_URL unset — live gateway stack not present');
        }
        config(['services.gateway.integrasi.url' => $gw]);
        config(['services.gateway.timeout' => 10.0]);
    }

    public function test_gateway_health_is_up(): void
    {
        $this->assertTrue((new IntegrasiGrpcClient)->health());
    }

    public function test_fetch_siman_inventory_returns_seeded_tanah_asset(): void
    {
        // E2E-B-2 = jenis_aset 'Tanah' (seed-multisatker.sql) — the gateway's
        // 4-category by-id scan includes Tanah, so this nup is reachable.
        $asset = (new IntegrasiGrpcClient)->fetchInventoryFromSIMAN('E2E-B-2');

        $this->assertIsArray($asset);
        $this->assertStringContainsString('E2E-B-2', json_encode($asset));
    }

    public function test_fetch_mysimkari_employee_returns_seeded_pegawai(): void
    {
        // NIP seeded in seed-simpelv1-live.sql → gateway returns the first match.
        $emp = (new IntegrasiGrpcClient)->fetchEmployeeFromMySIMKARI('200000000000000001');

        $this->assertIsArray($emp);
        $this->assertStringContainsString('200000000000000001', json_encode($emp));
    }

    public function test_monsakti_is_dormant_returns_null(): void
    {
        // MonSAKTI dormant (migrating to MyIntress) → gateway 501 → client null.
        $this->assertNull((new IntegrasiGrpcClient)->fetchAssetFromMonSAKTI('anything'));
    }
}
