// Tachyon Editor — ProseMirror-based Markdown editor
//
// Exposes `window.TachyonEditor` with the following API:
//   create(container, options) → editorView
//   getMarkdown(editorView) → string
//   setMarkdown(editorView, markdown) → void
//   setOnChange(editorView, callback) → void
//   setEditable(editorView, editable) → void
//   focus(editorView) → void
//   destroy(editorView) → void

import { Schema } from 'prosemirror-model';
import { EditorState, Plugin, PluginKey } from 'prosemirror-state';
import { EditorView } from 'prosemirror-view';
import { baseKeymap, toggleMark, setBlockType, wrapIn, toggleStrongmark } from 'prosemirror-commands';
import { keymap } from 'prosemirror-keymap';
import { history, undo, redo } from 'prosemirror-history';
import { inputRules, wrappingInputRule, textblockTypeInputRule, InputRule } from 'prosemirror-inputrules';
import { gapCursor } from 'prosemirror-gapcursor';
import {
  wrapInList,
  splitListItem,
  liftListItem,
  sinkListItem,
} from 'prosemirror-schema-list';
import {
  addColumnBefore,
  addColumnAfter,
  deleteColumn,
  addRowBefore,
  addRowAfter,
  deleteRow,
  mergeCells,
  splitCell,
  setCellAttr,
  toggleHeaderRow,
  toggleHeaderColumn,
  goToNextCell,
  deleteTable,
  tableEditing,
} from 'prosemirror-tables';
import {
  defaultMarkdownParser,
  defaultMarkdownSerializer,
  schema as markdownSchema,
} from 'prosemirror-markdown';

// ─── OnChange plugin key ──────────────────────────────────────────
const onChangeKey = new PluginKey('onChange');

// ─── Schema extensions ─────────────────────────────────────────────
// Extend the basic markdown schema with tables and strikethrough.

