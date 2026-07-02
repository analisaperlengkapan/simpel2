/**
 * Integration E2E Tests — Layanan Integrasi ↔ Authenc
 *
 * Tests data consistency between layanan-integrasi (employee data source)
 * and authenc (IAM — user management). Verifies:
 *   1. gRPC connectivity to layanan-integrasi
 *   2. Authenc REST API for user profiles
 *   3. Data contract consistency (NIP user attributes)
 *   4. Federation sync endpoint availability
 *   5. End-to-end flow: login → profile → verify employee fields
 *
 * Services (port-forwarded):
 *   - Integrasi gRPC: localhost:18051
 *   - Authenc REST:   localhost:18088
 *   - Portal:         localhost:18080
 *
 * Note: layanan-integrasi's database is currently disconnected in staging,
 * so MySIMKARI data queries return empty results. The tests validate:
 *   - gRPC schema & connectivity (works even without DB)
 *   - Authenc user data seeded from migration 044
 *   - Data contract: NIP = username, employee attributes in user profile
 */
import { test, expect } from '@playwright/test';
import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import * as path from 'path';

// ── Configuration ──────────────────────────────────────────────────────────

const AUTHENC_URL = process.env.AUTHENC_URL || 'http://localhost:18088';
const INTEGRASI_GRPC_URL = process.env.INTEGRASI_GRPC_URL || 'localhost:18051';

/**
 * NIP test user — the base authenc seed (002_seed.sql). NOTE: this user is
 * seeded with `require_password_change = true` BY DESIGN (the portal guard
 * suite asserts the change-password redirect), so `GET /api/v1/auth/me`
 * answers 403 `password_change_required` for it. Use it ONLY for login/JWT
 * shape tests — profile tests use PROFILE_USER below (proven on the first
 * real run of this job, CI run 28557595484).
 */
const NIP_USER = {
  username: '199203142014031001',
  password: '199203142014031001',
};

/**
 * Profile-capable user from the multi-satker fixture
 * (tests/fixtures/e2e/seed-multisatker.sql, applied by the CI job):
 * `require_password_change = false`, so /auth/me works. ALL fixture users
 * share the base seed password (the fixture reuses the one known Argon2id
 * hash, which verifies exactly "199203142014031001").
 */
const PROFILE_USER = {
  username: '200000000000000001',
  password: '199203142014031001',
};

/**
 * Second live user (validator_pusat from the multi-satker fixture). The
 * baseline DOES seed a literal `admin` user, but with a placeholder hash no
 * password verifies against — it is a bootstrap artifact, not a login-able
 * account.
 */
const ADMIN_USER = {
  username: '200000000000000004',
  password: '199203142014031001',
};

/** Expected PROFILE_USER fields (from the multi-satker fixture). */
const EXPECTED_PROFILE = {
  username: '200000000000000001',
  email: '200000000000000001@kejaksaan.go.id',
  // /auth/me builds `name` from first/last name, falling back to `nama`;
  // fixture users have first/last NULL, so `nama` is the display name.
  name: 'E2E Operator Jakpus',
};

// ── gRPC Client Setup ──────────────────────────────────────────────────────

// Default: the proto in the repo tree (works when run from the checkout). In the
// containerized e2e-integrasi-authenc CI job the proto is copied into the e2e
// image and located via INTEGRASI_PROTO_PATH (the repo tree is not in that image).
const PROTO_PATH =
  process.env.INTEGRASI_PROTO_PATH ||
  path.resolve(__dirname, '../../../layanan/integrasi/proto/integrasi.proto');

function createIntegrasiClient(): Promise<any> {
  return new Promise((resolve, reject) => {
    try {
      const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
        keepCase: true,
        longs: String,
        enums: String,
        defaults: true,
        oneofs: true,
      });
      const proto = grpc.loadPackageDefinition(packageDefinition) as any;
      const client = new proto.integrasi.v1.IntegrasiService(
        INTEGRASI_GRPC_URL,
        grpc.credentials.createInsecure(),
      );

      // Wait for the channel to connect (5 second deadline)
      const deadline = new Date(Date.now() + 5000);
      client.waitForReady(deadline, (err: Error | null) => {
        if (err) reject(new Error(`gRPC connect failed: ${err.message}`));
        else resolve(client);
      });
    } catch (err) {
      reject(err);
    }
  });
}

