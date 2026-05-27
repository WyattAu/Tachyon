#![allow(dead_code, clippy::redundant_locals)]

use crate::api::ApiClient;
use crate::types::*;
use leptos::prelude::*;
use leptos::task::spawn_local;

async fn fetch_plans() -> Result<BillingPlansResponse, String> {
    let client = ApiClient::default();
    client.get_billing_plans().await.map_err(|e| e.to_string())
}

async fn fetch_subscription(org_id: &str) -> Result<SubscriptionResponse, String> {
    let client = ApiClient::default();
    client
        .get_subscription(org_id)
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_invoices(org_id: &str) -> Result<InvoicesResponse, String> {
    let client = ApiClient::default();
    client.get_invoices(org_id).await.map_err(|e| e.to_string())
}

async fn fetch_usage(org_id: &str) -> Result<UsageResponse, String> {
    let client = ApiClient::default();
    client.get_usage(org_id).await.map_err(|e| e.to_string())
}

async fn do_cancel_subscription(org_id: &str) -> Result<(), String> {
    let client = ApiClient::default();
    client
        .cancel_subscription(org_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn do_create_subscription(org_id: &str, plan: &str) -> Result<SubscriptionResponse, String> {
    let client = ApiClient::default();
    client
        .create_subscription(org_id, plan)
        .await
        .map_err(|e| e.to_string())
}

fn format_cents(cents: u64) -> String {
    let pounds = cents as f64 / 100.0;
    format!("\u{00a3}{:.2}/mo", pounds)
}

fn format_storage_bytes(bytes: u64) -> String {
    let mb = bytes as f64 / (1024.0 * 1024.0);
    if mb < 1024.0 {
        format!("{:.1} MB", mb)
    } else {
        format!("{:.1} GB", mb / 1024.0)
    }
}

#[component]
pub fn BillingPage() -> impl IntoView {
    let (plans_r, set_plans) = signal(Vec::<BillingPlanInfo>::new());
    let (subscription, set_subscription) = signal(None::<SubscriptionResponse>);
    let (invoices, set_invoices) = signal(Vec::<InvoiceInfo>::new());
    let (usage, set_usage) = signal(None::<UsageResponse>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (success_msg, set_success_msg) = signal(None::<String>);
    let (show_cancel_confirm, set_show_cancel_confirm) = signal(false);
    let (active_tab, set_active_tab) = signal(0u8);

    let plans = plans_r;

    let load_data = {
        let set_plans = set_plans;
        let set_subscription = set_subscription;
        let set_invoices = set_invoices;
        let set_usage = set_usage;
        let set_loading = set_loading;
        let set_error = set_error;
        move || {
            let set_plans = set_plans;
            let set_subscription = set_subscription;
            let set_invoices = set_invoices;
            let set_usage = set_usage;
            let set_loading = set_loading;
            let set_error = set_error;
            spawn_local(async move {
                set_loading.set(true);
                set_error.set(None);

                if let Ok(p) = fetch_plans().await {
                    set_plans.set(p.plans);
                }

                if let Ok(sub) = fetch_subscription("default").await {
                    set_subscription.set(Some(sub));
                }

                if let Ok(inv) = fetch_invoices("default").await {
                    set_invoices.set(inv.invoices);
                }

                if let Ok(u) = fetch_usage("default").await {
                    set_usage.set(Some(u));
                }

                set_loading.set(false);
            });
        }
    };

    Effect::new(move |_| {
        load_data();
    });

    let handle_cancel = {
        let load_data = load_data;
        move |_: leptos::ev::MouseEvent| {
            let load_data = load_data;
            spawn_local(async move {
                match do_cancel_subscription("default").await {
                    Ok(()) => {
                        set_show_cancel_confirm.set(false);
                        set_success_msg.set(Some("Subscription cancelled. You can continue using the service until the end of the billing period.".to_string()));
                        load_data();
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    let handle_subscribe = {
        let load_data = load_data;
        move |plan_name: String| {
            let load_data = load_data;
            spawn_local(async move {
                set_error.set(None);
                match do_create_subscription("default", &plan_name).await {
                    Ok(_) => {
                        set_success_msg.set(Some(format!(
                            "Successfully subscribed to {} plan!",
                            plan_name
                        )));
                        load_data();
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    view! {
        <div class="p-4 md:p-6 max-w-6xl mx-auto">
            <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center mb-6">
                <div>
                    <h1 class="text-xl sm:text-2xl font-bold text-gray-900 dark:text-white">"Billing & Subscription"</h1>
                    <p class="text-gray-600 dark:text-gray-400 mt-1">"Manage your subscription, view invoices, and track usage."</p>
                </div>
            </div>

            {move || error.get().map(|e| view! {
                <div class="mb-4 p-4 bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-200 rounded-none">
                    {e}
                </div>
            })}

            {move || success_msg.get().map(|msg| view! {
                <div class="mb-4 p-4 bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-200 rounded-none">
                    {msg}
                    <button class="ml-2 underline" on:click=move |_| set_success_msg.set(None)>"Dismiss"</button>
                </div>
            })}

            {move || if loading.get() {
                Some(view! {
                    <div class="flex justify-center items-center py-12">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    </div>
                }.into_any())
            } else {
                None
            }}

            // Current Plan & Usage
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
                // Current plan card
                <div class="bg-white dark:bg-gray-800 shadow rounded-none p-6 border border-gray-900 dark:border-gray-100">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Current Plan"</h2>
                    {move || subscription.get().map(|sub| {
                        let cancel_at_end = sub.subscription.cancel_at_period_end;
                        let plan_name = sub.plan_details.name.clone();
                        let sub_status = sub.subscription.status.clone();
                        let period_start = sub.subscription.current_period_start.split('T').next().unwrap_or("-").to_string();
                        let period_end = sub.subscription.current_period_end.split('T').next().unwrap_or("-").to_string();
                        let price_cents = sub.plan_details.price_monthly_cents;
                        view! {
                            <div>
                                <div class="flex items-center justify-between mb-4">
                                    <div>
                                        <span class="text-2xl font-bold text-gray-900 dark:text-white capitalize">{plan_name.clone()}</span>
                                        <p class="text-gray-500 dark:text-gray-400 text-sm mt-1">
                                            {format_cents(price_cents)}
                                        </p>
                                    </div>
                                    <span class={
                                        format!("px-3 py-1 rounded-full text-xs font-medium {}",
                                            if sub_status == "active" { "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200" }
                                            else if sub_status == "cancelled" { "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200" }
                                            else { "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300" }
                                        )
                                    }>
                                        {sub_status.clone()}
                                    </span>
                                </div>
                                {cancel_at_end.then(|| view! {
                                    <div class="mb-4 p-3 bg-yellow-50 dark:bg-yellow-900/30 border-2 border-yellow-200 dark:border-yellow-800 rounded-none">
                                        <p class="text-sm text-yellow-800 dark:text-yellow-200">"Your subscription will be cancelled at the end of the current billing period."</p>
                                    </div>
                                })}
                                <div class="space-y-2 text-sm">
                                    <div class="flex justify-between text-gray-600 dark:text-gray-400">
                                        <span>"Period start"</span>
                                        <span class="text-gray-900 dark:text-white">{period_start}</span>
                                    </div>
                                    <div class="flex justify-between text-gray-600 dark:text-gray-400">
                                        <span>"Period end"</span>
                                        <span class="text-gray-900 dark:text-white">{period_end}</span>
                                    </div>
                                </div>
                                {(!cancel_at_end && plan_name != "free").then(|| view! {
                                    <div class="mt-4">
                                        <button
                                            class="text-sm text-red-600 dark:text-red-400 hover:underline"
                                            on:click=move |_| set_show_cancel_confirm.set(true)
                                        >
                                            "Cancel subscription"
                                        </button>
                                    </div>
                                })}
                            </div>
                        }
                    }).map(|v| v.into_any()).unwrap_or_else(|| view! {
                        <div class="text-center py-6">
                            <p class="text-gray-500 dark:text-gray-400">"No active subscription. Choose a plan below."</p>
                        </div>
                    }.into_any())}
                </div>

                // Usage stats card
                <div class="bg-white dark:bg-gray-800 shadow rounded-none p-6 border border-gray-900 dark:border-gray-100">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Usage"</h2>
                    {move || usage.get().map(|u| {
                        let u = u.usage;
                        let plans_snapshot = plans.get();
                        let plan = plans_snapshot.iter().find(|p| p.name == u.plan);
                        let max_docs = plan.map(|p| p.max_documents).unwrap_or(usize::MAX);
                        let max_members = plan.map(|p| p.max_members).unwrap_or(usize::MAX);
                        let doc_pct = if max_docs > 0 { (u.documents_total as f64 / max_docs as f64 * 100.0).min(100.0) } else { 0.0 };
                        let member_pct = if max_members > 0 { (u.members_total as f64 / max_members as f64 * 100.0).min(100.0) } else { 0.0 };
                        view! {
                            <div class="space-y-5">
                                <div>
                                    <div class="flex justify-between text-sm mb-1">
                                        <span class="text-gray-600 dark:text-gray-400">"Documents"</span>
                                        <span class="text-gray-900 dark:text-white">{u.documents_total}"/"{max_docs}</span>
                                    </div>
                                    <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                                        <div class="bg-blue-600 h-2 rounded-full transition-all" style={format!("width: {:.1}%", doc_pct)}></div>
                                    </div>
                                </div>
                                <div>
                                    <div class="flex justify-between text-sm mb-1">
                                        <span class="text-gray-600 dark:text-gray-400">"Members"</span>
                                        <span class="text-gray-900 dark:text-white">{u.members_total}"/"{max_members}</span>
                                    </div>
                                    <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                                        <div class="bg-green-600 h-2 rounded-full transition-all" style={format!("width: {:.1}%", member_pct)}></div>
                                    </div>
                                </div>
                                <div class="flex justify-between text-sm">
                                    <span class="text-gray-600 dark:text-gray-400">"Storage"</span>
                                    <span class="text-gray-900 dark:text-white">{format_storage_bytes(u.storage_bytes)}</span>
                                </div>
                                <div class="flex justify-between text-sm">
                                    <span class="text-gray-600 dark:text-gray-400">"Plan"</span>
                                    <span class="text-gray-900 dark:text-white capitalize">{u.plan.clone()}</span>
                                </div>
                            </div>
                        }
                    }.into_any()).unwrap_or_else(|| view! {
                        <p class="text-gray-500 dark:text-gray-400 text-center py-6">"No usage data available."</p>
                    }.into_any())}
                </div>
            </div>

            // Tabs
            <div class="bg-white dark:bg-gray-800 shadow rounded-none border border-gray-900 dark:border-gray-100">
                <div class="border-b border-gray-200 dark:border-gray-700">
                    <nav class="flex -mb-px">
                        <button
                            class:px-6=true
                            class:py-3=true
                            class:text-sm=true
                            class:font-medium=true
                            class:border-b-2=true
                            class:transition-colors=true
                            class:border-blue-500=move || active_tab.get() == 0
                            class:text-blue-600=move || active_tab.get() == 0
                            class:dark:text-blue-400=move || active_tab.get() == 0
                            class:border-transparent=move || active_tab.get() != 0
                            class:text-gray-500=move || active_tab.get() != 0
                            class:hover:text-gray-700=move || active_tab.get() != 0
                            class:dark:text-gray-400=move || active_tab.get() != 0
                            class:dark:hover:text-gray-200=move || active_tab.get() != 0
                            on:click=move |_| set_active_tab.set(0)
                        >"Plans"</button>
                        <button
                            class:px-6=true
                            class:py-3=true
                            class:text-sm=true
                            class:font-medium=true
                            class:border-b-2=true
                            class:transition-colors=true
                            class:border-blue-500=move || active_tab.get() == 1
                            class:text-blue-600=move || active_tab.get() == 1
                            class:dark:text-blue-400=move || active_tab.get() == 1
                            class:border-transparent=move || active_tab.get() != 1
                            class:text-gray-500=move || active_tab.get() != 1
                            class:hover:text-gray-700=move || active_tab.get() != 1
                            class:dark:text-gray-400=move || active_tab.get() != 1
                            class:dark:hover:text-gray-200=move || active_tab.get() != 1
                            on:click=move |_| set_active_tab.set(1)
                        >"Invoices"</button>
                        <button
                            class:px-6=true
                            class:py-3=true
                            class:text-sm=true
                            class:font-medium=true
                            class:border-b-2=true
                            class:transition-colors=true
                            class:border-blue-500=move || active_tab.get() == 2
                            class:text-blue-600=move || active_tab.get() == 2
                            class:dark:text-blue-400=move || active_tab.get() == 2
                            class:border-transparent=move || active_tab.get() != 2
                            class:text-gray-500=move || active_tab.get() != 2
                            class:hover:text-gray-700=move || active_tab.get() != 2
                            class:dark:text-gray-400=move || active_tab.get() != 2
                            class:dark:hover:text-gray-200=move || active_tab.get() != 2
                            on:click=move |_| set_active_tab.set(2)
                        >"Payment Method"</button>
                    </nav>
                </div>

                <div class="p-6">
                    // Plans tab
                    {move || if active_tab.get() == 0 {
                        Some(view! {
                            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                                {plans.get().into_iter().map(|plan| {
                                    let is_current = subscription.get().as_ref().map(|s| s.plan_details.name == plan.name).unwrap_or(false);
                                    let plan_name = plan.name.clone();
                                    let handle_subscribe = handle_subscribe;
                                    view! {
                                        <div class={
                                            format!("rounded-none border-2 border-gray-900 dark:border-gray-100 p-5 {}",
                                                if is_current { "border-blue-500 bg-blue-50 dark:bg-blue-900/20" } else { "border-gray-200 dark:border-gray-700" }
                                            )
                                        }>
                                            <h3 class="text-lg font-semibold capitalize text-gray-900 dark:text-white">{plan.name.clone()}</h3>
                                            <p class="text-2xl font-bold mt-2 text-gray-900 dark:text-white">
                                                {format_cents(plan.price_monthly_cents)}
                                            </p>
                                            <ul class="mt-4 space-y-2">
                                                {plan.features.iter().map(|f| view! {
                                                    <li class="flex items-center text-sm text-gray-600 dark:text-gray-400">
                                                        <svg class="w-4 h-4 mr-2 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                                        </svg>
                                                        {f.clone()}
                                                    </li>
                                                }).collect::<Vec<_>>()}
                                            </ul>
                                            <div class="mt-4">
                                                {if is_current {
                                                    view! {
                                                        <span class="block w-full text-center py-2 px-4 rounded-none text-sm font-medium bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300">
                                                            "Current plan"
                                                        </span>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <button
                                                            class="w-full py-2 px-4 rounded-none text-sm font-medium bg-blue-600 text-white hover:bg-blue-700 transition-colors"
                                                            on:click=move |_| handle_subscribe(plan_name.clone())
                                                        >
                                                            {if plan.price_monthly_cents == 0 { "Downgrade" } else { "Upgrade" }}
                                                        </button>
                                                    }.into_any()
                                                }}
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any())
                    } else {
                        None
                    }}

                    // Invoices tab
                    {move || if active_tab.get() == 1 {
                        Some(view! {
                            {move || {
                                let invs = invoices.get();
                                if invs.is_empty() {
                                    view! {
                                        <p class="text-gray-500 dark:text-gray-400 text-center py-8">"No invoices yet."</p>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="overflow-x-auto">
                                            <table class="w-full text-sm">
                                                <thead>
                                                    <tr class="border-b border-gray-200 dark:border-gray-700">
                                                        <th class="text-left py-3 px-4 font-medium text-gray-500 dark:text-gray-400">"Date"</th>
                                                        <th class="text-left py-3 px-4 font-medium text-gray-500 dark:text-gray-400">"Description"</th>
                                                        <th class="text-left py-3 px-4 font-medium text-gray-500 dark:text-gray-400">"Amount"</th>
                                                        <th class="text-left py-3 px-4 font-medium text-gray-500 dark:text-gray-400">"Status"</th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
                                                    {invs.into_iter().map(|inv| {
                                                        let amt = inv.amount_cents as f64 / 100.0;
                                                        let inv_date = inv.created_at.split('T').next().unwrap_or("-").to_string();
                                                        let inv_desc = inv.description.clone();
                                                        let inv_status = inv.status.clone();
                                                        let inv_status_class = if inv_status == "paid" { "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200" } else { "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200" };
                                                        view! {
                                                            <tr class="hover:bg-gray-50 dark:hover:bg-gray-700/50">
                                                                <td class="py-3 px-4 text-gray-900 dark:text-white">{inv_date}</td>
                                                                <td class="py-3 px-4 text-gray-600 dark:text-gray-400">{inv_desc}</td>
                                                                <td class="py-3 px-4 text-gray-900 dark:text-white">{format!("\u{00a3}{:.2}", amt)}</td>
                                                                <td class="py-3 px-4">
                                                                    <span class={inv_status_class}>
                                                                        {inv_status}
                                                                    </span>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_any()
                                }
                            }}
                        }.into_any())
                    } else {
                        None
                    }}

                    // Payment method tab
                    {move || if active_tab.get() == 2 {
                        Some(view! {
                            <div class="text-center py-8">
                                {move || subscription.get().as_ref().and_then(|s| s.subscription.payment_method_id.as_ref()).map(|pm_id| {
                                    let pm_id = pm_id.clone();
                                    view! {
                                        <div>
                                            <div class="w-16 h-16 mx-auto bg-blue-100 dark:bg-blue-900 rounded-full flex items-center justify-center mb-4">
                                                <svg class="w-8 h-8 text-blue-600 dark:text-blue-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z" />
                                                </svg>
                                            </div>
                                            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">"Bank Account (TrueLayer)"</h3>
                                            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{format!("Mandate: {}", pm_id)}</p>
                                        </div>
                                    }
                                }).map(|v| v.into_any()).unwrap_or_else(|| view! {
                                    <div>
                                        <div class="w-16 h-16 mx-auto bg-gray-100 dark:bg-gray-700 rounded-full flex items-center justify-center mb-4">
                                            <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z" />
                                            </svg>
                                        </div>
                                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">"No payment method"</h3>
                                        <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">"Set up a bank account via TrueLayer to enable paid plans."</p>
                                    </div>
                                }.into_any())}
                            </div>
                        }.into_any())
                    } else {
                        None
                    }}
                </div>
            </div>

            // Cancel confirmation modal
            {move || if show_cancel_confirm.get() {
                Some(view! {
                    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                        <div class="bg-white dark:bg-gray-800 rounded-none p-6 w-full max-w-md">
                            <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-2">"Cancel Subscription"</h2>
                            <p class="text-gray-600 dark:text-gray-400 mb-6">"Are you sure you want to cancel? You will retain access until the end of your current billing period."</p>
                            <div class="flex justify-end gap-3">
                                <button
                                    class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-none transition-colors"
                                    on:click=move |_| set_show_cancel_confirm.set(false)
                                >
                                    "Keep subscription"
                                </button>
                                <button
                                    class="px-4 py-2 bg-red-600 text-white rounded-none hover:bg-red-700 transition-colors"
                                    on:click=handle_cancel
                                >
                                    "Yes, cancel"
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_any())
            } else {
                None
            }}
        </div>
    }
}
