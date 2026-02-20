/**
 * Markdown Editor Component using CodeMirror 6
 */

import { EditorView, basicSetup } from 'codemirror';
import { EditorState, Transaction } from '@codemirror/state';
import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
import { syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language';
import { oneDark } from '@codemirror/theme-one-dark';
import { Events } from '../utils/events';

let editorView: EditorView | null = null;

/**
 * Initialize the markdown editor
 */
export async function initializeEditor(): Promise<void> {
  const editorElement = document.getElementById('editor');
  if (!editorElement) {
    console.warn('Editor element not found');
    return;
  }

  // Get initial content
  const initialContent = editorElement.textContent || '';

  // Determine theme based on current setting
  const isDark = document.documentElement.classList.contains('dark');
  const themeExtensions = isDark ? [oneDark] : [];

  // Create editor state
  const state = EditorState.create({
    doc: initialContent,
    extensions: [
      basicSetup,
      markdown({ base: markdownLanguage }),
      syntaxHighlighting(defaultHighlightStyle),
      ...themeExtensions,
      EditorView.updateListener.of(handleEditorUpdate),
      EditorView.lineWrapping,
    ],
  });

  // Create editor view
  editorView = new EditorView({
    state,
    parent: editorElement,
  });

  // Clear the initial text content
  editorElement.textContent = '';

  // Subscribe to document load events
  window.Tachyon.events.on(Events.DOCUMENT_LOADED, (data: { content: string }) => {
    if (editorView && data.content !== undefined) {
      editorView.dispatch({
        changes: { from: 0, to: editorView.state.doc.length, insert: data.content },
      });
    }
  });

  // Subscribe to theme changes
  window.Tachyon.events.on(Events.THEME_CHANGED, ({ isDark }: { isDark: boolean }) => {
    // Recreate editor with new theme
    if (editorView) {
      const content = editorView.state.doc.toString();
      editorView.destroy();
      
      const newThemeExtensions = isDark ? [oneDark] : [];
      const newState = EditorState.create({
        doc: content,
        extensions: [
          basicSetup,
          markdown({ base: markdownLanguage }),
          syntaxHighlighting(defaultHighlightStyle),
          ...newThemeExtensions,
          EditorView.updateListener.of(handleEditorUpdate),
          EditorView.lineWrapping,
        ],
      });
      
      editorView = new EditorView({
        state: newState,
        parent: editorElement!,
      });
    }
  });

  console.log('Markdown editor initialized');
}

/**
 * Handle editor updates
 */
function handleEditorUpdate(update: any): void {
  if (update.docChanged && editorView) {
    const content = editorView.state.doc.toString();
    
    // Emit document changed event
    window.Tachyon.events.emit(Events.DOCUMENT_CHANGED, {
      content,
      length: content.length,
    });
  }
}

/**
 * Get current editor content
 */
export function getEditorContent(): string {
  return editorView?.state.doc.toString() || '';
}

/**
 * Set editor content
 */
export function setEditorContent(content: string): void {
  if (editorView) {
    editorView.dispatch({
      changes: { from: 0, to: editorView.state.doc.length, insert: content },
    });
  }
}

/**
 * Focus the editor
 */
export function focusEditor(): void {
  editorView?.focus();
}

/**
 * Save current document
 */
export async function saveDocument(): Promise<boolean> {
  const content = getEditorContent();
  const doc = window.Tachyon.currentDocument;
  
  if (!doc) {
    console.warn('No current document to save');
    return false;
  }

  try {
    await window.Tachyon.api.updateDocument(doc.id, { content });
    window.Tachyon.events.emit(Events.DOCUMENT_SAVED, { id: doc.id });
    return true;
  } catch (error) {
    console.error('Failed to save document:', error);
    window.Tachyon.events.emit(Events.ERROR, { message: 'Failed to save document' });
    return false;
  }
}
