import { For, Show, createResource, createSignal } from "solid-js";
import { useNavigate, useParams } from "@solidjs/router";
import { resolveBaseUrl, getToken } from "@/lib/api/client";

/** Blog — Solid port (Phase B). Public feed + post view + admin editor.
 * Blog endpoints are not yet in the OpenAPI spec — hand-typed. */

interface Post {
  slug: string;
  title: string;
  content: string;
  published?: boolean;
  created_at?: string;
}

async function blog<T>(path: string, fallback: T): Promise<T> {
  try {
    const res = await fetch(`${resolveBaseUrl()}${path}`, {
      headers: { Authorization: `Bearer ${getToken() ?? ""}` },
    });
    if (!res.ok) return fallback;
    const body = (await res.json()) as unknown as { posts?: Post[] };
    return (body.posts ?? []) as unknown as T;
  } catch {
    return fallback;
  }
}

export function BlogList() {
  const [posts] = createResource(async () => blog<Post[]>("/blog/posts", []));
  return (
    <div class="mx-auto max-w-4xl px-4 py-6 sm:px-6 lg:px-8">
      <div class="mb-4 flex items-center justify-between">
        <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Blog</h1>
        <A href="/blog/admin/new" class="rounded-none bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-700">
          New Post
        </A>
      </div>
      <Show when={!posts.loading} fallback={<div class="h-20 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />}>
        <Show
          when={(posts() ?? []).length > 0}
          fallback={<p class="text-sm text-gray-500 dark:text-gray-400">No posts published yet.</p>}
        >
          <div class="space-y-3">
            <For each={posts()}>
              {(post) => (
                <A
                  href={`/blog/${post.slug}`}
                  class="block rounded-none border border-gray-300 bg-white p-4 hover:shadow-spatial-sm dark:border-gray-600 dark:bg-gray-800"
                >
                  <p class="font-semibold text-gray-900 dark:text-white">{post.title}</p>
                  <Show when={post.created_at}>
                    <p class="mt-1 text-xs text-gray-400">
                      {new Date(post.created_at as string).toLocaleDateString()}
                    </p>
                  </Show>
                </A>
              )}
            </For>
          </div>
        </Show>
      </Show>
    </div>
  );
}

export function BlogPost() {
  const params = useParams();
  const [post] = createResource(async () => blog<Post | null>(`/blog/posts/${params.slug}`, null));
  return (
    <div class="mx-auto max-w-3xl px-4 py-6 sm:px-6 lg:px-8">
      <Show when={post()} fallback={<p class="text-sm text-gray-500">Post not found.</p>}>
        {(p) => (
          <article>
            <h1 class="font-display text-3xl font-bold text-gray-900 dark:text-white">{p().title}</h1>
            <div class="mt-4 whitespace-pre-wrap text-gray-700 dark:text-gray-300">{p().content}</div>
          </article>
        )}
      </Show>
    </div>
  );
}

export function BlogAdminNew() {
  const navigate = useNavigate();
  const [title, setTitle] = createSignal("");
  const [content, setContent] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");

  const publish = async () => {
    setBusy(true);
    setError("");
    try {
      const slug = title().toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/(^-|-$)/g, "");
      const res = await fetch(`${resolveBaseUrl()}/blog/posts`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${getToken() ?? ""}`,
        },
        body: JSON.stringify({ title: title(), slug, content: content(), published: true }),
      });
      if (!res.ok) throw new Error(`publish failed (${res.status})`);
      navigate(`/blog/${slug}`);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div class="mx-auto max-w-3xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">New Post</h1>
      <form
        class="mt-4 space-y-3"
        onSubmit={(e) => {
          e.preventDefault();
          void publish();
        }}
      >
        <input
          required
          placeholder="Post title"
          class="w-full rounded-none border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
          value={title()}
          onInput={(e) => setTitle(e.currentTarget.value)}
        />
        <textarea
          required
          rows={12}
          placeholder="Write your post (plain text)…"
          class="w-full rounded-none border border-gray-300 px-3 py-2 font-mono text-sm dark:border-gray-600 dark:bg-gray-800 dark:text-white"
          value={content()}
          onInput={(e) => setContent(e.currentTarget.value)}
        />
        <Show when={error()}>
          <p class="text-sm text-red-600">{error()}</p>
        </Show>
        <button
          type="submit"
          disabled={busy()}
          class="rounded-none bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
        >
          {busy() ? "Publishing…" : "Publish"}
        </button>
      </form>
    </div>
  );
}

import { A } from "@solidjs/router";
