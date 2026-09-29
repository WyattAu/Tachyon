import { For, Show, createResource, createSignal } from "solid-js";
import { api, resolveBaseUrl, getToken } from "@/lib/api/client";

/**
 * Billing — Solid port (Phase B). Plans + usage + subscription state.
 * Usage endpoint: GET /billing/usage/{org_id} (org "default" for single-user).
 * Subscriptions 404 = no active plan (correct semantics, verified live).
 */

interface Plan {
  name: string;
  price_monthly_cents: number;
  max_documents: number;
  max_members: number;
  features: string[];
}

interface Usage {
  documents_total: number;
  members_total: number;
  storage_bytes: number;
  plan: string;
}

const UNLIMITED = 1_000_000_000; // server Plan::UNLIMITED sentinel (JS-safe)
const fmtMax = (m: number) => (m >= UNLIMITED ? "∞" : String(m));
const fmtPrice = (c: number) => `£${(c / 100).toFixed(2)}/mo`;

export default function Billing() {
  const [error, setError] = createSignal<string | null>(null);

  const [plans] = createResource(async () => {
    const { data, error: e } = await api.GET("/billing/plans", {});
    if (e) throw new Error("plans fetch failed");
    return ((data as unknown as { plans?: Plan[] })?.plans ?? []) as Plan[];
  });

  const [usage] = createResource(async () => {
    // typed path exists; the response envelope {usage} is easier untyped here
    const res = await fetch(`${resolveBaseUrl()}/billing/usage/default`, {
      headers: { Authorization: `Bearer ${getToken() ?? ""}` },
    });
    if (!res.ok) return null;
    return ((await res.json()) as unknown as { usage?: Usage }).usage ?? null;
  });

  return (
    <div class="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">
        Billing &amp; Subscription
      </h1>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        Manage your subscription, view invoices, and track usage.
      </p>

      <Show when={error()}>
        <p class="mt-3 text-sm text-red-600 dark:text-red-400">{error()}</p>
      </Show>

      <div class="mt-6 grid gap-4 md:grid-cols-2">
        {/* Current plan */}
        <div class="rounded-none border border-gray-900 bg-white p-6 dark:border-gray-100 dark:bg-gray-800">
          <h2 class="mb-3 text-lg font-semibold">Current Plan</h2>
          <p class="text-sm text-gray-500 dark:text-gray-400">
            No active subscription. Choose a plan below.
          </p>
        </div>

        {/* Usage */}
        <div class="rounded-none border border-gray-900 bg-white p-6 dark:border-gray-100 dark:bg-gray-800">
          <h2 class="mb-3 text-lg font-semibold">Usage</h2>
          <Show
            when={usage()}
            fallback={<p class="text-sm text-gray-500 dark:text-gray-400">No usage data available.</p>}
          >
            {(u) => (
              <div class="space-y-3 text-sm">
                <div class="flex justify-between">
                  <span class="text-gray-600 dark:text-gray-400">Documents</span>
                  <span class="font-medium">
                    {u().documents_total}/{fmtMax((plans()?.find((p) => p.name.toLowerCase() === u().plan)?.max_documents ?? UNLIMITED))}
                  </span>
                </div>
                <div class="flex justify-between">
                  <span class="text-gray-600 dark:text-gray-400">Members</span>
                  <span class="font-medium">
                    {u().members_total}/{fmtMax(plans()?.find((p) => p.name.toLowerCase() === u().plan)?.max_members ?? UNLIMITED)}
                  </span>
                </div>
                <div class="flex justify-between">
                  <span class="text-gray-600 dark:text-gray-400">Storage</span>
                  <span class="font-medium">{(u().storage_bytes / 1024 / 1024).toFixed(1)} MB</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-gray-600 dark:text-gray-400">Plan</span>
                  <span class="font-medium capitalize">{u().plan}</span>
                </div>
              </div>
            )}
          </Show>
        </div>
      </div>

      {/* Plans */}
      <div class="mt-6 rounded-none border border-gray-900 bg-white p-6 dark:border-gray-100 dark:bg-gray-800">
        <div class="mb-4 flex gap-4 border-b border-gray-200 pb-2 text-sm dark:border-gray-700">
          <span class="border-b-2 border-blue-600 pb-2 font-medium text-blue-600">Plans</span>
          <span class="pb-2 text-gray-500">Invoices</span>
          <span class="pb-2 text-gray-500">Payment Method</span>
        </div>
        <Show when={!plans.loading} fallback={<div class="h-24 animate-pulse rounded bg-gray-200 dark:bg-gray-700" />}>
          <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
            <For each={plans()}>
              {(plan) => (
                <div class="rounded-none border border-gray-200 p-4 dark:border-gray-700">
                  <p class="font-semibold capitalize text-gray-900 dark:text-white">{plan.name}</p>
                  <p class="mt-1 text-xl font-bold">{fmtPrice(plan.price_monthly_cents)}</p>
                  <ul class="mt-3 space-y-1 text-xs text-gray-500 dark:text-gray-400">
                    <For each={plan.features}>{(f) => <li>✓ {f}</li>}</For>
                  </ul>
                  <button class="mt-3 w-full rounded-none bg-blue-600 py-1.5 text-sm text-white hover:bg-blue-700">
                    Choose {plan.name}
                  </button>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>
    </div>
  );
}