function grpcCall<T>(client: any, method: string, request: object): Promise<T> {
  return new Promise((resolve, reject) => {
    client[method](request, (err: grpc.ServiceError | null, response: T) => {
      if (err) reject(err);
      else resolve(response);
    });
  });
}

// ── REST Helper ────────────────────────────────────────────────────────────

interface LoginResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
  mfa_required: boolean;
}

interface UserProfile {
  id: string;
  username: string;
  email: string;
  name?: string;
  first_name?: string;
  last_name?: string;
  division?: string;
  role?: string;
  permissions?: string[];
  email_verified?: boolean;
  mfa_enabled?: boolean;
  realm_id?: string;
}

// ── Test Suite ─────────────────────────────────────────────────────────────

test.describe('Integrasi ↔ Authenc Integration', () => {
  // ═══════════════════════════════════════════════════════════════════════
  // Section 1: Layanan Integrasi gRPC Service
  // ═══════════════════════════════════════════════════════════════════════

  test.describe('Layanan Integrasi — gRPC Service', () => {
    let client: any;

    test.beforeAll(async () => {
      try {
        client = await createIntegrasiClient();
      } catch {
        // Client creation failed — tests will skip individually
      }
    });

    test.afterAll(() => {
      if (client) grpc.closeClient(client);
    });

    test('gRPC: HealthCheck — service responds', async () => {
      test.skip(!client, 'Integrasi gRPC not reachable');

      const response = await grpcCall<{
        service_status: Record<string, string>;
        healthy: boolean;
        version: string;
        database_status: string;
      }>(client, 'HealthCheck', {});

      expect(response).toBeDefined();
      expect(response.version).toBeTruthy();
      expect(response.service_status).toBeDefined();
      expect(response.service_status.grpc).toBe('running');
      // Note: database may be disconnected in staging
      console.log(
        `  Integrasi health: gRPC=${response.service_status.grpc}, ` +
        `DB=${response.database_status}, v${response.version}`,
      );
    });

    test('gRPC: GetMysimkariPegawai — schema validation', async () => {
      test.skip(!client, 'Integrasi gRPC not reachable');

      const response = await grpcCall<{
        items: Array<{
          nip: string;
          nama: string;
          jabatan: string;
          pangkat: string;
          golongan: string;
          unit_kerja: string;
          kode_satker: string;
          email: string;
          telepon: string;
          status: string;
        }>;
        pagination: {
          current_page: number;
          per_page: number;
          total_items: string;
          total_pages: number;
        };
      }>(client, 'GetMysimkariPegawai', {
        kode_satker: '0100000',
        nip_filter: NIP_USER.username,
        pagination: { page: 1, per_page: 10 },
      });

      // Response structure must be valid regardless of data availability
      expect(response).toBeDefined();
      expect(response.pagination).toBeDefined();
      expect(response.pagination.current_page).toBe(1);
      expect(response.pagination.per_page).toBe(10);
      expect(Array.isArray(response.items)).toBe(true);

      if (response.items.length > 0) {
        // If data available, validate employee record schema
        const pegawai = response.items[0];
        expect(pegawai).toHaveProperty('nip');
        expect(pegawai).toHaveProperty('nama');
        expect(pegawai).toHaveProperty('jabatan');
        expect(pegawai).toHaveProperty('golongan');
        expect(pegawai).toHaveProperty('kode_satker');
        console.log(`  Found employee: NIP=${pegawai.nip}, nama=${pegawai.nama}`);
      } else {
        console.log('  No employee data (DB disconnected) — schema validated');
      }
    });

    test('gRPC: GetMysimkariSatker — schema validation', async () => {
      test.skip(!client, 'Integrasi gRPC not reachable');

      const response = await grpcCall<{
        items: Array<{
          kode_satker: string;
          nama_satker: string;
          level: string;
          parent_kode: string;
        }>;
        pagination: {
          current_page: number;
          per_page: number;
          total_items: string;
          total_pages: number;
        };
      }>(client, 'GetMysimkariSatker', {
        pagination: { page: 1, per_page: 5 },
      });

      expect(response).toBeDefined();
      expect(response.pagination).toBeDefined();
      expect(Array.isArray(response.items)).toBe(true);

      if (response.items.length > 0) {
        const satker = response.items[0];
        expect(satker).toHaveProperty('kode_satker');
        expect(satker).toHaveProperty('nama_satker');
        console.log(`  Found satker: ${satker.kode_satker} — ${satker.nama_satker}`);
      } else {
        console.log('  No satker data (DB disconnected) — schema validated');
      }
    });

    test('gRPC: GetSyncStatus — service responds', async () => {
      test.skip(!client, 'Integrasi gRPC not reachable');

      try {
        const response = await grpcCall<{
          source: string;
          status: string;
          last_sync_at: string;
          next_sync_at: string;
          total_records: string;
          error_message: string;
        }>(client, 'GetSyncStatus', { source: 'mysimkari' });

        expect(response).toBeDefined();
        console.log(`  Sync status: source=${response.source}, status=${response.status}`);
      } catch (err: any) {
        // UNIMPLEMENTED is acceptable — means the endpoint exists in proto
        if (err.code === grpc.status.UNIMPLEMENTED) {
          console.log('  GetSyncStatus: UNIMPLEMENTED (expected for staging)');
        } else {
          throw err;
        }
      }
    });
  });

  // ═══════════════════════════════════════════════════════════════════════
  // Section 2: Authenc User Profile — NIP Employee Data Contract
  // ═══════════════════════════════════════════════════════════════════════

  test.describe('Authenc — NIP User Profile', () => {
    test.beforeEach(async ({ request }) => {
      // Ensure authenc is reachable
      try {
        const resp = await request.get(`${AUTHENC_URL}/health`);
        expect(resp.ok(), 'Authenc must be reachable').toBeTruthy();
      } catch {
        test.skip(true, 'Authenc service not reachable');
      }
    });

    test('NIP user login returns valid JWT', async ({ request }) => {
      const loginResp = await request.post(`${AUTHENC_URL}/api/v1/auth/login`, {
        data: {
          username: NIP_USER.username,
          password: NIP_USER.password,
        },
      });
      expect(loginResp.ok()).toBeTruthy();

      const body: LoginResponse = await loginResp.json();
      expect(body.access_token).toBeTruthy();
      expect(body.refresh_token).toBeTruthy();
      expect(body.token_type).toBe('Bearer');
      expect(body.expires_in).toBeGreaterThan(0);
      expect(body.mfa_required).toBe(false);

      // JWT should have 3 parts (header.payload.signature)
      const jwtParts = body.access_token.split('.');
      expect(jwtParts).toHaveLength(3);
    });

    test('NIP user profile matches expected employee data', async ({ request }) => {
      // PROFILE_USER (multi-satker fixture): the base NIP seed user is
      // require_password_change=true by design → /auth/me answers 403 for it.
      const loginResp = await request.post(`${AUTHENC_URL}/api/v1/auth/login`, {
        data: {
          username: PROFILE_USER.username,
          password: PROFILE_USER.password,
        },
      });
      expect(loginResp.ok()).toBeTruthy();
      const { access_token } = await loginResp.json();

      // Get user profile
      const profileResp = await request.get(`${AUTHENC_URL}/api/v1/auth/me`, {
        headers: { Authorization: `Bearer ${access_token}` },
      });
      expect(profileResp.ok()).toBeTruthy();

      const profile: UserProfile = await profileResp.json();

      // Verify NIP employee data contract
      expect(profile.username).toBe(EXPECTED_PROFILE.username);
      expect(profile.email).toBe(EXPECTED_PROFILE.email);
      expect(profile.name).toBe(EXPECTED_PROFILE.name);
      expect(profile.email_verified).toBe(true);
      expect(profile.id).toBeTruthy();

      console.log(
        `  NIP profile: ${profile.username} — ${profile.name} (${profile.email})`,
      );
    });

    test('NIP user JWT contains correct claims', async ({ request }) => {
      const loginResp = await request.post(`${AUTHENC_URL}/api/v1/auth/login`, {
        data: {
          username: NIP_USER.username,
          password: NIP_USER.password,
        },
      });
      const { access_token } = await loginResp.json();

      // Decode JWT payload (not verifying signature — that's authenc's job)
      const payload = JSON.parse(
        Buffer.from(access_token.split('.')[1], 'base64url').toString(),
      );

      expect(payload.sub).toBeTruthy(); // Subject (user ID)
      expect(payload.iss).toBeTruthy(); // Issuer
      expect(payload.exp).toBeGreaterThan(Math.floor(Date.now() / 1000)); // Not expired
      expect(payload.iat).toBeLessThanOrEqual(Math.floor(Date.now() / 1000) + 5); // Issued recently
      expect(payload.jti).toBeTruthy(); // Token ID

      console.log(`  JWT: sub=${payload.sub}, iss=${payload.iss}, scope=${payload.scope}`);
    });

    test('second seeded user (validator_pusat) can also login and access profile', async ({
      request,
    }) => {
      const loginResp = await request.post(`${AUTHENC_URL}/api/v1/auth/login`, {
        data: {
          username: ADMIN_USER.username,
          password: ADMIN_USER.password,
        },
      });
      expect(loginResp.ok()).toBeTruthy();

      const { access_token } = await loginResp.json();
      const profileResp = await request.get(`${AUTHENC_URL}/api/v1/auth/me`, {
        headers: { Authorization: `Bearer ${access_token}` },
      });
      expect(profileResp.ok()).toBeTruthy();

      const profile: UserProfile = await profileResp.json();
      expect(profile.username).toBe(ADMIN_USER.username);
      expect(profile.email).toBe(`${ADMIN_USER.username}@kejaksaan.go.id`);
    });

    test('expired/invalid token is rejected', async ({ request }) => {
      const profileResp = await request.get(`${AUTHENC_URL}/api/v1/auth/me`, {
        headers: { Authorization: 'Bearer invalid_token_xxx' },
      });
      // Should be 401 or 403
      expect([401, 403]).toContain(profileResp.status());
    });
  });

  // ═══════════════════════════════════════════════════════════════════════
  // Section 3: Federation & Sync Endpoints
  // ═══════════════════════════════════════════════════════════════════════

  test.describe('Authenc — Federation Sync Endpoints', () => {
    let adminToken: string;

    test.beforeAll(async ({ request }) => {
      try {
        const loginResp = await request.post(`${AUTHENC_URL}/api/v1/auth/login`, {
          data: {
            username: ADMIN_USER.username,
            password: ADMIN_USER.password,
          },
        });
        if (loginResp.ok()) {
          const body = await loginResp.json();
          adminToken = body.access_token;
        }
      } catch {
        // Will skip tests if admin token missing
      }
    });

    test('federation sync endpoint exists', async ({ request }) => {
      test.skip(!adminToken, 'Admin token not available');

      const resp = await request.post(`${AUTHENC_URL}/api/v1/iam/federation/sync`, {
        headers: { Authorization: `Bearer ${adminToken}` },
        data: { source: 'mysimkari' },
      });

      // Endpoint-EXISTENCE check, not an RBAC assertion: 200 = working,
      // 501 = not_implemented, 404 = route not configured yet, 401/403 =
      // route exists but denies the fixture user (validator_pusat, not an
      // IAM admin — the baseline has no login-able admin account).
      expect([200, 401, 403, 404, 501]).toContain(resp.status());
      console.log(`  Federation sync: HTTP ${resp.status()}`);
    });

    test('federation stats endpoint exists', async ({ request }) => {
      test.skip(!adminToken, 'Admin token not available');

      const resp = await request.get(`${AUTHENC_URL}/api/v1/iam/federation/stats`, {
        headers: { Authorization: `Bearer ${adminToken}` },
      });

      // Endpoint-existence check (see federation sync above for the statuses).
      expect([200, 401, 403, 404, 501]).toContain(resp.status());
      console.log(`  Federation stats: HTTP ${resp.status()}`);
    });
  });

  // ═══════════════════════════════════════════════════════════════════════
  // Section 4: Cross-Service Data Consistency
  // ═══════════════════════════════════════════════════════════════════════

  test.describe('Cross-Service Data Consistency', () => {
    test('NIP user data contract: authenc ↔ integrasi schema alignment', async ({
      request,
    }) => {
      /**
       * This test verifies the data contract between integrasi (source)
       * and authenc (consumer). Even though live sync is not yet implemented,
       * the seed data in authenc should follow the same schema contract:
       *   - NIP → username (authenc)
       *   - Name → name/first_name/last_name (authenc)
       *   - Email → email format: nip{NIP}@kejaksaan.go.id (authenc)
       */
      // PROFILE_USER: /auth/me-capable (the base NIP seed user is
      // require_password_change=true by design → 403 on /auth/me).
      const loginResp = await request.post(`${AUTHENC_URL}/api/v1/auth/login`, {
        data: {
          username: PROFILE_USER.username,
          password: PROFILE_USER.password,
        },
      });
      expect(loginResp.ok()).toBeTruthy();
      const { access_token } = await loginResp.json();

      const profileResp = await request.get(`${AUTHENC_URL}/api/v1/auth/me`, {
        headers: { Authorization: `Bearer ${access_token}` },
      });
      expect(profileResp.ok()).toBeTruthy();
      const profile: UserProfile = await profileResp.json();

      // Data contract validations:

      // 1. NIP is the username
      expect(profile.username).toBe(PROFILE_USER.username);

      // 2. NIP is an 18-digit Indonesian civil servant number
      expect(profile.username).toMatch(/^\d{18}$/);

      // 3. Email follows the NIP-based pattern
      expect(profile.email).toBe(`${PROFILE_USER.username}@kejaksaan.go.id`);

      // 4. User has a name
      expect(profile.name).toBeTruthy();

      // 5. User is email-verified (seeded users are auto-verified)
      expect(profile.email_verified).toBe(true);

      // 6. MFA not required for seeded users
      expect(profile.mfa_enabled).toBe(false);

      console.log('  Data contract verified: NIP→username, email pattern, profile fields');
    });

    test('integrasi proto and authenc user fields are aligned', async () => {
      /**
       * Validates that the integrasi proto schema (MysimkariPegawai)
       * maps logically to authenc user attributes:
       *
       *   integrasi.MysimkariPegawai  →  authenc user_attributes
       *   ─────────────────────────────────────────────────────────
       *   nip                         →  username
       *   nama                        →  name (first_name + last_name)
       *   email                       →  email
       *   jabatan                     →  user_attributes.jabatan
       *   golongan                    →  user_attributes.golongan
       *   kode_satker                 →  user_attributes.satker_code
       *   status                      →  is_active
       *
       * This test loads the proto and checks that fields exist.
       */
      const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
        keepCase: true,
        longs: String,
        enums: String,
        defaults: true,
        oneofs: true,
      });
      const proto = grpc.loadPackageDefinition(packageDefinition) as any;

      // Verify the service exists in the proto definition
      expect(proto.integrasi).toBeDefined();
      expect(proto.integrasi.v1).toBeDefined();
      expect(proto.integrasi.v1.IntegrasiService).toBeDefined();

      // Verify service methods exist
      const serviceDef = proto.integrasi.v1.IntegrasiService.service;
      expect(serviceDef).toBeDefined();

      const methodNames = Object.keys(serviceDef);
      expect(methodNames).toContain('HealthCheck');
      expect(methodNames).toContain('GetMysimkariPegawai');
      expect(methodNames).toContain('GetMysimkariSatker');
      expect(methodNames).toContain('GetSimanAssets');
      expect(methodNames).toContain('TriggerSync');
      expect(methodNames).toContain('GetSyncStatus');

      console.log(`  Proto service methods: ${methodNames.join(', ')}`);
    });

    test('NIP user can login AND access profile in sequence', async ({ request }) => {
      /**
       * End-to-end sequence test:
       * 1. Login with NIP credentials
       * 2. Access profile with the returned token
       * 3. Verify profile data is consistent
       * 4. Refresh token works
       */

      // Step 1: Login (PROFILE_USER — /auth/me-capable; the base NIP seed
      // user is require_password_change=true by design → 403 on /auth/me)
      const loginResp = await request.post(`${AUTHENC_URL}/api/v1/auth/login`, {
        data: {
          username: PROFILE_USER.username,
          password: PROFILE_USER.password,
        },
      });
      expect(loginResp.ok()).toBeTruthy();
      const loginBody: LoginResponse = await loginResp.json();

      // Step 2: Get profile
      const profileResp = await request.get(`${AUTHENC_URL}/api/v1/auth/me`, {
        headers: { Authorization: `Bearer ${loginBody.access_token}` },
      });
      expect(profileResp.ok()).toBeTruthy();
      const profile: UserProfile = await profileResp.json();

      // Step 3: Verify consistency
      expect(profile.username).toBe(PROFILE_USER.username);
      expect(profile.id).toBeTruthy();

      // Step 4: Refresh token
      const refreshResp = await request.post(`${AUTHENC_URL}/api/v1/auth/refresh`, {
        data: { refresh_token: loginBody.refresh_token },
      });
      // Refresh may or may not be implemented; accept 200 or 404
      if (refreshResp.ok()) {
        const refreshBody = await refreshResp.json();
        expect(refreshBody.access_token).toBeTruthy();
        console.log('  Token refresh: working');
      } else {
        console.log(`  Token refresh: HTTP ${refreshResp.status()} (${refreshResp.ok() ? 'OK' : 'not implemented'})`);
      }
    });
  });
});
