/**
 * E2E document encryption (Phase C) — WebCrypto AES-256-GCM, zero-knowledge:
 * key material NEVER leaves the client; the server stores only metadata
 * (document_id, algorithm, key fingerprint) via /e2e/keys.
 *
 * Mirrors the Leptos crypto.rs contract so both frontends can decrypt the
 * same documents.
 */

const ALGO = "AES-GCM";
const KEY_LENGTH = 256;
const IV_LENGTH = 12; // bytes — GCM standard

export interface EncryptedPayload {
  /** base64 iv */
  iv: string;
  /** base64 ciphertext||tag */
  ciphertext: string;
  algorithm: "AES-256-GCM";
}

function toBase64(buf: ArrayBuffer | Uint8Array): string {
  const bytes = buf instanceof Uint8Array ? buf : new Uint8Array(buf);
  let bin = "";
  for (const b of bytes) bin += String.fromCharCode(b);
  return btoa(bin);
}

function fromBase64(s: string): Uint8Array<ArrayBuffer> {
  const bin = atob(s);
  const out = new Uint8Array(new ArrayBuffer(bin.length));
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

/** Generate a fresh per-document key. */
export async function generateDocumentKey(): Promise<CryptoKey> {
  return crypto.subtle.generateKey({ name: ALGO, length: KEY_LENGTH }, true, [
    "encrypt",
    "decrypt",
  ]);
}

/** SHA-256 fingerprint of the key (registered server-side, never the key). */
export async function keyFingerprint(key: CryptoKey): Promise<string> {
  const raw = await crypto.subtle.exportKey("raw", key);
  const hash = await crypto.subtle.digest("SHA-256", raw);
  return toBase64(hash).slice(0, 32);
}

/** Encrypt plaintext; returns the wire payload stored in the document. */
export async function encryptContent(
  key: CryptoKey,
  plaintext: string
): Promise<EncryptedPayload> {
  const iv = crypto.getRandomValues(new Uint8Array(IV_LENGTH));
  const ct = await crypto.subtle.encrypt(
    { name: ALGO, iv },
    key,
    new TextEncoder().encode(plaintext)
  );
  return { iv: toBase64(iv), ciphertext: toBase64(ct), algorithm: "AES-256-GCM" };
}

/** Decrypt a wire payload back to plaintext. Throws on tamper/wrong key. */
export async function decryptContent(
  key: CryptoKey,
  payload: EncryptedPayload
): Promise<string> {
  const pt = await crypto.subtle.decrypt(
    { name: ALGO, iv: fromBase64(payload.iv) },
    key,
    fromBase64(payload.ciphertext)
  );
  return new TextDecoder().decode(pt);
}

/** Wrap (export) a key so it can be stored locally / backed up. */
export async function exportKey(key: CryptoKey): Promise<JsonWebKey> {
  return crypto.subtle.exportKey("jwk", key);
}

/** Re-import an exported JWK key. */
export async function importKey(jwk: JsonWebKey): Promise<CryptoKey> {
  return crypto.subtle.importKey("jwk", jwk, { name: ALGO }, true, [
    "encrypt",
    "decrypt",
  ]);
}

/** Wrap the document key with a passphrase (PBKDF2 -> AES-GCM key wrap). */
export async function wrapKeyWithPassphrase(
  key: CryptoKey,
  passphrase: string
): Promise<{ salt: string; iv: string; wrapped: string }> {
  const salt = crypto.getRandomValues(new Uint8Array(16));
  const iv = crypto.getRandomValues(new Uint8Array(IV_LENGTH));
  const base = await crypto.subtle.importKey(
    "raw",
    new TextEncoder().encode(passphrase),
    "PBKDF2",
    false,
    ["deriveKey"]
  );
  const wrapKey = await crypto.subtle.deriveKey(
    { name: "PBKDF2", salt, iterations: 310000, hash: "SHA-256" },
    base,
    { name: ALGO, length: KEY_LENGTH },
    false,
    ["encrypt"]
  );
  const raw = await crypto.subtle.exportKey("raw", key);
  const wrapped = await crypto.subtle.encrypt({ name: ALGO, iv }, wrapKey, raw);
  return { salt: toBase64(salt), iv: toBase64(iv), wrapped: toBase64(wrapped) };
}
