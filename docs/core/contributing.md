[← Summary](./SUMMARY.md)

# Contributing

Thank you for considering contributing to Easy Cloud! Every contribution makes cloud access easier for someone.

## How to Contribute

### 1. Fork & Clone

```bash
git clone https://github.com/YOUR-USERNAME/easy-cloud.git
cd easy-cloud
npm install
```

### 2. Create a Branch

```bash
git checkout -b feature/my-feature
```

### 3. Make Changes

- **Frontend** → files in `src/`
- **Backend (Rust)** → files in `src-tauri/src/`
- **Docs** → files in `docs/`

### 4. Test

```bash
# Frontend type check
npx vue-tsc --noEmit

# Frontend build
npx vite build

# Backend check
cd src-tauri && cargo check
```

### 5. Submit PR

Push your branch and open a Pull Request against `main`.

## Development Setup

```bash
# Install dependencies
npm install

# Run in dev mode (hot reload)
npm run tauri dev
```

First run takes a few minutes for Rust compilation. Subsequent runs are fast.

## Code Style

### Frontend (TypeScript/Vue)
- Vue 3 Composition API with `<script setup>`
- TypeScript strict mode
- No CSS framework — use design tokens from `styles.css`
- Match existing patterns in the codebase

### Backend (Rust)
- One module per AWS service in `src/commands/`
- Each command is a `pub async fn` with `#[tauri::command]`
- Return `Result<ResponseStruct, String>`
- Use helper functions (e.g., `build_s3_client`) to reduce boilerplate
- Register new commands in `lib.rs`

## Adding a New AWS Service

1. Create `src-tauri/src/commands/your_service.rs`
2. Add `pub mod your_service;` to `commands/mod.rs`
3. Add `use commands::your_service::*;` to `lib.rs`
4. Register commands in `generate_handler![]`
5. Add the SDK crate to `Cargo.toml`:
   ```toml
   aws-sdk-yourservice = { version = "1", default-features = false, features = ["rustls"] }
   ```
6. Create the Vue view in `src/views/`
7. Add the route in `src/router/index.ts`
8. Add the nav link in `src/App.vue`

## Adding a New Frontend Feature

1. Create or modify the view in `src/views/`
2. If needed, create a Pinia store in `src/store/`
3. Use `invoke('command_name', { ... })` to call backend
4. Follow existing UI patterns (glass panels, badges, modals, context menus)
5. Run `npx vue-tsc --noEmit` to verify types

## Issues & Feature Requests

- Check [existing issues](https://github.com/Andrezz64/easy-cloud/issues) first
- Use labels: `bug`, `feature`, `docs`, `good first issue`
- Provide clear description and steps to reproduce (for bugs)

## License

By contributing, you agree that your contributions will be licensed under the AGPL-3.0 License.
