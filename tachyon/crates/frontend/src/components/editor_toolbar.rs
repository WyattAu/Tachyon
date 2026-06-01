use crate::components::native_editor::{insert_line_prefix, insert_markdown_syntax};
use leptos::prelude::*;
use tachyon_editor::Editor;

#[component]
pub fn EditorToolbar(
    editor: RwSignal<Editor>,
    #[prop(default = Callback::new(|_: ()| {}))] on_save: Callback<()>,
    #[prop(default = Callback::new(|_: ()| {}))] on_preview: Callback<()>,
    #[prop(default = Callback::new(|_: ()| {}))] on_search: Callback<()>,
) -> impl IntoView {
    let ed = editor;
    let bold_action = move |_: leptos::ev::MouseEvent| {
        insert_markdown_syntax(ed, "**", "**", "bold");
    };
    let ed = editor;
    let italic_action = move |_: leptos::ev::MouseEvent| {
        insert_markdown_syntax(ed, "*", "*", "italic");
    };
    let ed = editor;
    let strikethrough_action = move |_: leptos::ev::MouseEvent| {
        insert_markdown_syntax(ed, "~~", "~~", "strikethrough");
    };
    let ed = editor;
    let code_action = move |_: leptos::ev::MouseEvent| {
        insert_markdown_syntax(ed, "`", "`", "code");
    };
    let ed = editor;
    let h1_action = move |_: leptos::ev::MouseEvent| {
        insert_line_prefix(ed, "# ");
    };
    let ed = editor;
    let h2_action = move |_: leptos::ev::MouseEvent| {
        insert_line_prefix(ed, "## ");
    };
    let ed = editor;
    let h3_action = move |_: leptos::ev::MouseEvent| {
        insert_line_prefix(ed, "### ");
    };
    let ed = editor;
    let ul_action = move |_: leptos::ev::MouseEvent| {
        insert_line_prefix(ed, "- ");
    };
    let ed = editor;
    let ol_action = move |_: leptos::ev::MouseEvent| {
        insert_line_prefix(ed, "1. ");
    };
    let ed = editor;
    let task_action = move |_: leptos::ev::MouseEvent| {
        insert_line_prefix(ed, "- [ ] ");
    };
    let ed = editor;
    let link_action = move |_: leptos::ev::MouseEvent| {
        insert_markdown_syntax(ed, "[", "](url)", "link text");
    };
    let ed = editor;
    let image_action = move |_: leptos::ev::MouseEvent| {
        insert_markdown_syntax(ed, "![", "](url)", "alt text");
    };
    let ed = editor;
    let code_block_action = move |_: leptos::ev::MouseEvent| {
        insert_markdown_syntax(ed, "```\n", "\n```", "code");
    };
    let ed = editor;
    let table_action = move |_: leptos::ev::MouseEvent| {
        ed.update(|e| {
            e.insert_text("| Header | Header |\n| --- | --- |\n| Cell | Cell |");
        });
    };
    let ed = editor;
    let hr_action = move |_: leptos::ev::MouseEvent| {
        insert_line_prefix(ed, "---\n");
    };
    let ed = editor;
    let blockquote_action = move |_: leptos::ev::MouseEvent| {
        insert_line_prefix(ed, "> ");
    };
    let ed = editor;
    let undo_action = move |_: leptos::ev::MouseEvent| {
        ed.update(|e| {
            e.undo();
        });
    };
    let ed = editor;
    let redo_action = move |_: leptos::ev::MouseEvent| {
        ed.update(|e| {
            e.redo();
        });
    };

    view! {
        <div class="editor-toolbar">
            // Text style
            <ToolbarBtn title="Bold" on_click={bold_action} disabled={false}>
                <span class="font-bold">{"B"}</span>
            </ToolbarBtn>
            <ToolbarBtn title="Italic" on_click={italic_action} disabled={false}>
                <span class="italic">{"I"}</span>
            </ToolbarBtn>
            <ToolbarBtn title="Strikethrough" on_click={strikethrough_action} disabled={false}>
                <span class="line-through">{"S"}</span>
            </ToolbarBtn>
            <ToolbarBtn title="Inline Code" on_click={code_action} disabled={false}>
                <span class="font-mono text-xs">{"<>"}</span>
            </ToolbarBtn>

            <ToolbarSep />

            // Headings
            <ToolbarBtn title="Heading 1" on_click={h1_action} disabled={false}>
                <span class="font-bold text-xs">{"H1"}</span>
            </ToolbarBtn>
            <ToolbarBtn title="Heading 2" on_click={h2_action} disabled={false}>
                <span class="font-bold text-xs">{"H2"}</span>
            </ToolbarBtn>
            <ToolbarBtn title="Heading 3" on_click={h3_action} disabled={false}>
                <span class="font-bold text-xs">{"H3"}</span>
            </ToolbarBtn>

            <ToolbarSep />

            // Lists
            <ToolbarBtn title="Bullet List" on_click={ul_action} disabled={false}>
                {"\u{2022} List"}
            </ToolbarBtn>
            <ToolbarBtn title="Ordered List" on_click={ol_action} disabled={false}>
                {"1. List"}
            </ToolbarBtn>
            <ToolbarBtn title="Task List" on_click={task_action} disabled={false}>
                {"\u{2610} Task"}
            </ToolbarBtn>
            <ToolbarBtn title="Blockquote" on_click={blockquote_action} disabled={false}>
                {"\u{201C} Quote"}
            </ToolbarBtn>

            <ToolbarSep />

            // Insert
            <ToolbarBtn title="Link" on_click={link_action} disabled={false}>
                {"\u{1F517}"}
            </ToolbarBtn>
            <ToolbarBtn title="Image" on_click={image_action} disabled={false}>
                {"\u{1F5BC}"}
            </ToolbarBtn>
            <ToolbarBtn title="Code Block" on_click={code_block_action} disabled={false}>
                <span class="font-mono text-xs">{"{ }"}</span>
            </ToolbarBtn>
            <ToolbarBtn title="Table" on_click={table_action} disabled={false}>
                {"\u{2638}"}
            </ToolbarBtn>
            <ToolbarBtn title="Horizontal Rule" on_click={hr_action} disabled={false}>
                {"\u{2500}"}
            </ToolbarBtn>

            <ToolbarSep />

            // Actions
            <ToolbarBtn title="Undo (Ctrl+Z)" on_click={undo_action} disabled={editor.with(|e| !e.can_undo())}>
                {"\u{21A9}"}
            </ToolbarBtn>
            <ToolbarBtn title="Redo (Ctrl+Shift+Z)" on_click={redo_action} disabled={editor.with(|e| !e.can_redo())}>
                {"\u{21AA}"}
            </ToolbarBtn>
            <ToolbarBtn title="Search (Ctrl+F)" on_click={move |_: leptos::ev::MouseEvent| on_search.run(())} disabled={false}>
                {"\u{1F50D}"}
            </ToolbarBtn>
            <ToolbarBtn title="Preview" on_click={move |_: leptos::ev::MouseEvent| on_preview.run(())} disabled={false}>
                {"\u{1F441}"}
            </ToolbarBtn>
            <ToolbarBtn title="Save (Ctrl+S)" on_click={move |_: leptos::ev::MouseEvent| on_save.run(())} disabled={false}>
                {"\u{1F4BE}"}
            </ToolbarBtn>
        </div>
    }
}

#[component]
fn ToolbarBtn(
    title: &'static str,
    on_click: impl Fn(leptos::ev::MouseEvent) + 'static,
    #[prop(default = false)] disabled: bool,
    children: Children,
) -> impl IntoView {
    let child_views = children();
    view! {
        <button
            class="editor-toolbar-btn"
            on:click={on_click}
            title={title}
            aria-label={title}
            disabled={disabled}
        >
            {child_views}
        </button>
    }
}

#[component]
fn ToolbarSep() -> impl IntoView {
    view! {
        <div class="editor-toolbar-sep"></div>
    }
}
