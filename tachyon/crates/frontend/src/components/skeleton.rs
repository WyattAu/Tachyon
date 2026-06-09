#![allow(dead_code)]

use leptos::prelude::*;

#[component]
pub fn SkeletonText(#[prop(default = 3)] lines: usize) -> impl IntoView {
    view! {
        <div class="space-y-2 animate-pulse" aria-busy="true" role="status">
            {(0..lines).map(|i| {
                let width = if i == lines.saturating_sub(1) { "w-3/4" } else { "w-full" };
                view! {
                    <div class={format!("h-4 bg-gray-200 dark:bg-gray-700 rounded-none {}", width)}></div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
pub fn SkeletonCard() -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-200 dark:border-gray-700 p-6 animate-pulse" aria-busy="true" role="status">
            <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded-none w-3/4 mb-2"></div>
            <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded-none w-1/2 mb-4"></div>
            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded-none w-full mb-2"></div>
            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded-none w-5/6 mb-4"></div>
            <div class="flex gap-2">
                <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded-none w-12"></div>
                <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded-none w-16"></div>
            </div>
            <div class="flex justify-between mt-4">
                <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded-none w-20"></div>
                <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded-none w-20"></div>
            </div>
        </div>
    }
}

#[component]
pub fn SkeletonTable(
    #[prop(default = 5)] rows: usize,
    #[prop(default = 4)] cols: usize,
) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-200 dark:border-gray-700 overflow-hidden animate-pulse" aria-busy="true" role="status">
            <div class="border-b-2 border-gray-200 dark:border-gray-700 px-4 py-3 bg-gray-50 dark:bg-gray-800/50">
                <div class="flex gap-4">
                    {(0..cols).map(|_| {
                        view! {
                            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded-none flex-1"></div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
            {(0..rows).map(|_| {
                view! {
                    <div class="border-b border-gray-200 dark:border-gray-700 px-4 py-3 last:border-b-0">
                        <div class="flex gap-4">
                            {(0..cols).map(|_| {
                                view! {
                                    <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded-none flex-1"></div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
pub fn SkeletonEditor() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full animate-pulse" aria-busy="true" role="status">
            <div class="h-10 bg-gray-100 dark:bg-gray-800 border-b-2 border-gray-200 dark:border-gray-700 flex items-center px-4 gap-3">
                <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded-none w-40"></div>
                <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded-none w-6"></div>
            </div>
            <div class="flex-1 p-4 space-y-3">
                <div class="h-6 bg-gray-200 dark:bg-gray-700 rounded-none w-2/3"></div>
                <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded-none w-full"></div>
                <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded-none w-full"></div>
                <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded-none w-5/6"></div>
                <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded-none w-full"></div>
                <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded-none w-3/4"></div>
                <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded-none w-full"></div>
                <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded-none w-2/3"></div>
            </div>
            <div class="h-8 bg-gray-100 dark:bg-gray-800 border-t-2 border-gray-200 dark:border-gray-700 flex items-center justify-between px-4">
                <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded-none w-24"></div>
                <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded-none w-32"></div>
            </div>
        </div>
    }
}

#[component]
pub fn SkeletonDocumentList(#[prop(default = 5)] items: usize) -> impl IntoView {
    view! {
        <div>
            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded-none w-48 mb-4 animate-pulse"></div>
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {(0..items).map(|_| {
                    view! {
                        <SkeletonCard />
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
