// Global styles for the application

use leptos::prelude::*;

/// Global styles component - injects CSS into the page
#[component]
pub fn GlobalStyles() -> impl IntoView {
    view! {
        <style>
            {r#"
                /* Reset and base styles */
                *, *::before, *::after {
                    box-sizing: border-box;
                }
                
                body {
                    margin: 0;
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
                    -webkit-font-smoothing: antialiased;
                    -moz-osx-font-smoothing: grayscale;
                }
                
                /* Dark mode - applied to html element */
                html.dark {
                    background-color: #111827;
                    color: #f9fafb;
                }
                
                html.dark body {
                    background-color: #111827;
                    color: #f9fafb;
                }
                
                /* Light mode */
                html {
                    background-color: #f9fafb;
                    color: #111827;
                }
                
                /* Utility classes */
                .min-h-screen {
                    min-height: 100vh;
                }
                
                .flex {
                    display: flex;
                }
                
                .flex-col {
                    flex-direction: column;
                }
                
                .items-center {
                    align-items: center;
                }
                
                .justify-center {
                    justify-content: center;
                }
                
                .justify-between {
                    justify-content: space-between;
                }
                
                .space-y-1 > * + * {
                    margin-top: 0.25rem;
                }
                
                .space-y-4 > * + * {
                    margin-top: 1rem;
                }
                
                .space-x-4 > * + * {
                    margin-left: 1rem;
                }
                
                .fixed {
                    position: fixed;
                }
                
                .sticky {
                    position: sticky;
                }
                
                .inset-y-0 {
                    top: 0;
                    bottom: 0;
                }
                
                .left-0 {
                    left: 0;
                }
                
                .right-0 {
                    right: 0;
                }
                
                .top-0 {
                    top: 0;
                }
                
                .z-20 {
                    z-index: 20;
                }
                
                .z-30 {
                    z-index: 30;
                }
                
                .w-5 { width: 1.25rem; }
                .w-8 { width: 2rem; }
                .w-10 { width: 2.5rem; }
                .w-12 { width: 3rem; }
                .w-16 { width: 4rem; }
                .w-64 { width: 16rem; }
                .w-full { width: 100%; }
                
                .h-5 { height: 1.25rem; }
                .h-8 { height: 2rem; }
                .h-10 { height: 2.5rem; }
                .h-12 { height: 3rem; }
                .h-full { height: 100%; }
                
                .ml-3 { margin-left: 0.75rem; }
                .ml-4 { margin-left: 1rem; }
                .ml-16 { margin-left: 4rem; }
                
                .mt-1 { margin-top: 0.25rem; }
                .mt-2 { margin-top: 0.5rem; }
                .mt-4 { margin-top: 1rem; }
                .mt-8 { margin-top: 2rem; }
                .mb-2 { margin-bottom: 0.5rem; }
                .mb-3 { margin-bottom: 0.75rem; }
                .mb-4 { margin-bottom: 1rem; }
                .mb-8 { margin-bottom: 2rem; }
                
                .p-2 { padding: 0.5rem; }
                .p-3 { padding: 0.75rem; }
                .p-4 { padding: 1rem; }
                .p-6 { padding: 1.5rem; }
                .p-8 { padding: 2rem; }
                
                .px-4 { padding-left: 1rem; padding-right: 1rem; }
                .px-6 { padding-left: 1.5rem; padding-right: 1.5rem; }
                .py-3 { padding-top: 0.75rem; padding-bottom: 0.75rem; }
                .py-4 { padding-top: 1rem; padding-bottom: 1rem; }
                
                .overflow-y-auto { overflow-y: auto; }
                
                /* Colors - Light mode */
                .bg-gray-50 { background-color: #f9fafb; }
                .bg-gray-100 { background-color: #f3f4f6; }
                .bg-white { background-color: #ffffff; }
                .bg-blue-50 { background-color: #eff6ff; }
                .bg-blue-100 { background-color: #dbeafe; }
                .bg-blue-600 { background-color: #2563eb; }
                .bg-red-50 { background-color: #fef2f2; }
                
                /* Colors - Dark mode */
                .dark .bg-gray-800 { background-color: #1f2937; }
                .dark .bg-gray-900 { background-color: #111827; }
                .dark .bg-blue-900 { background-color: #1e3a8a; }
                
                /* Text colors - Light mode */
                .text-gray-400 { color: #9ca3af; }
                .text-gray-500 { color: #6b7280; }
                .text-gray-600 { color: #4b5563; }
                .text-gray-700 { color: #374151; }
                .text-gray-900 { color: #111827; }
                .text-white { color: #ffffff; }
                .text-blue-600 { color: #2563eb; }
                .text-red-700 { color: #b91c1c; }
                
                /* Text colors - Dark mode */
                .dark .text-gray-300 { color: #d1d5db; }
                .dark .text-gray-400 { color: #9ca3af; }
                .dark .text-white { color: #ffffff; }
                .dark .text-blue-400 { color: #60a5fa; }
                
                /* Border colors */
                .border { border-width: 1px; }
                .border-r { border-right-width: 1px; }
                .border-b { border-bottom-width: 1px; }
                .border-gray-200 { border-color: #e5e7eb; }
                .border-gray-700 { border-color: #374151; }
                .border-blue-500 { border-color: #3b82f6; }
                .border-red-200 { border-color: #fecaca; }
                
                .dark .border-gray-700 { border-color: #374151; }
                
                /* Border radius */
                .rounded { border-radius: 0.25rem; }
                .rounded-lg { border-radius: 0.5rem; }
                
                /* Typography */
                .text-sm { font-size: 0.875rem; }
                .text-lg { font-size: 1.125rem; }
                .text-xl { font-size: 1.25rem; }
                .text-2xl { font-size: 1.5rem; }
                .text-3xl { font-size: 1.875rem; }
                
                .font-medium { font-weight: 500; }
                .font-semibold { font-weight: 600; }
                .font-bold { font-weight: 700; }
                
                /* Transitions */
                .transition-all { transition: all 300ms; }
                .transition-colors { transition: color 150ms, background-color 150ms, border-color 150ms; }
                .duration-300 { transition-duration: 300ms; }
                
                /* Hover states */
                .hover\:bg-gray-100:hover { background-color: #f3f4f6; }
                .hover\:bg-gray-700:hover { background-color: #374151; }
                .hover\:bg-blue-700:hover { background-color: #1d4ed8; }
                .hover\:text-gray-300:hover { color: #d1d5db; }
                .hover\:text-gray-700:hover { color: #374151; }
                .hover\:underline:hover { text-decoration: underline; }
                .hover\:border-blue-500:hover { border-color: #3b82f6; }
                
                /* Button styles */
                button {
                    cursor: pointer;
                    border: none;
                    background: none;
                    font: inherit;
                }
                
                /* Link styles */
                a {
                    text-decoration: none;
                    color: inherit;
                }
                
                /* Input styles */
                input, textarea, select {
                    font: inherit;
                }
                
                /* Headings */
                h1, h2, h3, h4, h5, h6 {
                    margin: 0;
                }
                
                /* Grid */
                .grid { display: grid; }
                .grid-cols-1 { grid-template-columns: repeat(1, minmax(0, 1fr)); }
                .grid-cols-3 { grid-template-columns: repeat(3, minmax(0, 1fr)); }
                
                .gap-4 { gap: 1rem; }
                
                @media (min-width: 768px) {
                    .md\:grid-cols-2 { grid-template-columns: repeat(2, minmax(0, 1fr)); }
                    .md\:grid-cols-3 { grid-template-columns: repeat(3, minmax(0, 1fr)); }
                }
                
                @media (min-width: 1024px) {
                    .lg\:grid-cols-4 { grid-template-columns: repeat(4, minmax(0, 1fr)); }
                }
                
                /* Custom scrollbar */
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
                
                .dark ::-webkit-scrollbar-thumb {
                    background: #4b5563;
                }
                
                ::-webkit-scrollbar-thumb:hover {
                    background: #9ca3af;
                }
                
                .dark ::-webkit-scrollbar-thumb:hover {
                    background: #6b7280;
                }
                
                /* Animation */
                @keyframes pulse {
                    0%, 100% { opacity: 1; }
                    50% { opacity: .5; }
                }
                
                .animate-pulse {
                    animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
                }
                
                /* Shadow */
                .shadow {
                    box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1);
                }
                
                /* Block/Inline */
                .block { display: block; }
                .inline-block { display: inline-block; }
                
                /* Text align */
                .text-center { text-align: center; }
            "#}
        </style>
    }
}
