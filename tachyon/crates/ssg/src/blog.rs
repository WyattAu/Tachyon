//! Blog support for the Tachyon SSG.
//!
//! Generates blog listing pages, individual post pages,
//! RSS/Atom feeds, and social media meta tags.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{SsgError, SsgResult};
use crate::render::escape_xml;
use crate::slug::slugify;

/// A blog post to be included in the static site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
    /// URL slug (used as filename: `blog/{slug}.html`)
    pub slug: String,
    /// Post title
    pub title: String,
    /// Raw markdown content
    pub content: String,
    /// Short description (for meta tags, RSS, listings)
    pub description: Option<String>,
    /// Author display name
    pub author: String,
    /// Publication date
    pub date: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional cover image URL
    pub cover_image: Option<String>,
    /// Whether this post is published (visible) or a draft
    #[serde(default = "default_true")]
    pub published: bool,
}

fn default_true() -> bool {
    true
}

impl BlogPost {
    /// Create a new blog post with auto-generated slug from title.
    pub fn new(
        title: impl Into<String>,
        author: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        let title = title.into();
        let slug = slugify(&title);
        let now = Utc::now();
        Self {
            slug,
            title,
            content: content.into(),
            description: None,
            author: author.into(),
            date: now,
            updated_at: now,
            tags: vec![],
            cover_image: None,
            published: true,
        }
    }

    /// Generate Open Graph meta tags for this post.
    pub fn open_graph_meta(&self, base_url: &str) -> String {
        let base = base_url.trim_end_matches('/');
        let url = format!("{}/blog/{}.html", base, self.slug);
        let description = self
            .description
            .as_deref()
            .unwrap_or("Blog post on Tachyon");

        let mut meta = format!(
            r#"<meta property="og:type" content="article"/>
<meta property="og:title" content="{}"/>
<meta property="og:description" content="{}"/>
<meta property="og:url" content="{}"/>
<meta property="og:site_name" content="Tachyon Blog"/>"#,
            escape_xml(&self.title),
            escape_xml(description),
            escape_xml(&url),
        );

        if let Some(ref image) = self.cover_image {
            meta.push_str(&format!(
                r#"
<meta property="og:image" content="{}"/>"#,
                escape_xml(image),
            ));
        }

        meta.push_str(&format!(
            r#"
<meta property="article:published_time" content="{}"/>
<meta property="article:modified_time" content="{}"/>
<meta property="article:author" content="{}"/>"#,
            self.date.to_rfc3339(),
            self.updated_at.to_rfc3339(),
            escape_xml(&self.author),
        ));

        for tag in &self.tags {
            meta.push_str(&format!(
                r#"
<meta property="article:tag" content="{}"/>"#,
                escape_xml(tag),
            ));
        }

        meta
    }

    /// Generate Twitter Card meta tags for this post.
    pub fn twitter_card_meta(&self, base_url: &str) -> String {
        let base = base_url.trim_end_matches('/');
        let url = format!("{}/blog/{}.html", base, self.slug);
        let description = self
            .description
            .as_deref()
            .unwrap_or("Blog post on Tachyon");

        let card_type = if self.cover_image.is_some() {
            "summary_large_image"
        } else {
            "summary"
        };

        let mut meta = format!(
            r#"<meta name="twitter:card" content="{}"/>
<meta name="twitter:title" content="{}"/>
<meta name="twitter:description" content="{}"/>
<meta name="twitter:url" content="{}"/>"#,
            card_type,
            escape_xml(&self.title),
            escape_xml(description),
            escape_xml(&url),
        );

        if let Some(ref image) = self.cover_image {
            meta.push_str(&format!(
                r#"
<meta name="twitter:image" content="{}"/>"#,
                escape_xml(image),
            ));
        }

        meta
    }
}

/// Blog configuration for the site.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlogConfig {
    /// Blog listing page title
    #[serde(default = "default_blog_title")]
    pub title: String,
    /// Blog description
    #[serde(default = "default_blog_description")]
    pub description: String,
    /// Posts per page in listings
    #[serde(default = "default_posts_per_page")]
    pub posts_per_page: usize,
    /// Enable RSS feed generation
    #[serde(default = "default_true")]
    pub rss_enabled: bool,
}

fn default_blog_title() -> String {
    "Blog".to_string()
}
fn default_blog_description() -> String {
    "Latest posts from the Tachyon team".to_string()
}
fn default_posts_per_page() -> usize {
    10
}

