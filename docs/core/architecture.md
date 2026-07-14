[← Summary](./SUMMARY.md)

# Architecture

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop runtime | [Tauri 2](https://tauri.app/) |
| Backend | Rust |
| Frontend | Vue 3 + TypeScript |
| Build tool | Vite |
| State management | Pinia |
| Routing | Vue Router |
| Code editor | CodeMirror 6 |
| AWS SDKs | aws-sdk-s3, aws-sdk-cloudformation, aws-sdk-costexplorer, aws-sdk-sts (Rust native) |

## Project Structure

```
easy-cloud/
├── src/                    # Vue frontend
│   ├── assets/            # Global styles (design system)
│   ├── components/        # Reusable components
│   │   ├── ModalDialog.vue
│   │   ├── ToastNotification.vue
│   │   └── S3FileEditor.vue
│   ├── views/             # Page views
│   │   ├── DashboardView.vue
│   │   ├── S3View.vue
│   │   ├── S3BucketView.vue
│   │   ├── CloudFormationView.vue
│   │   └── BillingView.vue
│   ├── store/             # Pinia stores
│   │   ├── awsAccounts.ts
│   │   ├── billing.ts
│   │   └── index.ts
│   ├── router/            # Vue Router config
│   ├── App.vue
│   └── main.ts
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs         # Tauri wiring (plugins + commands)
│   │   └── commands/      # AWS API commands by service
│   │       ├── mod.rs
│   │       ├── sts.rs
│   │       ├── s3.rs
│   │       ├── cloudformation.rs
│   │       ├── billing.rs
│   │       └── dashboard.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docs/                  # Documentation
├── landing/               # Landing page (static HTML)
└── package.json
```

## Backend Architecture

The Rust backend is organized by AWS service:

| Module | SDK | Commands |
|--------|-----|----------|
| `sts.rs` | aws-sdk-sts | verify_aws_credentials |
| `s3.rs` | aws-sdk-s3 | 19 commands (list, upload, download, copy, move, preview, edit, presign, etc.) |
| `cloudformation.rs` | aws-sdk-cloudformation | 21 commands (stacks, events, drift, change sets, exports, template, etc.) |
| `billing.rs` | aws-sdk-costexplorer | 6 commands (cost summary, daily, forecast, usage, report) |
| `dashboard.rs` | aws-sdk-s3 + cloudformation | 1 consolidated summary command |

Each command:
1. Builds an AWS client with the provided credentials
2. Makes the API call
3. Returns a serialized response struct

## Frontend Architecture

### State Management
- **awsAccounts store** — manages accounts, active selection, localStorage persistence
- **billing store** — manages cost data with per-account 2h TTL cache

### Communication
Frontend ↔ Backend via Tauri's `invoke()`:
```typescript
const res = await invoke('command_name', { param1, param2 });
```

### Design System
All styling uses CSS custom properties defined in `src/assets/styles.css`:
- Dark mode only (Vercel-inspired)
- Glass morphism effects
- Consistent spacing, radius, transitions
- No CSS framework — vanilla CSS with design tokens

## Security Model

```
[User's Machine]
    │
    ├── Easy Cloud (desktop app)
    │   ├── Credentials (memory + localStorage)
    │   └── AWS SDK calls ──────────► [AWS APIs]
    │
    └── No external servers
```

- Credentials never leave the machine
- No analytics, telemetry, or tracking
- No proxy servers or cloud functions
- Direct machine → AWS communication only
