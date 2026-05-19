import { APIRequestContext } from '@playwright/test';

export async function isServiceHealthy(
  request: APIRequestContext,
  path = '/'
): Promise<boolean> {
  try {
    const response = await request.get(path);
    // Consider service reachable if it responds and is not a server-side failure.
    return response.status() < 500;
  } catch {
    return false;
  }
}
