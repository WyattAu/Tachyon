// User Avatar Component
// Displays a user avatar image or initials fallback with deterministic color

#![allow(dead_code)]

use leptos::prelude::*;

fn name_to_color(name: &str) -> &'static str {
    let hash: usize = name.bytes().map(|b| b as usize).sum();
    let palette = [
        "#ef4444", "#f97316", "#eab308", "#22c55e",
        "#06b6d4", "#3b82f6", "#8b5cf6", "#ec4899",
    ];
    palette[hash % palette.len()]
}

fn extract_initials(name: &str) -> String {
    let parts: Vec<&str> = name.split_whitespace().collect();
    match parts.as_slice() {
        [first] => first.chars().next().unwrap_or('?').to_uppercase().to_string(),
        [first, .., last] => format!(
            "{}{}",
            first.chars().next().unwrap_or('?'),
            last.chars().next().unwrap_or('?')
        )
        .to_uppercase(),
        _ => "?".to_string(),
    }
}

#[component]
pub fn UserAvatar(
    #[prop(into)]
    name: String,
    #[prop(optional, into)]
    image_url: Option<String>,
    #[prop(default = 32)]
    size: u32,
    #[prop(optional, into)]
    class: Option<String>,
) -> impl IntoView {
    let initials = extract_initials(&name);
    let bg_color = name_to_color(&name);
    let font_size = size / 2;

    let size_style = format!("width: {}px; height: {}px;", size, size);
    let initials_style = format!(
        "width: {}px; height: {}px; background-color: {}; font-size: {}px;",
        size, size, bg_color, font_size
    );

    let extra_class = class.unwrap_or_default();

    view! {
        {if let Some(url) = image_url {
            view! {
                <img
                    src=url
                    alt=name
                    class=format!("rounded-full object-cover {}", extra_class)
                    style=size_style
                />
            }.into_any()
        } else {
            view! {
                <div
                    class=format!("rounded-full flex items-center justify-center text-white font-medium {}", extra_class)
                    style=initials_style
                >
                    {initials}
                </div>
            }.into_any()
        }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_initials_single_word() {
        assert_eq!(extract_initials("Alice"), "A");
    }

    #[test]
    fn test_extract_initials_two_words() {
        assert_eq!(extract_initials("Alice Smith"), "AS");
    }

    #[test]
    fn test_extract_initials_three_words() {
        assert_eq!(extract_initials("Alice B. Smith"), "AS");
    }

    #[test]
    fn test_extract_initials_many_words() {
        assert_eq!(extract_initials("Alice Beth Catherine Smith"), "AS");
    }

    #[test]
    fn test_extract_initials_empty() {
        assert_eq!(extract_initials(""), "?");
    }

    #[test]
    fn test_extract_initials_whitespace() {
        assert_eq!(extract_initials("   "), "?");
    }

    #[test]
    fn test_extract_initials_case() {
        assert_eq!(extract_initials("alice smith"), "AS");
        assert_eq!(extract_initials("ALICE SMITH"), "AS");
    }

    #[test]
    fn test_name_to_color_deterministic() {
        assert_eq!(name_to_color("Alice"), name_to_color("Alice"));
        assert_eq!(name_to_color("Bob"), name_to_color("Bob"));
    }

    #[test]
    fn test_name_to_color_covers_palette() {
        let palette = [
            "#ef4444", "#f97316", "#eab308", "#22c55e",
            "#06b6d4", "#3b82f6", "#8b5cf6", "#ec4899",
        ];
        let names = ["A", "B", "C", "D", "E", "F", "G", "H"];
        let colors: std::collections::HashSet<_> = names.iter().map(|n| name_to_color(n)).collect();
        assert_eq!(colors.len(), 8);
        for color in &colors {
            assert!(palette.contains(color), "color {} not in palette", color);
        }
    }

    #[test]
    fn test_name_to_color_different_names() {
        let color_a = name_to_color("Alice");
        let color_b = name_to_color("Bob");
        assert_ne!(color_a, color_b);
    }
}
