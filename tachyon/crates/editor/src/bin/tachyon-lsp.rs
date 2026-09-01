use std::path::PathBuf;

fn main() {
    let mut args = std::env::args_os().skip(1);
    let mut root = PathBuf::from(".");
    while let Some(arg) = args.next() {
        if arg == "--root" {
            root = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));
        }
    }

    if let Err(error) = tachyon_editor::lsp_server::run_stdio(root) {
        eprintln!("tachyon-lsp: {error}");
        std::process::exit(1);
    }
}