/// Generate the blog listing page HTML.
pub fn render_blog_listing(posts: &[BlogPost], config: &BlogConfig, base_url: &str) -> String {
    let _base = base_url.trim_end_matches('/');
    let mut cards = String::new();

    for post in posts {
        if !post.published {
            continue;
        }
        let description = post.description.as_deref().unwrap_or("No description");
        let date_str = post.date.format("%B %d, %Y").to_string();
        let tag_html: String = post
            .tags
            .iter()
            .map(|t| {
                format!(
                    r#"<span class="inline-block bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-xs px-2 py-1 rounded mr-1">{}</span>"#,
                    escape_xml(t)
                )
            })
            .collect();

        cards.push_str(&format!(
            r#"
        <article class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 mb-6 hover:shadow-lg transition-shadow">
            <h2 class="text-xl font-bold mb-2">
                <a href="blog/{}.html" class="text-gray-900 dark:text-white hover:text-blue-600 dark:hover:text-blue-400">{}</a>
            </h2>
            <div class="flex items-center text-sm text-gray-500 dark:text-gray-400 mb-3">
                <time datetime="{}">{}</time>
                <span class="mx-2">·</span>
                <span>{}</span>
            </div>
            <p class="text-gray-600 dark:text-gray-300 mb-3">{}</p>
            <div class="flex flex-wrap">{}</div>
        </article>"#,
            post.slug,
            escape_xml(&post.title),
            post.date.to_rfc3339(),
            date_str,
            escape_xml(&post.author),
            escape_xml(description),
            tag_html,
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>{title}</title>
    <meta name="description" content="{description}"/>
    <link rel="alternate" type="application/rss+xml" title="{title} RSS" href="feed.xml"/>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css"/>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 0; background: #f9fafb; }}
        .container {{ max-width: 800px; margin: 0 auto; padding: 2rem 1rem; }}
        @media (prefers-color-scheme: dark) {{ body {{ background: #111827; }} }}
    </style>
</head>
<body>
    <div class="container">
        <header class="mb-8">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">{title}</h1>
            <p class="text-gray-600 dark:text-gray-400">{description}</p>
        </header>
        <main>{cards}
        </main>
        <footer class="mt-8 pt-6 border-t border-gray-200 dark:border-gray-700 text-center text-sm text-gray-500">
            <p>Built with Tachyon SSG</p>
        </footer>
    </div>
</body>
</html>"#,
        title = escape_xml(&config.title),
        description = escape_xml(&config.description),
        cards = cards,
    )
}

/// Generate an individual blog post page.
pub fn render_blog_post(post: &BlogPost, base_url: &str) -> String {
    let _base = base_url.trim_end_matches('/');
    let date_str = post.date.format("%B %d, %Y").to_string();
    let og_meta = post.open_graph_meta(base_url);
    let twitter_meta = post.twitter_card_meta(base_url);

    let tag_html: String = post
        .tags
        .iter()
        .map(|t| {
            format!(
                r#"<span class="inline-block bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-sm px-3 py-1 rounded-full mr-2">{}</span>"#,
                escape_xml(t)
            )
        })
        .collect();

    let content_html = crate::render::render_markdown(&post.content, "client", "github-dark");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>{title} — Blog</title>
    <meta name="description" content="{description}"/>
    {og_meta}
    {twitter_meta}
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css"/>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/github-dark.min.css"/>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/rust.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/contrib/auto-render.min.js"></script>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 0; background: #f9fafb; line-height: 1.7; }}
        .container {{ max-width: 720px; margin: 0 auto; padding: 2rem 1rem; }}
        .post-content h1 {{ font-size: 1.8rem; margin-top: 2rem; }}
        .post-content h2 {{ font-size: 1.4rem; margin-top: 1.8rem; border-bottom: 1px solid #e5e7eb; padding-bottom: 0.3rem; }}
        .post-content h3 {{ font-size: 1.2rem; margin-top: 1.5rem; }}
        .post-content pre {{ background: #1f2937; color: #e5e7eb; padding: 1rem; border-radius: 0.5rem; overflow-x: auto; }}
        .post-content code {{ font-family: 'Fira Code', monospace; font-size: 0.9em; }}
        .post-content p code {{ background: #f3f4f6; padding: 0.15rem 0.4rem; border-radius: 0.25rem; }}
        .post-content blockquote {{ border-left: 4px solid #3b82f6; margin: 1rem 0; padding: 0.5rem 1rem; background: #eff6ff; }}
        .post-content img {{ max-width: 100%; border-radius: 0.5rem; }}
        .post-content table {{ border-collapse: collapse; width: 100%; margin: 1rem 0; }}
        .post-content th, .post-content td {{ border: 1px solid #d1d5db; padding: 0.5rem 1rem; text-align: left; }}
        .post-content th {{ background: #f9fafb; }}
        @media (prefers-color-scheme: dark) {{ body {{ background: #111827; }} .post-content blockquote {{ background: #1e293b; }} .post-content p code {{ background: #374151; }} .post-content th {{ background: #1f2937; }} }}
    </style>
</head>
<body>
    <article class="container">
        <header class="mb-8">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-4">{title}</h1>
            <div class="flex items-center text-sm text-gray-500 dark:text-gray-400 mb-4">
                <time datetime="{date_rfc3339}">{date_display}</time>
                <span class="mx-2">·</span>
                <span>{author}</span>
            </div>
            <div class="flex flex-wrap mb-4">{tags}</div>
            {cover_image}
        </header>
        <div class="post-content prose dark:prose-invert">{content}</div>
        <footer class="mt-12 pt-6 border-t border-gray-200 dark:border-gray-700">
            <a href="../blog.html" class="text-blue-600 dark:text-blue-400 hover:underline">&larr; Back to Blog</a>
        </footer>
    </article>
    <script>hljs.highlightAll();</script>
    <script>
        document.addEventListener("DOMContentLoaded", function() {{
            renderMathInElement(document.body, {{
                delimiters: [
                    {{left: "$$", right: "$$", display: true}},
                    {{left: "$", right: "$", display: false}},
                    {{left: "\\\\(", right: "\\\\)", display: false}},
                    {{left: "\\\\[", right: "\\\\]", display: true}}
                ]
            }});
        }});
    </script>
</body>
</html>"#,
        title = escape_xml(&post.title),
        description = escape_xml(
            post.description.as_deref().unwrap_or("Blog post"),
        ),
        og_meta = og_meta,
        twitter_meta = twitter_meta,
        date_rfc3339 = post.date.to_rfc3339(),
        date_display = date_str,
        author = escape_xml(&post.author),
        tags = tag_html,
        cover_image = post.cover_image.as_ref().map(|img| {
            format!(
                r#"<div class="mb-4"><img src="{}" alt="{}" class="w-full rounded-lg shadow-md"/></div>"#,
                escape_xml(img),
                escape_xml(&post.title),
            )
        }).unwrap_or_default(),
        content = content_html,
    )
}

/// Generate an RSS feed for blog posts.
pub fn render_blog_rss(posts: &[BlogPost], config: &BlogConfig, base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let now = Utc::now().to_rfc3339();

    let items: String = posts
        .iter()
        .filter(|p| p.published)
        .take(50)
        .map(|post| {
            let description = post.description.as_deref().unwrap_or("No description");
            let categories: String = post
                .tags
                .iter()
                .map(|t| format!("\n      <category>{}</category>", escape_xml(t)))
                .collect();

            format!(
                r#"
    <item>
      <title>{}</title>
      <link>{}/blog/{}.html</link>
      <description>{}</description>
      <pubDate>{}</pubDate>
      <guid isPermaLink="true">{}/blog/{}.html</guid>
      <author>{}</author>{}
    </item>"#,
                escape_xml(&post.title),
                base,
                post.slug,
                escape_xml(description),
                post.date.to_rfc2822(),
                base,
                post.slug,
                escape_xml(&post.author),
                categories,
            )
        })
        .collect();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom" xmlns:content="http://purl.org/rss/1.0/modules/content/">
  <channel>
    <title>{title}</title>
    <link>{base}/blog.html</link>
    <description>{description}</description>
    <language>en</language>
    <lastBuildDate>{now}</lastBuildDate>
    <atom:link href="{base}/blog/feed.xml" rel="self" type="application/rss+xml"/>
    <generator>Tachyon SSG</generator>{items}
  </channel>
</rss>"#,
        title = escape_xml(&config.title),
        base = base,
        description = escape_xml(&config.description),
        now = now,
        items = items,
    )
}

impl crate::build::SiteGenerator {
    /// Build blog pages into the output directory.
    pub(crate) fn build_blog(
        &self,
        posts: &[BlogPost],
        output_dir: &std::path::Path,
    ) -> SsgResult<usize> {
        let blog_dir = output_dir.join("blog");
        std::fs::create_dir_all(&blog_dir).map_err(|e| SsgError::Io(e.to_string()))?;

        let base_url = self.config.base_url.trim_end_matches('/');
        let blog_config = BlogConfig::default();

        // Generate individual post pages
        let mut count = 0;
        for post in posts {
            if !post.published {
                continue;
            }
            let html = render_blog_post(post, base_url);
            let path = blog_dir.join(format!("{}.html", post.slug));
            std::fs::write(&path, html).map_err(|e| SsgError::Io(e.to_string()))?;
            count += 1;
        }

        // Generate blog listing page
        let listing_html = render_blog_listing(posts, &blog_config, base_url);
        let listing_path = output_dir.join("blog.html");
        std::fs::write(&listing_path, listing_html).map_err(|e| SsgError::Io(e.to_string()))?;
        count += 1;

        // Generate RSS feed
        if blog_config.rss_enabled {
            let rss = render_blog_rss(posts, &blog_config, base_url);
            let rss_path = blog_dir.join("feed.xml");
            std::fs::write(&rss_path, rss).map_err(|e| SsgError::Io(e.to_string()))?;
            count += 1;
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_post() -> BlogPost {
        BlogPost {
            slug: "hello-world".to_string(),
            title: "Hello World".to_string(),
            content: "# Hello World\n\nThis is a test post.".to_string(),
            description: Some("A test blog post".to_string()),
            author: "Tachyon Team".to_string(),
            date: chrono::DateTime::parse_from_rfc3339("2025-01-15T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339("2025-01-15T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            tags: vec!["rust".to_string(), "tutorial".to_string()],
            cover_image: None,
            published: true,
        }
    }

    #[test]
    fn test_blog_post_new() {
        let post = BlogPost::new("My First Post", "Author", "Content here");
        assert_eq!(post.slug, "my-first-post");
        assert_eq!(post.title, "My First Post");
        assert!(post.published);
    }

    #[test]
    fn test_blog_post_open_graph_meta() {
        let post = sample_post();
        let meta = post.open_graph_meta("https://example.com");
        assert!(meta.contains(r#"og:type" content="article""#));
        assert!(meta.contains("Hello World"));
        assert!(meta.contains("A test blog post"));
        assert!(meta.contains("https://example.com/blog/hello-world.html"));
    }

    #[test]
    fn test_blog_post_twitter_card_meta() {
        let post = sample_post();
        let meta = post.twitter_card_meta("https://example.com");
        assert!(meta.contains(r#"twitter:card" content="summary""#));
        assert!(meta.contains("Hello World"));
        assert!(!meta.contains("twitter:image"));
    }

    #[test]
    fn test_blog_post_twitter_card_with_image() {
        let mut post = sample_post();
        post.cover_image = Some("https://example.com/image.jpg".to_string());
        let meta = post.twitter_card_meta("https://example.com");
        assert!(meta.contains(r#"twitter:card" content="summary_large_image""#));
        assert!(meta.contains("twitter:image"));
    }

    #[test]
    fn test_render_blog_listing() {
        let posts = vec![sample_post()];
        let config = BlogConfig::default();
        let html = render_blog_listing(&posts, &config, "https://example.com");
        assert!(html.contains("Hello World"));
        assert!(html.contains("Hello World</a>"));
        assert!(html.contains("Tachyon Team"));
        assert!(html.contains("rust"));
        assert!(html.contains("tutorial"));
        assert!(html.contains("Tachyon SSG"));
    }

    #[test]
    fn test_render_blog_listing_drafts_hidden() {
        let mut post = sample_post();
        post.published = false;
        let posts = vec![post];
        let config = BlogConfig::default();
        let html = render_blog_listing(&posts, &config, "https://example.com");
        assert!(!html.contains("Hello World</a>"));
    }

    #[test]
    fn test_render_blog_post() {
        let post = sample_post();
        let html = render_blog_post(&post, "https://example.com");
        assert!(html.contains("Hello World"));
        assert!(html.contains("og:type"));
        assert!(html.contains("twitter:card"));
        assert!(html.contains("Back to Blog"));
        assert!(html.contains("hljs.highlightAll"));
        assert!(html.contains("renderMathInElement"));
    }

    #[test]
    fn test_render_blog_rss() {
        let posts = vec![sample_post()];
        let config = BlogConfig::default();
        let rss = render_blog_rss(&posts, &config, "https://example.com");
        assert!(rss.contains(r#"<rss version="2.0""#));
        assert!(rss.contains("Hello World"));
        assert!(rss.contains("Tachyon Team"));
        assert!(rss.contains("rust"));
        assert!(rss.contains("tutorial"));
        assert!(rss.contains("https://example.com/blog/hello-world.html"));
        assert!(rss.contains("<channel>"));
    }

    #[test]
    fn test_render_blog_rss_drafts_excluded() {
        let mut post = sample_post();
        post.published = false;
        let posts = vec![post];
        let config = BlogConfig::default();
        let rss = render_blog_rss(&posts, &config, "https://example.com");
        assert!(!rss.contains("Hello World"));
    }

    #[test]
    fn test_blog_listing_with_no_description() {
        let mut post = sample_post();
        post.description = None;
        let posts = vec![post];
        let config = BlogConfig::default();
        let html = render_blog_listing(&posts, &config, "https://example.com");
        assert!(html.contains("No description"));
    }

    #[test]
    fn test_blog_post_with_cover_image() {
        let mut post = sample_post();
        post.cover_image = Some("https://example.com/cover.jpg".to_string());
        let html = render_blog_post(&post, "https://example.com");
        assert!(html.contains("og:image"));
        assert!(html.contains("cover.jpg"));
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Rust & C++"), "rust-c");
        assert_eq!(slugify("  multiple   spaces  "), "multiple-spaces");
    }
}
