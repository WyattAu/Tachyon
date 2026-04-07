// Global styles for the application
// NOTE: Tailwind CSS is loaded via CDN in index.html and handles all utility classes.
// This component only provides custom base styles and animations not covered by Tailwind.

use leptos::prelude::*;

/// Global styles component - injects custom CSS into the page
/// Tailwind CDN handles the utility class framework.
/// This file provides only custom animations, scrollbar styling,
/// and element resets that Tailwind's preflight doesn't cover.
#[component]
pub fn GlobalStyles() -> impl IntoView {
    view! {
        <style>
            {r#"
                /* Design system tokens */
                :root {{
                    --color-primary-50: #eff6ff;
                    --color-primary-100: #dbeafe;
                    --color-primary-200: #bfdbfe;
                    --color-primary-300: #93c5fd;
                    --color-primary-400: #60a5fa;
                    --color-primary-500: #3b82f6;
                    --color-primary-600: #2563eb;
                    --color-primary-700: #1d4ed8;
                    --color-primary-800: #1e40af;
                    --color-primary-900: #1e3a8a;
                    --color-primary-950: #172554;

                    --color-gray-50: #f9fafb;
                    --color-gray-100: #f3f4f6;
                    --color-gray-200: #e5e7eb;
                    --color-gray-300: #d1d5db;
                    --color-gray-400: #9ca3af;
                    --color-gray-500: #6b7280;
                    --color-gray-600: #4b5563;
                    --color-gray-700: #374151;
                    --color-gray-800: #1f2937;
                    --color-gray-900: #111827;
                    --color-gray-950: #030712;

                    --color-success: #10b981;
                    --color-warning: #f59e0b;
                    --color-error: #ef4444;
                    --color-info: #3b82f6;

                    --space-1: 0.25rem;
                    --space-2: 0.5rem;
                    --space-3: 0.75rem;
                    --space-4: 1rem;
                    --space-5: 1.25rem;
                    --space-6: 1.5rem;
                    --space-8: 2rem;
                    --space-10: 2.5rem;
                    --space-12: 3rem;
                    --space-16: 4rem;
                    --space-20: 5rem;
                    --space-24: 6rem;
                    --space-32: 8rem;

                    --text-xs: 0.75rem;
                    --text-sm: 0.875rem;
                    --text-base: 1rem;
                    --text-lg: 1.125rem;
                    --text-xl: 1.25rem;
                    --text-2xl: 1.5rem;
                    --text-3xl: 1.875rem;
                    --text-4xl: 2.25rem;

                    --radius-sm: 0.25rem;
                    --radius-md: 0.375rem;
                    --radius-lg: 0.5rem;
                    --radius-xl: 0.75rem;
                    --radius-2xl: 1rem;
                    --radius-full: 9999px;

                    --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
                    --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
                    --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1);

                    --transition-fast: 150ms ease;
                    --transition-normal: 200ms ease;
                    --transition-slow: 300ms ease;
                }}

                /* Custom animations beyond Tailwind defaults */
                @keyframes fade-in {
                    from { opacity: 0; transform: translateY(4px); }
                    to { opacity: 1; transform: translateY(0); }
                }

                @keyframes slide-in {
                    from { opacity: 0; transform: translateX(-8px); }
                    to { opacity: 1; transform: translateX(0); }
                }

                @keyframes scale-in {
                    from { opacity: 0; transform: scale(0.95); }
                    to { opacity: 1; transform: scale(1); }
                }

                .animate-fade-in {
                    animation: fade-in 0.2s ease-out;
                }

                .animate-slide-in {
                    animation: slide-in 0.2s ease-out;
                }

                .animate-scale-in {
                    animation: scale-in 0.15s ease-out;
                }

                /* Custom scrollbar styling */
                ::-webkit-scrollbar {
                    width: 8px;
                    height: 8px;
                }

                ::-webkit-scrollbar-track {
                    background: transparent;
                }

                ::-webkit-scrollbar-thumb {
                    background: #d1d5db;
                    border-radius: 4px;
                }

                ::-webkit-scrollbar-thumb:hover {
                    background: #9ca3af;
                }

                .dark ::-webkit-scrollbar-thumb {
                    background: #4b5563;
                }

                .dark ::-webkit-scrollbar-thumb:hover {
                    background: #6b7280;
                }

                /* Firefox scrollbar */
                * {
                    scrollbar-width: thin;
                    scrollbar-color: #d1d5db transparent;
                }

                .dark * {
                    scrollbar-color: #4b5563 transparent;
                }

                /* Smooth focus transitions */
                :focus-visible {
                    outline: 2px solid #2563eb;
                    outline-offset: 2px;
                    border-radius: 4px;
                }

                /* Prose typography for rendered markdown */
                .prose {
                    color: #374151;
                    line-height: 1.75;
                    max-width: none;
                }

                .dark .prose {
                    color: #d1d5db;
                }

                .prose h1, .prose h2, .prose h3,
                .prose h4, .prose h5, .prose h6 {
                    color: #111827;
                    font-weight: 600;
                    margin-top: 1.5em;
                    margin-bottom: 0.5em;
                }

                .dark .prose h1, .dark .prose h2, .dark .prose h3,
                .dark .prose h4, .dark .prose h5, .dark .prose h6 {
                    color: #f9fafb;
                }

                .prose p {
                    margin-top: 1.25em;
                    margin-bottom: 1.25em;
                }

                .prose a {
                    color: #2563eb;
                    text-decoration: underline;
                    text-underline-offset: 2px;
                }

                .prose a:hover {
                    color: #1d4ed8;
                }

                .dark .prose a {
                    color: #60a5fa;
                }

                .dark .prose a:hover {
                    color: #93bbfd;
                }

                .prose code {
                    color: #db2777;
                    background-color: #fdf2f8;
                    padding: 0.125rem 0.375rem;
                    border-radius: 0.25rem;
                    font-size: 0.875em;
                }

                .dark .prose code {
                    color: #f472b6;
                    background-color: #831843;
                }

                .prose pre {
                    background-color: #f3f4f6;
                    border-radius: 0.5rem;
                    padding: 1rem;
                    overflow-x: auto;
                    margin-top: 1.5em;
                    margin-bottom: 1.5em;
                }

                .dark .prose pre {
                    background-color: #1f2937;
                }

                .prose pre code {
                    background-color: transparent;
                    padding: 0;
                    color: inherit;
                }

                .prose blockquote {
                    border-left: 4px solid #e5e7eb;
                    padding-left: 1rem;
                    color: #6b7280;
                    font-style: italic;
                    margin-top: 1.5em;
                    margin-bottom: 1.5em;
                }

                .dark .prose blockquote {
                    border-left-color: #374151;
                    color: #9ca3af;
                }

                .prose table {
                    width: 100%;
                    border-collapse: collapse;
                    margin-top: 1.5em;
                    margin-bottom: 1.5em;
                }

                .prose th, .prose td {
                    border: 1px solid #e5e7eb;
                    padding: 0.5rem 0.75rem;
                    text-align: left;
                }

                .dark .prose th, .dark .prose td {
                    border-color: #374151;
                }

                .prose th {
                    background-color: #f9fafb;
                    font-weight: 600;
                }

                .dark .prose th {
                    background-color: #1f2937;
                }

                .prose ul, .prose ol {
                    padding-left: 1.5rem;
                    margin-top: 1.25em;
                    margin-bottom: 1.25em;
                }

                .prose ul {
                    list-style-type: disc;
                }

                .prose ol {
                    list-style-type: decimal;
                }

                .prose li {
                    margin-top: 0.5em;
                    margin-bottom: 0.5em;
                }

                .prose hr {
                    border: none;
                    border-top: 1px solid #e5e7eb;
                    margin-top: 2em;
                    margin-bottom: 2em;
                }

                .dark .prose hr {
                    border-top-color: #374151;
                }

                .prose img {
                    border-radius: 0.5rem;
                    margin-top: 1.5em;
                    margin-bottom: 1.5em;
                }

                /* Diff viewer styles for version history */
                .diff-added {
                    background-color: #dcfce7;
                    color: #166534;
                }

                .dark .diff-added {
                    background-color: #14532d;
                    color: #86efac;
                }

                .diff-removed {
                    background-color: #fee2e2;
                    color: #991b1b;
                }

                .dark .diff-removed {
                    background-color: #7f1d1d;
                    color: #fca5a5;
                }

                /* Selection styling */
                ::selection {
                    background-color: #bfdbfe;
                    color: #1e3a8a;
                }

                .dark ::selection {
                    background-color: #1e3a8a;
                    color: #bfdbfe;
                }
            "#}
        </style>
    }
}
