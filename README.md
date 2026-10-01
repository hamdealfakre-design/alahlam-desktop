# ALAHLAM Desktop (Svelte + Tauri + SQLite)

This repository contains a starter Tauri desktop app wrapping a Svelte frontend and a bundled SQLite database for a hotel management system.

Requirements:
- Node.js 18+
- Rust (rustup) + cargo
- npm

Quick start:
1. Clone the repository
2. Install dependencies:

```bash
npm install
```

3. Run the app in development (requires Rust installed):

```bash
npm run tauri:dev
```

4. Build release bundles:

```bash
npm run build
npm run tauri:build
```

The SQLite DB is stored under `./data/alahlam.db` when running locally.
