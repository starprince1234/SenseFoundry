import { env } from '$env/dynamic/public';
import { getSession } from '$lib/auth';

export async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${env.PUBLIC_API_URL ?? 'http://localhost:8080'}${path}`, {
    ...init,
    headers: { 'Content-Type': 'application/json', ...(getSession() ? { Authorization: `Bearer ${getSession()?.accessToken}` } : {}), ...init?.headers }
  });
  if (!response.ok) throw new Error(`API request failed: ${response.status}`);
  return response.json() as Promise<T>;
}
