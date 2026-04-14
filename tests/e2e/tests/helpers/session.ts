import { Page } from '@playwright/test';

export type SupportedRole =
  | 'operator_satker'
  | 'validator_wilayah'
  | 'validator_pusat'
  | 'admin';

interface MockSessionInput {
  role: SupportedRole;
  permissions?: string[];
}

function buildMockSession(input: MockSessionInput): string {
  return JSON.stringify({
    id: '20000000-0000-0000-0000-000000000001',
    username: '199203142014031001',
    name: 'User E2E',
    email: 'e2e@kejaksaan.go.id',
    role: { Custom: input.role },
    avatar: null,
    division: 'Biro Perlengkapan',
    captcha_validated: true,
    mfa_enabled: false,
    mfa_setup_required: false,
    created_at: new Date().toISOString(),
    access_token: 'mock-jwt-token',
    refresh_token: 'mock-refresh',
    expires_at: Math.floor(Date.now() / 1000) + 86400,
    permissions: input.permissions ?? (input.role === 'admin' ? ['*'] : []),
  });
}

export async function loginAsRole(page: Page, role: SupportedRole): Promise<void> {
  const mockSession = buildMockSession({ role });

  // Set localStorage before app scripts evaluate.
  await page.addInitScript(
    ({ session, activeRole }) => {
      localStorage.setItem('user_session', session);
      localStorage.setItem('auth_token', 'mock-jwt-token-for-e2e');
      localStorage.setItem('active_role', activeRole);
      localStorage.setItem(
        'available_roles',
        JSON.stringify(['operator_satker', 'validator_wilayah', 'validator_pusat', 'admin'])
      );
    },
    { session: mockSession, activeRole: role }
  );
}
