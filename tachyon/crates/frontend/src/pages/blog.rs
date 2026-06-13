// Blog Pages
//
// Blog listing with pagination, individual post view,
// and admin page for creating/editing posts.

use crate::api::blog::{BlogPost, CreateBlogPostRequest};
use leptos::prelude::*;
use leptos::prelude::{Get, Set};

/// Blog listing page with pagination.
#[component]
pub fn BlogPage() -> impl IntoView {
    let (page, set_page) = signal(1usize);
    let (posts, set_posts) = signal(Vec::<BlogPost>::new());
    let (total, set_total) = signal(0i64);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    // Load posts on mount
    let _ = leptos::task::spawn_local(async move {
        load_posts(1, set_posts, set_total, set_loading, set_error).await;
    });

    let load_page = move |new_page: usize| {
        set_page.set(new_page);
        leptos::task::spawn_local(async move {
            load_posts(new_page, set_posts, set_total, set_loading, set_error).await;
        });
    };

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8">
            <header class="mb-8">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white">"Blog"</h1>
                <p class="mt-2 text-gray-600 dark:text-gray-400">
                    "Latest posts from the Tachyon team"
                </p>
                <div class="mt-2">
                    <a href="/api/v1/blog/feed" class="text-blue-600 dark:text-blue-400 hover:underline text-sm">
                        "RSS Feed"
                    </a>
                </div>
            </header>

            {move || {
                if loading.get() {
                    view! {
                        <div class="text-center py-12">
                            <p class="text-gray-500">"Loading posts..."</p>
                        </div>
                    }.into_any()
                } else if let Some(ref err) = error.get() {
                    view! {
                        <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 mb-6">
                            <p class="text-red-700 dark:text-red-400">{err.clone()}</p>
                        </div>
                    }.into_any()
                } else if posts.get().is_empty() {
                    view! {
                        <div class="text-center py-12">
                            <p class="text-gray-500 dark:text-gray-400">"No posts yet."</p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div>
                            {posts.get().into_iter().map(|post| {
                                view! { <BlogPostCard post=post /> }
                            }).collect_view()}
                        </div>
                        {move || {
                            let current = page.get();
                            let total_pages = (total.get() as f64 / 20.0).ceil() as usize;
                            if total_pages > 1 {
                                view! {
                                    <div class="flex justify-between items-center mt-8 pt-6 border-t border-gray-200 dark:border-gray-700">
                                        <button
                                            on:click=move |_| { if current > 1 { load_page(current - 1); } }
                                            disabled=move || page.get() <= 1
                                            class="px-4 py-2 bg-gray-200 dark:bg-gray-700 rounded disabled:opacity-50 text-gray-900 dark:text-white"
                                        >
                                            "← Previous"
                                        </button>
                                        <span class="text-sm text-gray-500 dark:text-gray-400">
                                            {move || format!("Page {} of {}", page.get(), total_pages)}
                                        </span>
                                        <button
                                            on:click=move |_| { let next = page.get() + 1; if next <= total_pages { load_page(next); } }
                                            disabled=move || page.get() >= total_pages
                                            class="px-4 py-2 bg-gray-200 dark:bg-gray-700 rounded disabled:opacity-50 text-gray-900 dark:text-white"
                                        >
                                            "Next →"
                                        </button>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }
                        }}
                    }.into_any()
                }
            }}
        </div>
    }
}

async fn load_posts(
    page: usize,
    set_posts: WriteSignal<Vec<BlogPost>>,
    set_total: WriteSignal<i64>,
    set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    set_loading.set(true);
    set_error.set(None);

    let api = crate::api::ApiClient::default();
    match api.list_blog_posts(None, Some(page)).await {
        Ok(resp) => {
            set_posts.set(resp.posts);
            set_total.set(resp.total);
        }
        Err(e) => {
            set_error.set(Some(format!("Failed to load posts: {}", e)));
        }
    }
    set_loading.set(false);
}

