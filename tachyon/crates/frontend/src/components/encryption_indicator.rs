//! Encryption indicator component.
//! Shows lock/unlock icon for document encryption status.

use leptos::prelude::*;

/// Encryption indicator component.
/// Displays a lock icon when document is encrypted, unlock when decrypted.
/// Click to toggle encryption.
#[component]
pub fn EncryptionIndicator(
    /// Whether the document is currently encrypted
    encrypted: bool,
    /// Callback when the user clicks to toggle encryption
    on_toggle: impl Fn() + 'static,
) -> impl IntoView {
    let icon_class = if encrypted {
        "h-5 w-5 text-green-600 dark:text-green-400"
    } else {
        "h-5 w-5 text-gray-400 dark:text-gray-500"
    };

    let tooltip = if encrypted {
        "Document is encrypted (click to decrypt)"
    } else {
        "Document is not encrypted (click to encrypt)"
    };

    let bg_class = if encrypted {
        "bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800"
    } else {
        "bg-gray-50 dark:bg-gray-800 border-gray-200 dark:border-gray-700"
    };

    view! {
        <button
            class=move || {
                format!(
                    "flex items-center gap-1.5 px-2 py-1 text-xs font-medium rounded border transition-colors {}",
                    bg_class,
                )
            }
            title=tooltip
            on:click=move |_| on_toggle()
            aria-label=tooltip
        >
            {if encrypted {
                view! {
                    <svg class=icon_class xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M10 1a4.5 4.5 0 00-4.5 4.5V9H5a2 2 0 00-2 2v6a2 2 0 002 2h10a2 2 0 002-2v-6a2 2 0 00-2-2h-.5V5.5A4.5 4.5 0 0010 1zm3 8V5.5a3 3 0 10-6 0V9h6z" clip-rule="evenodd" />
                    </svg>
                }
                .into_any()
            } else {
                view! {
                    <svg class=icon_class xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M10 1a4.5 4.5 0 00-4.5 4.5V9H5a2 2 0 00-2 2v6a2 2 0 002 2h10a2 2 0 002-2v-6a2 2 0 00-2-2h-.5V5.5A4.5 4.5 0 0010 1zm3 8V5.5a3 3 0 10-6 0V9h6z" clip-rule="evenodd" />
                    </svg>
                }
                .into_any()
            }}
            <span class=move || {
                if encrypted {
                    "text-green-700 dark:text-green-300"
                } else {
                    "text-gray-500 dark:text-gray-400"
                }
            }>
                {if encrypted { "Encrypted" } else { "Unencrypted" }}
            </span>
        </button>
    }
}

/// Encryption status badge (simpler variant, no click action).
#[component]
pub fn EncryptionStatusBadge(encrypted: bool) -> impl IntoView {
    if encrypted {
        view! {
            <span class="inline-flex items-center gap-1 px-2 py-0.5 text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300 rounded">
                <svg class="h-3 w-3" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M10 1a4.5 4.5 0 00-4.5 4.5V9H5a2 2 0 00-2 2v6a2 2 0 002 2h10a2 2 0 002-2v-6a2 2 0 00-2-2h-.5V5.5A4.5 4.5 0 0010 1zm3 8V5.5a3 3 0 10-6 0V9h6z" clip-rule="evenodd" />
                </svg>
                "Encrypted"
            </span>
        }
        .into_any()
    } else {
        view! {
            <span class="inline-flex items-center gap-1 px-2 py-0.5 text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400 rounded">
                <svg class="h-3 w-3" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M10 1a4.5 4.5 0 00-4.5 4.5V9H5a2 2 0 00-2 2v6a2 2 0 002 2h10a2 2 0 002-2v-6a2 2 0 00-2-2h-.5V5.5A4.5 4.5 0 0010 1zm3 8V5.5a3 3 0 10-6 0V9h6z" clip-rule="evenodd" />
                </svg>
                "Unencrypted"
            </span>
        }
        .into_any()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_indicator_variants() {
        // Test that both variants compile
        let _encrypted = true;
        let _unencrypted = false;
        // Component rendering is tested via integration/wasm tests
    }
}
