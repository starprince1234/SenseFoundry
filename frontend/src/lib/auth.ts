import * as oidc from 'openid-client';
import { browser } from '$app/environment';
import { env } from '$env/dynamic/public';

export type Session = { accessToken: string; idToken?: string; role: string };
const sessionKey = 'sensefoundry.session';

async function configuration(): Promise<oidc.Configuration> {
  const issuer = env.PUBLIC_KEYCLOAK_ISSUER;
  const clientId = env.PUBLIC_OIDC_CLIENT_ID;
  if (!issuer || !clientId) throw new Error('PUBLIC_KEYCLOAK_ISSUER and PUBLIC_OIDC_CLIENT_ID are required');
  return oidc.discovery(new URL(issuer), clientId);
}

export async function login(): Promise<void> {
  const config = await configuration();
  const verifier = oidc.randomPKCECodeVerifier();
  sessionStorage.setItem('oidc.verifier', verifier);
  const challenge = await oidc.calculatePKCECodeChallenge(verifier);
  const url = oidc.buildAuthorizationUrl(config, {
    redirect_uri: `${location.origin}/`, scope: 'openid profile roles',
    code_challenge: challenge, code_challenge_method: 'S256'
  });
  location.assign(url.href);
}

export async function completeLogin(): Promise<void> {
  if (!browser || !new URL(location.href).searchParams.has('code')) return;
  const verifier = sessionStorage.getItem('oidc.verifier');
  if (!verifier) throw new Error('Missing OIDC PKCE verifier');
  const tokens = await oidc.authorizationCodeGrant(await configuration(), new URL(location.href), { pkceCodeVerifier: verifier });
  const claims = tokens.claims();
  const roles = (claims?.realm_access as { roles?: string[] } | undefined)?.roles ?? [];
  sessionStorage.setItem(sessionKey, JSON.stringify({ accessToken: tokens.access_token, idToken: tokens.id_token, role: roles[0] ?? 'expert' }));
  history.replaceState({}, '', '/');
}

export function getSession(): Session | null {
  if (!browser) return null;
  const value = sessionStorage.getItem(sessionKey);
  return value ? JSON.parse(value) as Session : null;
}

export function logout(): void { sessionStorage.clear(); location.assign('/'); }