#[component]
fn BlogPostCard(post: BlogPost) -> impl IntoView {
    let date_str = format_date(&post.created_at);
    let description = post
        .description
        .clone()
        .unwrap_or_else(|| "No description".to_string());
    let slug = post.slug.clone();
    let title = post.title.clone();
    let author = post.author.clone();
    let created_at = post.created_at.clone();
    let tags = post.tags.clone();

    view! {
        <article class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 mb-6 hover:shadow-lg transition-shadow">
            <h2 class="text-xl font-bold mb-2">
                <a
                    href={format!("/blog/{}", slug)}
                    class="text-gray-900 dark:text-white hover:text-blue-600 dark:hover:text-blue-400"
                >
                    {title}
                </a>
            </h2>
            <div class="flex items-center text-sm text-gray-500 dark:text-gray-400 mb-3">
                <time datetime={created_at.clone()}>{date_str}</time>
                <span class="mx-2">"·"</span>
                <span>{author}</span>
            </div>
            <p class="text-gray-600 dark:text-gray-300 mb-3">{description}</p>
            <div class="flex flex-wrap">
                {tags.into_iter().map(|t| {
                    view! {
                        <span class="inline-block bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-xs px-2 py-1 rounded mr-1 mb-1">
                            {t}
                        </span>
                    }
                }).collect_view()}
            </div>
        </article>
    }
}

