use leptos::prelude::*;
use tachyon_editor::Editor;

#[derive(Clone)]
pub struct SlashCommand {
    pub name: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub prefix: &'static str,
    pub suffix: &'static str,
    pub default_text: &'static str,
}

pub fn get_slash_commands() -> Vec<SlashCommand> {
    vec![
        SlashCommand {
            name: "heading1",
            label: "H1",
            description: "Heading 1",
            prefix: "# ",
            suffix: "",
            default_text: "Heading",
        },
        SlashCommand {
            name: "heading2",
            label: "H2",
            description: "Heading 2",
            prefix: "## ",
            suffix: "",
            default_text: "Heading",
        },
        SlashCommand {
            name: "heading3",
            label: "H3",
            description: "Heading 3",
            prefix: "### ",
            suffix: "",
            default_text: "Heading",
        },
        SlashCommand {
            name: "code",
            label: "Code",
            description: "Code block",
            prefix: "```\n",
            suffix: "\n```",
            default_text: "code",
        },
        SlashCommand {
            name: "table",
            label: "Table",
            description: "Markdown table",
            prefix: "",
            suffix: "",
            default_text: "| Header | Header |\n| ------ | ------ |\n| Cell   | Cell   |",
        },
        SlashCommand {
            name: "math",
            label: "Math",
            description: "Math equation",
            prefix: "$$",
            suffix: "$$",
            default_text: "E = mc^2",
        },
        SlashCommand {
            name: "image",
            label: "Image",
            description: "Insert image",
            prefix: "![",
            suffix: "](url)",
            default_text: "alt text",
        },
        SlashCommand {
            name: "quote",
            label: "Quote",
            description: "Blockquote",
            prefix: "> ",
            suffix: "",
            default_text: "Quote",
        },
        SlashCommand {
            name: "list",
            label: "List",
            description: "Bullet list",
            prefix: "- ",
            suffix: "",
            default_text: "Item",
        },
        SlashCommand {
            name: "task",
            label: "Task",
            description: "Task list",
            prefix: "- [ ] ",
            suffix: "",
            default_text: "Task",
        },
    ]
}

#[component]
pub fn SlashCommandMenu(
    visible: Signal<bool>,
    query: Signal<String>,
    position: Signal<(f64, f64)>,
    #[allow(unused)] editor: RwSignal<Editor>,
    #[allow(unused)] render_tick: RwSignal<u64>,
    on_select: Callback<SlashCommand>,
    on_close: Callback<()>,
) -> impl IntoView {
    let (selected_idx, set_selected_idx) = signal(0usize);

    let filtered_commands = move || {
        let q = query.get().to_lowercase();
        get_slash_commands()
            .into_iter()
            .filter(|cmd| {
                q.is_empty()
                    || cmd.name.contains(&q)
                    || cmd.label.to_lowercase().contains(&q)
                    || cmd.description.to_lowercase().contains(&q)
            })
            .collect::<Vec<_>>()
    };

    // Reset selection when query changes
    Effect::new(move |_| {
        let _ = query.get();
        set_selected_idx.set(0);
    });

    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        let commands = filtered_commands();
        if commands.is_empty() {
            return;
        }
        match ev.key().as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                let idx = selected_idx.get();
                set_selected_idx.set((idx + 1) % commands.len());
            }
            "ArrowUp" => {
                ev.prevent_default();
                let idx = selected_idx.get();
                set_selected_idx.set(if idx == 0 {
                    commands.len() - 1
                } else {
                    idx - 1
                });
            }
            "Enter" | "Tab" => {
                ev.prevent_default();
                let idx = selected_idx.get();
                if let Some(cmd) = commands.get(idx) {
                    on_select.run(cmd.clone());
                }
            }
            "Escape" => {
                ev.prevent_default();
                on_close.run(());
            }
            _ => {}
        }
    };

    view! {
        {move || {
            if !visible.get() {
                return ().into_any();
            }

            let (left, top) = position.get();
            let commands = filtered_commands();
            if commands.is_empty() {
                return ().into_any();
            }

            let on_select_clone = on_select;
            let on_close_clone = on_close;

            view! {
                <div
                    class="slash-command-menu"
                    style:position="absolute"
                    style:left={format!("{}px", left)}
                    style:top={format!("{}px", top)}
                    style:z-index="1000"
                    on:keydown=handle_keydown
                    tabindex="0"
                >
                    {commands.into_iter().enumerate().map(|(idx, cmd)| {
                        let is_selected = move || selected_idx.get() == idx;
                        let cmd_clone = cmd.clone();
                        let on_select_inner = on_select_clone;
                        let _on_close_inner = on_close_clone;
                        view! {
                            <div
                                class="slash-command-item"
                                class:selected={is_selected}
                                on:click={move |_| on_select_inner.run(cmd_clone.clone())}
                                on:mouseenter={move |_| set_selected_idx.set(idx)}
                            >
                                <div class="slash-command-label">{cmd.label}</div>
                                <div class="slash-command-desc">{cmd.description}</div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            }.into_any()
        }}
    }
}
