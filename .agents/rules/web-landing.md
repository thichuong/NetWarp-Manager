---
trigger: always_on
glob: "public/index.html, public/assets/**/*"
description: Rules for maintaining formatting, structure, styles, and command copying behavior on the WiWarp Web Landing Page (public/index.html).
---

# Web Landing Page Development Rules

These rules ensure the landing page (`public/index.html`) maintains its premium UI design, responsive layout, and correct functional behaviors for command copying and terminal formatting.

## 1. Rule of Terminal Box Formatting & Indentation
To keep the HTML source code readable (indented) without breaking the alignment of the simulated terminal in the browser:
- **Do NOT** set `white-space: pre` or `white-space: pre-wrap` directly on the parent `.terminal-body` element.
- **Wrap every line** inside `.terminal-body` with a `<div class="terminal-line">` block.
- The `.terminal-line` class must have `display: block;` and `white-space: pre;` (or `white-space: pre-wrap;`) applied.
- This allows indentation spaces within the HTML file to be collapsed and ignored by the browser, while preserving spacing and options formatting inside the actual terminal lines.

## 2. Rule of Copy Command Synchronization
The copy button triggers the global `copyCommand(elementId, buttonId)` script.
- Ensure the terminal text elements use proper semantic span classes:
  - `<span class="comment">`: For comment text (ignored by the copier script).
  - `<span class="prompt">`: For prompt characters like `$` or `#` (ignored by the copier script).
  - `<span class="cmd">`: For actual command strings that are copied to the clipboard.
- The JavaScript copier selects `span.cmd` and joins them with newlines. Double-check that commands are correctly wrapped in `<span class="cmd">` so the user copies only what is executable.

## 3. Premium Aesthetic & Styling Guidelines
- **Responsive Layout**: Ensure all containers, grids, and terminal boxes resize gracefully on smaller screens (mobile / tablet).
- **Glassmorphism & Color Palette**: Preserve the unified dark gradient design using `--bg-dark`, `--bg-card`, cyan, blue, and purple accents.
- **Micro-animations**: Keep interactive elements (buttons, links, cards) responsive to `:hover` with smooth transitions (e.g. scale, translate, box-shadow shifts).

## 4. Metadata and SEO
- Maintain valid metadata inside `<head>` (e.g. `<title>`, `<meta name="description">`, OpenGraph tags if any).
- Ensure semantic HTML5 tags (like `<header>`, `<main>`, `<section>`, `<footer>`) are used to partition the landing page structure.