const tachyonSchema = new Schema({
  nodes: {
    doc: { content: 'block+' },

    paragraph: {
      content: 'inline*',
      group: 'block',
      parseDOM: [{ tag: 'p' }],
      toDOM() { return ['p', 0]; },
    },

    blockquote: {
      content: 'block+',
      group: 'block',
      defining: true,
      parseDOM: [{ tag: 'blockquote' }],
      toDOM() { return ['blockquote', 0]; },
    },

    horizontal_rule: {
      group: 'block',
      parseDOM: [{ tag: 'hr' }],
      toDOM() { return ['hr']; },
    },

    heading: {
      attrs: { level: { default: 1 } },
      content: 'inline*',
      group: 'block',
      defining: true,
      parseDOM: [
        { tag: 'h1', attrs: { level: 1 } },
        { tag: 'h2', attrs: { level: 2 } },
        { tag: 'h3', attrs: { level: 3 } },
        { tag: 'h4', attrs: { level: 4 } },
        { tag: 'h5', attrs: { level: 5 } },
        { tag: 'h6', attrs: { level: 6 } },
      ],
      toDOM(node) { return [`h${node.attrs.level}`, 0]; },
    },

    code_block: {
      content: 'text*',
      marks: '',
      group: 'block',
      code: true,
      defining: true,
      attrs: { params: { default: '' } },
      parseDOM: [{ tag: 'pre', preserveWhitespace: 'full' }],
      toDOM(node) { return ['pre', ['code', 0]]; },
    },

    ordered_list: {
      content: 'list_item+',
      group: 'block',
      attrs: { order: { default: 1 }, tight: { default: false } },
      parseDOM: [
        {
          tag: 'ol',
          getAttrs(dom, fragment) {
            return {
              order: dom.hasAttribute('start') ? +dom.getAttribute('start') : 1,
              tight: dom.hasAttribute('data-tight'),
            };
          },
        },
      ],
      toDOM(node) {
        return ['ol', { start: node.attrs.order == 1 ? null : node.attrs.order,
                        'data-tight': node.attrs.tight ? 'true' : null }, 0];
      },
    },

    bullet_list: {
      content: 'list_item+',
      group: 'block',
      attrs: { tight: { default: false } },
      parseDOM: [{ tag: 'ul', getAttrs: (dom) => ({ tight: dom.hasAttribute('data-tight') }) }],
      toDOM(node) {
        return ['ul', { 'data-tight': node.attrs.tight ? 'true' : null }, 0];
      },
    },

    list_item: {
      content: 'paragraph block*',
      parseDOM: [{ tag: 'li' }],
      toDOM() { return ['li', 0]; },
      defining: true,
    },

    text: { group: 'inline' },

    hard_break: {
      inline: true,
      group: 'inline',
      selectable: false,
      parseDOM: [{ tag: 'br' }],
      toDOM() { return ['br']; },
    },

    // Tables
    table: {
      content: 'table_row+',
      tableRole: 'table',
      isolating: true,
      group: 'block',
      parseDOM: [{ tag: 'table' }],
      toDOM() { return ['table', ['tbody', 0]]; },
    },

    table_row: {
      content: 'table_cell+',
      tableRole: 'row',
      parseDOM: [{ tag: 'tr' }],
      toDOM() { return ['tr', 0]; },
    },

    table_cell: {
      content: 'inline*',
      tableRole: 'cell',
      isolating: true,
      parseDOM: [{ tag: 'td' }],
      toDOM() { return ['td', 0]; },
    },

    table_header: {
      content: 'inline*',
      tableRole: 'header_cell',
      isolating: true,
      parseDOM: [{ tag: 'th' }],
      toDOM() { return ['th', 0]; },
    },
  },

  marks: {
    em: {
      parseDOM: [
        { tag: 'i' },
        { tag: 'em' },
        { style: 'font-style=italic' },
      ],
      toDOM() { return ['em', 0]; },
    },

    strong: {
      parseDOM: [
        { tag: 'strong' },
        { tag: 'b' },
        { style: 'font-weight=bold' },
        { style: 'font-weight=700' },
      ],
      toDOM() { return ['strong', 0]; },
    },

    code: {
      parseDOM: [{ tag: 'code' }],
      toDOM() { return ['code', 0]; },
    },

    link: {
      attrs: {
        href: { default: null },
        title: { default: null },
      },
      inclusive: false,
      parseDOM: [
        {
          tag: 'a[href]',
          getAttrs(dom) {
            return { href: dom.getAttribute('href'), title: dom.getAttribute('title') };
          },
        },
      ],
      toDOM(mark) {
        return ['a', { href: mark.attrs.href, title: mark.attrs.title }, 0];
      },
    },

    strikethrough: {
      parseDOM: [{ tag: 's' }, { tag: 'del' }, { tag: 'strike' }],
      toDOM() { return ['s', 0]; },
    },
  },
});

// ─── Markdown parser/serializer ────────────────────────────────────

// We use prosemirror-markdown's default parser/serializer as a base.
// It works with the standard schema, but we can use it with our schema
// since our node types are a superset. For production, we'd build a
// custom parser/serializer that maps our extensions (tables, strikethrough).

const tachyonParser = defaultMarkdownParser;

function serializeToMarkdown(doc) {
  return defaultMarkdownSerializer.serialize(doc);
}

// ─── Input rules ───────────────────────────────────────────────────
// Markdown-style auto-formatting rules.

