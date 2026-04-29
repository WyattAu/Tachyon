use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tachyon_renderer::{RenderConfig, Renderer};

fn generate_medium_md() -> String {
    let mut md = String::from("# Medium Document\n\n");
    md.push_str("This is a medium-sized markdown document for benchmarking.\n\n");
    for i in 0..50 {
        md.push_str(&format!("## Section {}\n\n", i));
        md.push_str("This section contains **bold text**, *italic text*, and `inline code`.\n\n");
        md.push_str("- List item 1\n- List item 2\n- List item 3\n\n");
        md.push_str("| Column A | Column B | Column C |\n|----------|----------|----------|\n");
        for j in 0..5 {
            md.push_str(&format!(
                "| data {}-{} | data {}-{} | data {}-{} |\n",
                i, j, i, j, i, j
            ));
        }
        md.push_str("\n```rust\nfn example() -> u32 {\n    42\n}\n```\n\n");
    }
    md
}

fn generate_large_md() -> String {
    let mut md = String::from("# Large Document\n\n");
    md.push_str("This is a large-sized markdown document for benchmarking.\n\n");
    for i in 0..200 {
        md.push_str(&format!(
            "## Section {}: Heading with **formatting**\n\n",
            i
        ));
        md.push_str(&format!(
            "Paragraph {} with **bold**, *italic*, `code`, [links](https://example.com), and ~~strikethrough~~ text.\n\n",
            i
        ));
        md.push_str("> This is a blockquote for section.\n>\n> With multiple lines.\n\n");
        md.push_str(&format!(
            "1. Ordered item {}\n2. Ordered item {}\n3. Ordered item {}\n\n",
            i * 3,
            i * 3 + 1,
            i * 3 + 2
        ));
        md.push_str("| Header 1 | Header 2 | Header 3 | Header 4 |\n");
        md.push_str("|----------|----------|----------|----------|\n");
        for j in 0..10 {
            md.push_str(&format!(
                "| cell {}-{} | cell {}-{} | cell {}-{} | cell {}-{} |\n",
                i, j, i, j, i, j, i, j
            ));
        }
        md.push_str("\n```python\ndef hello():\n    print(\"world\")\n    return 42\n```\n\n");
    }
    md
}

fn bench_markdown_rendering(c: &mut Criterion) {
    let renderer = Renderer::new(RenderConfig::default());
    let mut group = c.benchmark_group("markdown_rendering");

    let small = "# Title\n\nParagraph with **bold** and *italic* text.\n\n- item 1\n- item 2\n";
    group.bench_function("small", |b| {
        b.iter(|| renderer.render(black_box(small), None));
    });

    let medium = generate_medium_md();
    group.bench_function("medium", |b| {
        b.iter(|| renderer.render(black_box(&medium), None));
    });

    let large = generate_large_md();
    group.bench_function("large", |b| {
        b.iter(|| renderer.render(black_box(&large), None));
    });

    group.finish();
}

criterion_group!(benches, bench_markdown_rendering);
criterion_main!(benches);
