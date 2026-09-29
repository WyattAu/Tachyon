import { For, Show, createResource, createSignal } from "solid-js";
import { resolveBaseUrl, getToken } from "@/lib/api/client";

/** Flashcards — Solid port (Phase B). SRS review flow. Untyped endpoints. */

interface Card {
  id: string;
  front: string;
  back: string;
  [k: string]: unknown;
}

async function reviewQueue(): Promise<Card[]> {
  try {
    const res = await fetch(`${resolveBaseUrl()}/flashcards/review`, {
      headers: { Authorization: `Bearer ${getToken() ?? ""}` },
    });
    if (!res.ok) return [];
    const body = (await res.json()) as unknown as { cards?: Card[] } | Card[];
    return Array.isArray(body) ? body : (body.cards ?? []);
  } catch {
    return [];
  }
}

export default function Flashcards() {
  const [cards, { refetch }] = createResource(reviewQueue);
  const [index, setIndex] = createSignal(0);
  const [revealed, setRevealed] = createSignal(false);
  const [reviewed, setReviewed] = createSignal(0);

  const current = () => cards()?.[index()];

  const grade = async (quality: "again" | "good" | "easy") => {
    const card = current();
    if (!card) return;
    try {
      await fetch(`${resolveBaseUrl()}/flashcards/${card.id}/review`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${getToken() ?? ""}`,
        },
        body: JSON.stringify({ quality }),
      });
    } catch {
      // offline-tolerant: still advance the local session
    }
    setReviewed(reviewed() + 1);
    setRevealed(false);
    setIndex(index() + 1);
  };

  return (
    <div class="mx-auto max-w-3xl px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="font-display text-2xl font-bold text-gray-900 dark:text-white">Flashcards</h1>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        Spaced-repetition review — {reviewed()} reviewed this session
      </p>

      <Show when={!cards.loading} fallback={<div class="mt-6 h-48 animate-pulse rounded-none bg-gray-200 dark:bg-gray-700" />}>
        <Show
          when={current()}
          fallback={
            <div class="mt-6 rounded-none border border-gray-900 bg-white p-10 text-center dark:border-gray-100 dark:bg-gray-800">
              <p class="text-lg font-medium text-gray-700 dark:text-gray-300">
                Nothing due for review
              </p>
              <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                Create flashcards from your documents to build your deck.
              </p>
            </div>
          }
        >
          <div class="mt-6 rounded-none border-2 border-gray-900 bg-white p-8 dark:border-gray-100 dark:bg-gray-800">
            <p class="text-xs tracking-wider text-gray-400 uppercase">
              Card {index() + 1} of {cards()!.length}
            </p>
            <p class="mt-4 text-xl font-medium text-gray-900 dark:text-white">{current()!.front}</p>
            <Show
              when={revealed()}
              fallback={
                <button
                  class="mt-6 rounded-none bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-700"
                  onClick={() => setRevealed(true)}
                >
                  Reveal answer
                </button>
              }
            >
              <div class="mt-4 border-t border-gray-200 pt-4 dark:border-gray-700">
                <p class="text-gray-700 dark:text-gray-300">{current()!.back}</p>
              </div>
              <div class="mt-6 flex justify-center gap-2">
                <button class="rounded-none bg-red-600 px-4 py-2 text-sm text-white hover:bg-red-700" onClick={() => void grade("again")}>
                  Again
                </button>
                <button class="rounded-none bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-700" onClick={() => void grade("good")}>
                  Good
                </button>
                <button class="rounded-none bg-green-600 px-4 py-2 text-sm text-white hover:bg-green-700" onClick={() => void grade("easy")}>
                  Easy
                </button>
              </div>
            </Show>
          </div>
        </Show>
      </Show>
    </div>
  );
}