const tachyonInputRules = inputRules({
  rules: [
    // Heading: "# " at start of line
    textblockTypeInputRule(/^(#{1,6})\s$/, tachyonSchema.nodes.heading,
      (match) => ({ level: match[1].length })),
    // Code block: "```"
    textblockTypeInputRule(/^```$/, tachyonSchema.nodes.code_block),
    // Bullet list: "- " or "* "
    wrappingInputRule(/^\s*([-*])\s$/, tachyonSchema.nodes.bullet_list),
    // Ordered list: "1. "
    wrappingInputRule(/^\s*(\d+)\.\s$/, tachyonSchema.nodes.ordered_list,
      (match) => ({ order: +match[1] })),
    // Blockquote: "> "
    wrappingInputRule(/^\s*>\s$/, tachyonSchema.nodes.blockquote),
    // Horizontal rule: "---" or "***"
    new InputRule(/^---$|^(\*\*\*)$/, (state, match, start, end) => {
      return state.tr.replaceWith(start, end, tachyonSchema.nodes.horizontal_rule.create());
    }),
  ],
});

// ─── Keymap ────────────────────────────────────────────────────────

function buildKeymap() {
  const keys = {};

  // Undo/Redo
  keys['Mod-z'] = undo;
  keys['Mod-y'] = redo;
  keys['Mod-Shift-z'] = redo;

  // Lists
  keys['Enter'] = splitListItem(tachyonSchema.nodes.list_item);
  keys['Mod-['] = liftListItem(tachyonSchema.nodes.list_item);
  keys['Mod-]'] = sinkListItem(tachyonSchema.nodes.list_item);

  // Tables
  keys['Tab'] = goToNextCell(1);
  keys['Shift-Tab'] = goToNextCell(-1);

  return keymap(keys);
}

// ─── OnChange plugin ──────────────────────────────────────────────
// Calls the registered callback whenever the document changes.

function createOnChangePlugin(callback) {
  return new Plugin({
    key: onChangeKey,
    state: {
      init() { return { callback }; },
      apply(tr, value) { return value; },
    },
    view() {
      return {
        update(view, prevState) {
          if (view.state.doc !== prevState.doc) {
            const cb = onChangeKey.getState(view.state)?.callback;
            if (cb) {
              cb(serializeToMarkdown(view.state.doc));
            }
          }
        },
      };
    },
  });
}

// ─── Plugins ───────────────────────────────────────────────────────

function createPlugins(options = {}) {
  return [
    buildKeymap(),
    keymap(baseKeymap),
    history(),
    gapCursor(),
    tableEditing(),
    tachyonInputRules,
    createOnChangePlugin(options.onChange || null),
    createWikilinkPlugin(),
  ];
}

// ─── Public API ────────────────────────────────────────────────────

/**
 * Create a new ProseMirror editor instance.
 *
 * @param {HTMLElement} container - DOM element to mount the editor in.
 * @param {Object} options
 * @param {string} options.content - Initial markdown content.
 * @param {Function} options.onChange - Callback fired on every edit: (markdown: string) => void.
 * @param {boolean} options.editable - Whether the editor is editable (default: true).
 * @param {string} options.placeholder - Placeholder text (default: "Start writing...").
 * @param {string} options.className - Extra CSS class for the editor element.
 * @returns {EditorView} The ProseMirror EditorView instance.
 */
function create(container, options = {}) {
  const {
    content = '',
    onChange,
    editable = true,
    placeholder = 'Start writing...',
    className = '',
  } = options;

  let doc;
  try {
    doc = tachyonParser.parse(content || '');
  } catch (e) {
    console.warn('TachyonEditor: failed to parse markdown, using empty document:', e);
    doc = tachyonSchema.node('doc', null, [
      tachyonSchema.node('paragraph'),
    ]);
  }

  const state = EditorState.create({
    doc,
    plugins: createPlugins({ onChange }),
  });

  const view = new EditorView(container, {
    state,
    editable: () => editable,
    attributes: {
      class: [
        'tachyon-editor',
        'ProseMirror',
        'prose',
        'prose-lg',
        'focus:outline-none',
        'min-h-[200px]',
        'p-4',
        className,
      ].filter(Boolean).join(' '),
      'data-placeholder': placeholder,
    },
  });

  // Track the active editor for toolbar command dispatch
  _activeEditorView = view;

  return view;
}

/**
 * Get the current markdown content from an editor instance.
 * @param {EditorView} view
 * @returns {string}
 */
function getMarkdown(view) {
  return serializeToMarkdown(view.state.doc);
}

/**
 * Replace the editor content with new markdown.
 * @param {EditorView} view
 * @param {string} markdown
 */
function setMarkdown(view, markdown) {
  try {
    const doc = tachyonParser.parse(markdown || '');
    const tr = view.state.tr.replaceWith(0, view.state.doc.content.size, doc.content);
    view.dispatch(tr);
  } catch (e) {
    console.warn('TachyonEditor: failed to parse markdown for setContent:', e);
  }
}

/**
 * Update the onChange callback.
 * @param {EditorView} view
 * @param {Function} callback
 */
function setOnChange(view, callback) {
  const currentState = onChangeKey.getState(view.state);
  if (currentState) {
    const newState = { ...currentState, callback };
    const tr = view.state.tr;
    tr.setMeta(onChangeKey, newState);
    view.dispatch(tr);
  }
}

/**
 * Set whether the editor is editable.
 * @param {EditorView} view
 * @param {boolean} editable
 */
function setEditable(view, editable) {
  view.updateState(view.state.reconfigure({ editable: () => editable }));
}

/**
 * Focus the editor.
 * @param {EditorView} view
 */
function focus(view) {
  view.focus();
}

/**
 * Destroy the editor and clean up DOM.
 * @param {EditorView} view
 */
function destroy(view) {
  if (_activeEditorView === view) {
    _activeEditorView = null;
  }
  view.destroy();
}

// ─── Wikilink Autocomplete Plugin ────────────────────────────────────
// Triggers on [[ and shows a dropdown of document suggestions.

const wikilinkKey = new PluginKey('wikilink');

function createWikilinkPlugin() {
  let open = false;
  let query = '';
  let selectedIndex = 0;
  let suggestions = [];
  let dropdown = null;
  let debounceTimer = null;

  function getApiBaseUrl() {
    // Derive from current page location
    const loc = window.location;
    const origin = loc.origin;
    // If running in Tauri embedded mode, use the embedded server port
    return origin + '/api/v1';
  }

  async function fetchSuggestions(q) {
    if (!q || q.length < 1) {
      suggestions = [];
      return;
    }
    try {
      const base = getApiBaseUrl();
      const url = `${base}/search/suggest?q=${encodeURIComponent(q)}&limit=8`;
      const resp = await fetch(url);
      if (resp.ok) {
        suggestions = await resp.json();
      } else {
        suggestions = [];
      }
    } catch (e) {
      // Silently fail — autocomplete is a nice-to-have
      suggestions = [];
    }
  }

  function createDropdown(view) {
    removeDropdown();
    dropdown = document.createElement('div');
    dropdown.className = 'tachyon-wikilink-dropdown';
    dropdown.style.cssText = 'position:absolute;z-index:100;min-width:220px;max-width:350px;max-height:240px;overflow-y:auto;background:white;border:1px solid #d1d5db;border-radius:0.5rem;box-shadow:0 4px 12px rgba(0,0,0,0.15);font-size:14px;display:none;';
    document.body.appendChild(dropdown);
  }

  function removeDropdown() {
    if (dropdown) {
      dropdown.remove();
      dropdown = null;
    }
    open = false;
    selectedIndex = 0;
  }

  function renderDropdown() {
    if (!dropdown) return;
    if (!open || suggestions.length === 0) {
      dropdown.style.display = 'none';
      return;
    }
    dropdown.style.display = 'block';
    dropdown.innerHTML = suggestions.map((title, i) => {
      const cls = i === selectedIndex
        ? 'background:#eff6ff;color:#1d4ed8;cursor:pointer;padding:6px 12px;'
        : 'background:white;color:#374151;cursor:pointer;padding:6px 12px;';
      return `<div class="tachyon-wl-item" data-index="${i}" style="${cls}">${escapeHtml(title)}</div>`;
    }).join('');

    // Attach click handlers
    dropdown.querySelectorAll('.tachyon-wl-item').forEach(el => {
      el.addEventListener('mousedown', (e) => {
        e.preventDefault();
        e.stopPropagation();
        const idx = parseInt(el.dataset.index, 10);
        selectSuggestion(view, suggestions[idx]);
      });
    });
  }

  function positionDropdown(view) {
    if (!dropdown || !open) return;
    // Get the cursor position in the editor
    const { from } = view.state.selection;
    const coords = view.coordsAtPos(from);
    dropdown.style.left = coords.left + 'px';
    dropdown.style.top = (coords.bottom + 4) + 'px';
  }

  function selectSuggestion(view, title) {
    // Find the [[ marker position
    const { from } = view.state.selection;
    const $head = view.state.selection.$head;
    let bracketStart = from;
    // Search backwards for [[
    for (let i = from - 1; i >= 0; i--) {
      const ch = view.state.doc.textBetween(i, i + 1);
      if (ch === '[' && i > 0 && view.state.doc.textBetween(i - 1, i) === '[') {
        bracketStart = i - 1;
        break;
      }
      if (ch === '\n' || ch === undefined) break;
    }

    // Insert the wikilink: [[Document Title]]
    const tr = view.state.tr
      .delete(bracketStart, from)
      .insertText(`[[${title}]]`, bracketStart);
    view.dispatch(tr);
    removeDropdown();
    view.focus();
  }

  function escapeHtml(str) {
    return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }

  return new Plugin({
    key: wikilinkKey,
    view() {
      return {
        update(view, prevState) {
          // Check if we're inside a [[ ... ]] pattern
          const { from } = view.state.selection;
          const $head = view.state.selection.$head;
          const textBefore = $head.parent.textContent.slice(0, $head.parentOffset);

          // Find last [[ in the current text node
          const lastBracket = textBefore.lastIndexOf('[[');
          if (lastBracket === -1) {
            if (open) removeDropdown();
            return;
          }

          // Extract query after [[
          const afterBracket = textBefore.slice(lastBracket + 2);
          // If there's a ]] after [[, we're not in a wikilink
          if (afterBracket.includes(']]')) {
            if (open) removeDropdown();
            return;
          }

          query = afterBracket.trim();
          open = true;

          // Debounce suggestion fetch
          if (debounceTimer) clearTimeout(debounceTimer);
          debounceTimer = setTimeout(() => {
            fetchSuggestions(query).then(() => {
              selectedIndex = 0;
              createDropdown(view);
              renderDropdown();
              positionDropdown(view);
            });
          }, 150);
        },
        destroy() {
          removeDropdown();
          if (debounceTimer) clearTimeout(debounceTimer);
        },
      };
    },
    props: {
      handleKeyDown(view, event) {
        if (!open) return false;

        if (event.key === 'ArrowDown') {
          event.preventDefault();
          selectedIndex = Math.min(selectedIndex + 1, suggestions.length - 1);
          renderDropdown();
          return true;
        }
        if (event.key === 'ArrowUp') {
          event.preventDefault();
          selectedIndex = Math.max(selectedIndex - 1, 0);
          renderDropdown();
          return true;
        }
        if (event.key === 'Enter' && suggestions.length > 0) {
          event.preventDefault();
          selectSuggestion(view, suggestions[selectedIndex]);
          return true;
        }
        if (event.key === 'Escape') {
          event.preventDefault();
          removeDropdown();
          return true;
        }
        if (event.key === 'Tab' && suggestions.length > 0) {
          event.preventDefault();
          selectSuggestion(view, suggestions[selectedIndex]);
          return true;
        }

        return false;
      },
    },
  });
}

// ─── Global editor reference ──────────────────────────────────────────
// Track the most recently created editor for toolbar command dispatch.
let _activeEditorView = null;

// ─── Export ────────────────────────────────────────────────────────

const TachyonEditor = {
  create,
  getMarkdown,
  setMarkdown,
  setOnChange,
  setEditable,
  focus,
  destroy,
  dispatchCommand(view, commandName) {
    // If view is null/undefined, use the active editor
    const v = view || _activeEditorView;
    if (!v) return false;

    const { state, dispatch } = v;
    const schema = state.schema;

    switch (commandName) {
      case 'bold':
        return toggleMark(schema.marks.strong)(state, dispatch);
      case 'italic':
        return toggleMark(schema.marks.em)(state, dispatch);
      case 'code':
        return toggleMark(schema.marks.code)(state, dispatch);
      case 'strikethrough':
        return toggleMark(schema.marks.strikethrough)(state, dispatch);
      case 'h1':
        return setBlockType(schema.nodes.heading, { level: 1 })(state, dispatch);
      case 'h2':
        return setBlockType(schema.nodes.heading, { level: 2 })(state, dispatch);
      case 'h3':
        return setBlockType(schema.nodes.heading, { level: 3 })(state, dispatch);
      case 'bullet_list':
        return wrapIn(schema.nodes.bullet_list)(state, dispatch);
      case 'ordered_list':
        return wrapIn(schema.nodes.ordered_list)(state, dispatch);
      case 'blockquote':
        return wrapIn(schema.nodes.blockquote)(state, dispatch);
      case 'code_block':
        return setBlockType(schema.nodes.code_block)(state, dispatch);
      case 'horizontal_rule':
        {
          const tr = state.tr.replaceSelectionWith(
            schema.nodes.horizontal_rule.create()
          );
          dispatch(tr);
          return true;
        }
      case 'undo':
        return undo(state, dispatch);
      case 'redo':
        return redo(state, dispatch);
      default:
        console.warn('TachyonEditor: unknown command:', commandName);
        return false;
    }
  },
  schema: tachyonSchema,
};

export default TachyonEditor;
