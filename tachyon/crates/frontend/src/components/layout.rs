#![allow(dead_code)]

use leptos::prelude::*;

#[component]
pub fn ResponsiveContainer(children: Children) -> impl IntoView {
    view! {
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            {children()}
        </div>
    }
}

#[component]
pub fn ResponsiveGrid(
    #[prop(default = 1)]
    sm_cols: u32,
    #[prop(default = 2)]
    md_cols: u32,
    #[prop(default = 3)]
    lg_cols: u32,
    #[prop(default = 4)]
    gap: u32,
    children: Children,
) -> impl IntoView {
    let sm = match sm_cols {
        1 => "grid-cols-1",
        2 => "grid-cols-2",
        3 => "grid-cols-3",
        4 => "grid-cols-4",
        _ => "grid-cols-1",
    };
    let md = match md_cols {
        1 => "md:grid-cols-1",
        2 => "md:grid-cols-2",
        3 => "md:grid-cols-3",
        4 => "md:grid-cols-4",
        5 => "md:grid-cols-5",
        6 => "md:grid-cols-6",
        _ => "md:grid-cols-2",
    };
    let lg = match lg_cols {
        1 => "lg:grid-cols-1",
        2 => "lg:grid-cols-2",
        3 => "lg:grid-cols-3",
        4 => "lg:grid-cols-4",
        5 => "lg:grid-cols-5",
        6 => "lg:grid-cols-6",
        _ => "lg:grid-cols-3",
    };

    view! {
        <div class=format!("grid {} {} {} gap-{}", sm, md, lg, gap)>
            {children()}
        </div>
    }
}

#[component]
pub fn MobileOnly(children: Children) -> impl IntoView {
    view! {
        <div class="block md:hidden">
            {children()}
        </div>
    }
}

#[component]
pub fn DesktopOnly(children: Children) -> impl IntoView {
    view! {
        <div class="hidden md:block">
            {children()}
        </div>
    }
}
