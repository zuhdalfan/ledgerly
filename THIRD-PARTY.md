# Third-party components

Ledgerly is MIT licensed (see `LICENSE`). It also redistributes the following.

## Bundled in the application

**Inter** — typeface, by Rasmus Andersson.
SIL Open Font License 1.1. The licence text ships alongside the font at
`static/fonts/OFL.txt`, and the same files are embedded in generated PDFs from
`src-tauri/assets/fonts/`.

No other fonts, images, icons or third-party assets are bundled. The mascot is
drawn as inline SVG in `src/lib/components/Mascot.svelte`.

## Rust dependencies

The application links roughly 460 crates across the three platforms it ships to
(Windows, macOS, Linux). The overwhelming majority are `MIT OR Apache-2.0` or
plain `MIT`, which is the norm in the Rust ecosystem. A handful are
`MPL-2.0` (file-level weak copyleft, from the webview CSS stack) and are used
unmodified.

You can reproduce and verify this yourself:

```bash
npm run check:licenses
```

That script fails the build if anything copyleft (GPL, AGPL, SSPL, BUSL or a
non-commercial Creative Commons licence) appears anywhere in the shipped graph.
It runs in CI.

## JavaScript dependencies

Frontend packages are declared in `package.json` and are build-time only; none
are redistributed in the packaged application.
