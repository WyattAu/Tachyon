// Build script for Tachyon editor JS bundle.
// Usage: node build.mjs [--watch]
//
// Bundles the ProseMirror-based editor into public/editor.js
// which is served by Trunk as a static asset.

import * as esbuild from 'esbuild';

const isWatch = process.argv.includes('--watch');

const buildOptions = {
  entryPoints: ['src/index.js'],
  bundle: true,
  outfile: '../public/editor.js',
  format: 'iife',        // Immediately-invoked function expression (no module system needed)
  globalName: 'TachyonEditor', // Exposes window.TachyonEditor
  target: ['es2020'],
  minify: !isWatch,
  sourcemap: !isWatch ? false : 'inline',
  logLevel: 'info',
};

if (isWatch) {
  const ctx = await esbuild.context(buildOptions);
  await ctx.watch();
  console.log('Watching for changes...');
} else {
  await esbuild.build(buildOptions);
  console.log('Build complete: public/editor.js');
}