/// Individual blog post page.
#[component]
pub fn BlogPostPage() -> impl IntoView {
    let params = leptos_router::hooks::use_params_map();
    let slug = move || {
        params
            .get()
            .get("slug")
            .map(|s| s.to_string())
            .unwrap_or_default()
    };

    let (post, set_post) = signal(Option::<BlogPost>::None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    let current_slug = slug();
    let _ = leptos::task::spawn_local(async move {
        set_loading.set(true);
        let api = crate::api::ApiClient::default();
        match api.get_blog_post(&current_slug).await {
            Ok(p) => set_post.set(Some(p)),
            Err(e) => set_error.set(Some(format!("Failed to load post: {}", e))),
        }
        set_loading.set(false);
    });

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8">
            {move || {
                if loading.get() {
                    view! {
                        <div class="text-center py-12">
                            <p class="text-gray-500">"Loading post..."</p>
                        </div>
                    }.into_any()
                } else if let Some(ref err) = error.get() {
                    view! {
                        <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
                            <p class="text-red-700 dark:text-red-400">{err.clone()}</p>
                            <a href="/blog" class="mt-2 inline-block text-blue-600 hover:underline">
                                "← Back to Blog"
                            </a>
                        </div>
                    }.into_any()
                } else if let Some(ref p) = post.get() {
                    let date_str = format_date(&p.created_at);
                    view! {
                        <article>
                            <header class="mb-8">
                                <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-4">
                                    {p.title.clone()}
                                </h1>
                                <div class="flex items-center text-sm text-gray-500 dark:text-gray-400 mb-4">
                                    <time datetime={p.created_at.clone()}>{date_str}</time>
                                    <span class="mx-2">"·"</span>
                                    <span>{p.author.clone()}</span>
                                </div>
                                <div class="flex flex-wrap mb-4">
                                    {p.tags.clone().into_iter().map(|t| {
                                        view! {
                                            <span class="inline-block bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-sm px-3 py-1 rounded-full mr-2">
                                                {t}
                                            </span>
                                        }
                                    }).collect_view()}
                                </div>
                                {p.cover_image.as_ref().map(|img| {
                                    view! {
                                        <img
                                            src={img.clone()}
                                            alt={p.title.clone()}
                                            class="w-full rounded-lg shadow-md mb-4"
                                        />
                                    }
                                })}
                            </header>
                            <div class="prose dark:prose-invert max-w-none" inner_html={render_markdown_html(&p.content)}></div>
                            <footer class="mt-12 pt-6 border-t border-gray-200 dark:border-gray-700">
                                <a href="/blog" class="text-blue-600 dark:text-blue-400 hover:underline">
                                    "← Back to Blog"
                                </a>
                            </footer>
                        </article>
                    }.into_any()
                } else {
                    view! {
                        <div class="text-center py-12">
                            <p class="text-gray-500">"Post not found."</p>
                            <a href="/blog" class="mt-2 inline-block text-blue-600 hover:underline">
                                "← Back to Blog"
                            </a>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// Blog admin page for creating/editing posts.
#[component]
pub fn BlogAdminPage() -> impl IntoView {
    let (title, set_title) = signal(String::new());
    let (content, set_content) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (tags_input, set_tags_input) = signal(String::new());
    let (published, set_published) = signal(true);
    let (saving, set_saving) = signal(false);
    let (message, set_message) = signal(Option::<String>::None);
    let (is_error, set_is_error) = signal(false);

    let save_post = move |_: web_sys::MouseEvent| {
        let title_val = title.get();
        let content_val = content.get();
        let desc_val = description.get();
        let tags_val = tags_input.get();
        let published_val = published.get();

        if title_val.trim().is_empty() || content_val.trim().is_empty() {
            set_message.set(Some("Title and content are required".to_string()));
            set_is_error.set(true);
            return;
        }

        let tags: Vec<String> = tags_val
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        set_saving.set(true);
        leptos::task::spawn_local(async move {
            let api = crate::api::ApiClient::default();
            let req = CreateBlogPostRequest {
                title: title_val,
                content: content_val,
                description: if desc_val.is_empty() {
                    None
                } else {
                    Some(desc_val)
                },
                tags,
                cover_image: None,
                published: published_val,
            };
            match api.create_blog_post(&req).await {
                Ok(_) => {
                    set_message.set(Some("Post created successfully".to_string()));
                    set_is_error.set(false);
                    set_title.set(String::new());
                    set_content.set(String::new());
                    set_description.set(String::new());
                    set_tags_input.set(String::new());
                }
                Err(e) => {
                    set_message.set(Some(format!("Failed to create post: {}", e)));
                    set_is_error.set(true);
                }
            }
            set_saving.set(false);
        });
    };

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-6">"Create Blog Post"</h1>

            {move || {
                if let Some(ref msg) = message.get() {
                    let bg = if is_error.get() {
                        "bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800 text-red-700 dark:text-red-400"
                    } else {
                        "bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800 text-green-700 dark:text-green-400"
                    };
                    view! {
                        <div class={format!("{} border rounded-lg p-4 mb-6", bg)}>
                            {msg.clone()}
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}

            <form class="space-y-6">
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Title"</label>
                    <input
                        type="text"
                        prop:value=title
                        on:input=move |ev| set_title.set(event_target_value(&ev))
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                        placeholder="My awesome blog post"
                    />
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Description"</label>
                    <input
                        type="text"
                        prop:value=description
                        on:input=move |ev| set_description.set(event_target_value(&ev))
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                        placeholder="A short description for listings and meta tags"
                    />
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Tags (comma-separated)"</label>
                    <input
                        type="text"
                        prop:value=tags_input
                        on:input=move |ev| set_tags_input.set(event_target_value(&ev))
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                        placeholder="rust, tutorial, web"
                    />
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Content (Markdown)"</label>
                    <textarea
                        prop:value=content
                        on:input=move |ev| set_content.set(event_target_value(&ev))
                        rows="20"
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white font-mono text-sm"
                        placeholder="# Hello World\n\nYour blog post content here..."
                    ></textarea>
                </div>

                <div class="flex items-center">
                    <input
                        type="checkbox"
                        prop:checked=published
                        on:change=move |ev| set_published.set(event_target_checked(&ev))
                        class="mr-2"
                    />
                    <label class="text-sm text-gray-700 dark:text-gray-300">"Publish immediately"</label>
                </div>

                <button
                    type="button"
                    on:click=save_post
                    disabled=saving
                    class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 font-medium"
                >
                    {move || if saving.get() { "Saving..." } else { "Create Post" }}
                </button>
            </form>
        </div>
    }
}

fn format_date(iso_str: &str) -> String {
    // Simple date formatting without chrono dependency in WASM
    // Parse "2025-01-15T10:00:00Z" → "January 15, 2025"
    if let Some(date_part) = iso_str.split('T').next() {
        let parts: Vec<&str> = date_part.split('-').collect();
        if parts.len() == 3 {
            let months = [
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ];
            if let (Ok(year), Some(month_str), Ok(day)) = (
                parts[0].parse::<usize>(),
                months.get(parts[1].parse::<usize>().unwrap_or(0).wrapping_sub(1)),
                parts[2].parse::<usize>(),
            ) {
                return format!("{} {}, {}", month_str, day, year);
            }
        }
    }
    iso_str.to_string()
}

/// Simple markdown renderer using pulldown-cmark.
fn render_markdown_html(content: &str) -> String {
    use pulldown_cmark::{Options, Parser, html};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
