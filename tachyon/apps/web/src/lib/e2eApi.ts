import { api, resolveBaseUrl, getToken } from "@/lib/api/client";

/**
 * E2E key metadata API (Phase C). Hand-typed: /e2e/keys endpoints are not
 * in the OpenAPI spec (tracked for server annotation cleanup). The server
 * stores METADATA ONLY — key material never leaves the client.
 */

export interface EncryptionKeyMeta {
  id: string;
  document_id: string;
  owner_id: string;
  key_algorithm: string;
  public_key_fingerprint: string;
  created_at: string;
}

export interface EncryptionStatus {
  document_id: string;
  encrypted: boolean;
  key_algorithm: string | null;
  key_fingerprint: string | null;
}

async function e2e<T>(path: string, init?: RequestInit, fallback?: T): Promise<T> {
  try {
    const res = await fetch(`${resolveBaseUrl()}${path}`, {
      ...init,
      headers: { Authorization: `Bearer ${getToken() ?? ""}`, ...(init?.headers ?? {}) },
    });
    if (!res.ok) return fallback as T;
    return (await res.json()) as unknown as T;
  } catch {
    return fallback as T;
  }
}

export const e2eApi = {
  registerKey: (documentId: string, keyAlgorithm: string, publicKeyFingerprint: string) =>
    e2e("/e2e/keys", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        document_id: documentId,
        key_algorithm: keyAlgorithm,
        public_key_fingerprint: publicKeyFingerprint,
      }),
    }),

  listKeys: () => e2e<{ keys: EncryptionKeyMeta[]; total: number }>("/e2e/keys", {}, { keys: [], total: 0 }),

  status: (documentId: string) =>
    e2e<EncryptionStatus>(`/e2e/status/${documentId}`, {}, { document_id: documentId, encrypted: false, key_algorithm: null, key_fingerprint: null }),

  deleteKey: (documentId: string) =>
    e2e(`/e2e/keys/${documentId}`, { method: "DELETE" }),
};
