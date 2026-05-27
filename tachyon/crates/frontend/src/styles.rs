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

                    /* Amoebic design tokens */
                    --radius-amoebic: 60% 40% 30% 70% / 60% 30% 70% 40%;
                    --radius-amoebic-alt: 30% 70% 70% 30% / 30% 30% 70% 70%;
                    --radius-brutalist: 0px;
                    --radius-sharp: 2px;

                    /* Spatial depth */
                    --shadow-spatial-1: 2px 2px 0px 0px rgba(0, 0, 0, 0.15);
                    --shadow-spatial-2: 4px 4px 0px 0px rgba(0, 0, 0, 0.15);
                    --shadow-spatial-3: 8px 8px 0px 0px rgba(0, 0, 0, 0.12);

                    /* Spring animation */
                    --ease-spring: cubic-bezier(0.34, 1.56, 0.64, 1);
                    --ease-spatial: cubic-bezier(0.25, 0.46, 0.45, 0.94);
                }}

                .dark {{
                    --shadow-spatial-1: 2px 2px 0px 0px rgba(255, 255, 255, 0.08);
                    --shadow-spatial-2: 4px 4px 0px 0px rgba(255, 255, 255, 0.08);
                    --shadow-spatial-3: 8px 8px 0px 0px rgba(255, 255, 255, 0.06);
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

                /* ====================================================================
                   Native Editor Styles
                   ==================================================================== */

                .native-editor {{
                    position: relative;
                    overflow-y: auto;
                    overflow-x: hidden;
                    font-family: 'SF Mono', 'Cascadia Code', 'Fira Code', 'JetBrains Mono', 'Consolas', 'Monaco', monospace;
                    font-size: 14px;
                    line-height: 22px;
                    color: #1f2937;
                    background-color: #ffffff;
                    tab-size: 2;
                }}

                .dark .native-editor {{
                    color: #e5e7eb;
                    background-color: #111827;
                }}

                .native-editor:focus {{
                    outline: none;
                }}

                .editor-line {{
                    display: flex;
                    position: absolute;
                    left: 0;
                    right: 0;
                    white-space: pre;
                }}

                .editor-line.active-line {{
                    background: rgba(0, 0, 0, 0.04);
                }}

                .dark .editor-line.active-line {{
                    background: rgba(255, 255, 255, 0.04);
                }}

                .line-number {{
                    width: 50px;
                    min-width: 50px;
                    text-align: right;
                    padding-right: 12px;
                    color: #9ca3af;
                    user-select: none;
                    flex-shrink: 0;
                }}

                .dark .line-number {{
                    color: #4b5563;
                }}

                .line-content {{
                    flex: 1;
                    white-space: pre;
                    min-width: 0;
                }}

                .line-content.word-wrap {{
                    white-space: pre-wrap;
                    word-break: break-all;
                }}

                .editor-cursor {{
                    position: absolute;
                    width: 2px;
                    background: #1f2937;
                    animation: editor-cursor-blink 1s step-end infinite;
                    z-index: 10;
                    pointer-events: none;
                }}

                .dark .editor-cursor {{
                    background: #e5e7eb;
                }}

                @keyframes editor-cursor-blink {{
                    0%, 100% {{ opacity: 1; }}
                    50% {{ opacity: 0; }}
                }}

                .editor-selection {{
                    background: rgba(0, 120, 215, 0.2);
                    position: absolute;
                    z-index: 5;
                    pointer-events: none;
                    border-radius: 1px;
                }}

                .dark .editor-selection {{
                    background: rgba(96, 165, 250, 0.25);
                }}

                .editor-placeholder {{
                    position: absolute;
                    top: 0;
                    left: 62px;
                    color: #9ca3af;
                    pointer-events: none;
                    user-select: none;
                }}

                .dark .editor-placeholder {{
                    color: #4b5563;
                }}

                .editor-scroll-spacer {{
                    position: absolute;
                    top: 0;
                    left: 0;
                    width: 1px;
                    pointer-events: none;
                }}

                /* ====================================================================
                   Editor Highlight Token Styles
                   ==================================================================== */

                .ed-text {{ color: inherit; }}
                .ed-whitespace {{ color: transparent; }}

                .ed-h1 {{ font-weight: 700; font-size: 1.4em; color: #1e40af; }}
                .ed-h2 {{ font-weight: 700; font-size: 1.25em; color: #1e40af; }}
                .ed-h3 {{ font-weight: 700; font-size: 1.1em; color: #1e40af; }}
                .ed-h4 {{ font-weight: 700; font-size: 1em; color: #374151; }}
                .ed-h5 {{ font-weight: 700; font-size: 0.95em; color: #374151; }}
                .ed-h6 {{ font-weight: 700; font-size: 0.9em; color: #6b7280; }}

                .dark .ed-h1 {{ color: #60a5fa; }}
                .dark .ed-h2 {{ color: #60a5fa; }}
                .dark .ed-h3 {{ color: #93c5fd; }}
                .dark .ed-h4 {{ color: #d1d5db; }}
                .dark .ed-h5 {{ color: #d1d5db; }}
                .dark .ed-h6 {{ color: #9ca3af; }}

                .ed-bold {{ font-weight: 700; }}
                .ed-italic {{ font-style: italic; }}
                .ed-bold-italic {{ font-weight: 700; font-style: italic; }}
                .ed-strikethrough {{ text-decoration: line-through; color: #6b7280; }}
                .dark .ed-strikethrough {{ color: #9ca3af; }}

                .ed-code-inline {{
                    font-family: 'SF Mono', 'Cascadia Code', 'Fira Code', monospace;
                    background: rgba(0, 0, 0, 0.06);
                    padding: 1px 4px;
                    border-radius: 3px;
                    font-size: 0.9em;
                    color: #be185d;
                }}
                .dark .ed-code-inline {{
                    background: rgba(255, 255, 255, 0.08);
                    color: #f472b6;
                }}

                .ed-code-block {{
                    font-family: 'SF Mono', 'Cascadia Code', 'Fira Code', monospace;
                    background: #f3f4f6;
                    color: #374151;
                }}
                .dark .ed-code-block {{
                    background: #1f2937;
                    color: #d1d5db;
                }}

                .ed-link {{ color: #2563eb; text-decoration: underline; }}
                .dark .ed-link {{ color: #60a5fa; }}
                .ed-link-url {{ color: #6b7280; font-style: italic; }}
                .dark .ed-link-url {{ color: #9ca3af; }}
                .ed-link-text {{ color: #2563eb; text-decoration: underline; }}
                .dark .ed-link-text {{ color: #60a5fa; }}

                .ed-image {{ color: #059669; }}
                .ed-image-url {{ color: #6b7280; font-style: italic; }}
                .ed-image-alt {{ color: #059669; }}

                .ed-wiki-link {{ color: #7c3aed; text-decoration: underline; }}
                .dark .ed-wiki-link {{ color: #a78bfa; }}

                .ed-blockquote {{
                    border-left: 3px solid #d1d5db;
                    padding-left: 8px;
                    color: #6b7280;
                }}
                .dark .ed-blockquote {{
                    border-left-color: #4b5563;
                    color: #9ca3af;
                }}

                .ed-list-marker {{ color: #059669; font-weight: 600; }}
                .dark .ed-list-marker {{ color: #34d399; }}

                .ed-task-marker {{ color: #059669; font-weight: 600; }}
                .dark .ed-task-marker {{ color: #34d399; }}

                .ed-list-item {{ color: inherit; }}

                .ed-hr {{ color: #d1d5db; }}
                .dark .ed-hr {{ color: #4b5563; }}

                .ed-table-header {{ font-weight: 700; color: #374151; }}
                .dark .ed-table-header {{ color: #d1d5db; }}
                .ed-table-cell {{ color: inherit; }}
                .ed-table-border {{ color: #d1d5db; }}
                .dark .ed-table-border {{ color: #4b5563; }}

                .ed-frontmatter {{ color: #6b7280; font-style: italic; }}
                .dark .ed-frontmatter {{ color: #9ca3af; }}

                .ed-tag {{ color: #2563eb; }}
                .dark .ed-tag {{ color: #60a5fa; }}

                /* ====================================================================
                   Editor Toolbar Styles
                   ==================================================================== */

                .editor-toolbar {{
                    display: flex;
                    align-items: center;
                    gap: 2px;
                    padding: 4px 8px;
                    border-bottom: 1px solid #e5e7eb;
                    background: #f9fafb;
                    overflow-x: auto;
                    flex-shrink: 0;
                    flex-wrap: wrap;
                }}

                .dark .editor-toolbar {{
                    border-bottom-color: #374151;
                    background: #1f2937;
                }}

                .editor-toolbar-btn {{
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    min-width: 28px;
                    height: 28px;
                    padding: 2px 6px;
                    border: none;
                    border-radius: 4px;
                    background: transparent;
                    color: #374151;
                    font-size: 12px;
                    cursor: pointer;
                    transition: background-color 100ms ease;
                    white-space: nowrap;
                }}

                @media (max-width: 767px) {{
                    .editor-toolbar-btn {{
                        min-width: 44px;
                        min-height: 44px;
                        height: 44px;
                        padding: 4px 8px;
                        font-size: 14px;
                    }}
                }}

                .editor-toolbar-btn:hover:not(:disabled) {{
                    background: #e5e7eb;
                }}

                .dark .editor-toolbar-btn {{
                    color: #d1d5db;
                }}

                .dark .editor-toolbar-btn:hover:not(:disabled) {{
                    background: #374151;
                }}

                .editor-toolbar-btn:disabled {{
                    opacity: 0.4;
                    cursor: not-allowed;
                }}

                .editor-toolbar-sep {{
                    width: 1px;
                    height: 20px;
                    background: #d1d5db;
                    margin: 0 4px;
                    flex-shrink: 0;
                }}

                .dark .editor-toolbar-sep {{
                    background: #4b5563;
                }}

                /* ====================================================================
                   Editor Search Styles
                   ==================================================================== */

                .editor-search {{
                    position: absolute;
                    top: 8px;
                    right: 8px;
                    background: white;
                    border: 1px solid #d1d5db;
                    border-radius: 8px;
                    padding: 10px;
                    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
                    z-index: 50;
                    min-width: 340px;
                }}

                .dark .editor-search {{
                    background: #1f2937;
                    border-color: #4b5563;
                    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
                }}

                .editor-search-input {{
                    flex: 1;
                    padding: 4px 8px;
                    border: 1px solid #d1d5db;
                    border-radius: 4px;
                    font-size: 13px;
                    outline: none;
                    min-width: 0;
                }}

                .dark .editor-search-input {{
                    background: #111827;
                    border-color: #4b5563;
                    color: #e5e7eb;
                }}

                .editor-search-input:focus {{
                    border-color: #2563eb;
                    box-shadow: 0 0 0 2px rgba(37, 99, 235, 0.2);
                }}

                .editor-search-count {{
                    font-size: 11px;
                    color: #6b7280;
                    white-space: nowrap;
                    min-width: 60px;
                    text-align: center;
                }}

                .dark .editor-search-count {{
                    color: #9ca3af;
                }}

                .editor-search-btn {{
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    width: 26px;
                    height: 26px;
                    border: 1px solid #d1d5db;
                    border-radius: 4px;
                    background: transparent;
                    cursor: pointer;
                    font-size: 12px;
                    color: #374151;
                }}

                .dark .editor-search-btn {{
                    border-color: #4b5563;
                    color: #d1d5db;
                }}

                .editor-search-btn:hover {{
                    background: #f3f4f6;
                }}

                .dark .editor-search-btn:hover {{
                    background: #374151;
                }}

                .editor-search-toggle {{
                    display: flex;
                    align-items: center;
                    gap: 2px;
                    font-size: 11px;
                    color: #6b7280;
                    cursor: pointer;
                    user-select: none;
                }}

                .dark .editor-search-toggle {{
                    color: #9ca3af;
                }}

                .editor-search-toggle input[type="checkbox"] {{
                    margin: 0;
                    width: 14px;
                    height: 14px;
                    cursor: pointer;
                }}

                .editor-search-action-btn {{
                    padding: 4px 10px;
                    border: 1px solid #d1d5db;
                    border-radius: 4px;
                    background: transparent;
                    cursor: pointer;
                    font-size: 12px;
                    color: #374151;
                    white-space: nowrap;
                }}

                .dark .editor-search-action-btn {{
                    border-color: #4b5563;
                    color: #d1d5db;
                }}

                .editor-search-action-btn:hover {{
                    background: #f3f4f6;
                }}

                .dark .editor-search-action-btn:hover {{
                    background: #374151;
                }}

                /* ====================================================================
                   Editor Preview Styles
                   ==================================================================== */

                .editor-preview {{
                    flex: 1;
                    overflow-y: auto;
                    padding: 24px;
                    background: white;
                }}

                .dark .editor-preview {{
                    background: #111827;
                }}

                /* ====================================================================
                   Editor Split View Styles
                   ==================================================================== */

                .editor-split-container {{
                    display: flex;
                    flex-direction: column;
                    height: 100%;
                }}

                .editor-split-controls {{
                    display: flex;
                    align-items: center;
                    gap: 2px;
                    padding: 4px 8px;
                    border-bottom: 1px solid #e5e7eb;
                    background: #f9fafb;
                }}

                .dark .editor-split-controls {{
                    border-bottom-color: #374151;
                    background: #1f2937;
                }}

                .editor-split-btn {{
                    padding: 4px 10px;
                    border: 1px solid transparent;
                    border-radius: 4px;
                    background: transparent;
                    font-size: 12px;
                    color: #6b7280;
                    cursor: pointer;
                    transition: all 100ms ease;
                }}

                .editor-split-btn:hover {{
                    background: #e5e7eb;
                    color: #374151;
                }}

                .editor-split-btn.active {{
                    background: #dbeafe;
                    color: #1d4ed8;
                    border-color: #93c5fd;
                }}

                .dark .editor-split-btn {{
                    color: #9ca3af;
                }}

                .dark .editor-split-btn:hover {{
                    background: #374151;
                    color: #d1d5db;
                }}

                .dark .editor-split-btn.active {{
                    background: #1e3a8a;
                    color: #60a5fa;
                    border-color: #2563eb;
                }}

                .editor-split-content {{
                    display: flex;
                    flex: 1;
                    overflow: hidden;
                }}

                .editor-split-content.split-edit .editor-pane {{
                    flex: 1;
                }}

                .editor-split-content.split-preview .editor-pane {{
                    flex: 1;
                }}

                .editor-split-content.split-both .editor-pane {{
                    flex: 1;
                    min-width: 0;
                }}

                .editor-split-divider {{
                    width: 1px;
                    background: #e5e7eb;
                    flex-shrink: 0;
                }}

                .dark .editor-split-divider {{
                    background: #374151;
                }}

                .editor-pane {{
                    display: flex;
                    flex-direction: column;
                    overflow: hidden;
                }}

                .editor-pane-full {{
                    width: 100%;
                }}

                .editor-pane-left {{
                    border-right: none;
                }}

                .editor-pane-right {{
                    border-left: none;
                }}
            "#}
        </style>
    }
}
