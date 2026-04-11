// Common Components
// Simple reusable UI components for Leptos 0.8

#![allow(dead_code)]

use leptos::prelude::*;

/// Simple button component
#[component]
pub fn Button(
    /// Button text
    text: String,
    /// Button variant (primary, secondary, danger)
    #[prop(default = "primary".into())]
    variant: String,
) -> impl IntoView {
    let variant_class = match variant.as_str() {
        "primary" => "bg-blue-600 hover:bg-blue-700 text-white",
        "secondary" => "bg-gray-200 hover:bg-gray-300 text-gray-700",
        "danger" => "bg-red-600 hover:bg-red-700 text-white",
        _ => "bg-gray-200 hover:bg-gray-300 text-gray-700",
    };

    view! {
        <button
            class=format!("px-4 py-2 rounded-md font-medium transition-colors {}", variant_class)
        >
            {text}
        </button>
    }
}

/// Simple card component
#[component]
pub fn Card(
    /// Card title
    title: String,
    /// Card content
    children: Children,
) -> impl IntoView {
    view! {
        <div class="bg-white rounded-lg shadow-md overflow-hidden">
            <div class="px-4 py-3 border-b border-gray-200">
                <h3 class="text-lg font-semibold text-gray-900">{title}</h3>
            </div>
            <div class="p-4">
                {children()}
            </div>
        </div>
    }
}

/// Loading spinner component
#[component]
pub fn LoadingSpinner() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center p-4">
            <div class="h-8 w-8 animate-spin rounded-full border-b-2 border-blue-600"></div>
        </div>
    }
}

/// Status badge component
#[component]
pub fn StatusBadge(
    /// Status text to display
    status: String,
    /// Badge color (green, yellow, red, gray, blue, purple)
    #[prop(default = "gray".into())]
    color: String,
) -> impl IntoView {
    let color_class = match color.as_str() {
        "green" => "bg-green-100 text-green-800",
        "yellow" => "bg-yellow-100 text-yellow-800",
        "red" => "bg-red-100 text-red-800",
        "blue" => "bg-blue-100 text-blue-800",
        "purple" => "bg-purple-100 text-purple-800",
        _ => "bg-gray-100 text-gray-800",
    };

    view! {
        <span class=format!("px-2 py-1 text-xs font-medium rounded-full {}", color_class)>
            {status}
        </span>
    }
}

/// Empty state component
#[component]
pub fn EmptyState(
    /// Title message
    title: String,
) -> impl IntoView {
    view! {
        <div class="text-center py-12">
            <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5m16 0h-5.586a1 1 0 00.707.293l1.414-1.414a1 1 0 01.414 0l2.293 2.293L15 13l-2.293 2.293z" />
            </svg>
            <h3 class="mt-2 text-sm font-medium text-gray-900">{title}</h3>
        </div>
    }
}

/// Grid layout component
#[component]
pub fn Grid(
    /// Number of columns (1-6)
    #[prop(default = 3)]
    cols: u32,
    /// Grid gap
    #[prop(default = 4)]
    gap: u32,
    /// Grid content
    children: Children,
) -> impl IntoView {
    let cols_class = match cols {
        1 => "grid-cols-1",
        2 => "grid-cols-2",
        3 => "grid-cols-3",
        4 => "grid-cols-4",
        5 => "grid-cols-5",
        6 => "grid-cols-6",
        _ => "grid-cols-3",
    };

    view! {
        <div class=format!("grid {} gap-{}", cols_class, gap)>
            {children()}
        </div>
    }
}

/// Page header component
#[component]
pub fn PageHeader(
    /// Page title
    title: String,
) -> impl IntoView {
    view! {
        <div class="mb-6">
            <h1 class="text-2xl font-bold text-gray-900">{title}</h1>
        </div>
    }
}
