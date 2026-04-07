# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 回复语言

必须使用中文回复.

## Project Overview

Kiro One is a Tauri 2.0 desktop application for managing AI accounts, API gateways, and provider configurations. It combines a React frontend with a Rust backend, integrating functionality from three projects: kiro-account-manager, KiroaaS, and cc-switch.

## Development Commands

```bash
# Start development mode (frontend + Tauri desktop app)
npm run tauri:dev

# Build production desktop application
npm run tauri:build

# Frontend only (for UI development, but Tauri commands won't work)
npm run dev

# Install dependencies
npm install
```

**Important**: This is a Tauri desktop application. Always use `npm run tauri:dev` for full functionality. Running `npm run dev` alone will start the frontend but Tauri invoke commands will fail.

## Architecture

### Frontend-Backend Communication

The app uses Tauri's invoke system for all frontend-backend communication:

- **Frontend**: React components call Tauri commands via `invoke()` from `@tauri-apps/api/core`
- **Backend**: Rust functions marked with `#[tauri::command]` in `src-tauri/src/main.rs`
- **Wrapper**: `src/lib/tauri.ts` provides a compatibility layer for Tauri 2.0

All 17 Tauri commands are defined in a single file: `src-tauri/src/main.rs` (355 lines).

### State Management

**Critical**: All data is stored in-memory using Rust `HashMap` wrapped in `Mutex`. There is NO persistent storage or database. Data is lost when the application restarts.

The `AppState` struct contains three HashMaps:
- `accounts`: Account data (email, tokens, usage)
- `providers`: AI provider configurations
- `gateway_config`: API gateway settings

### Component Structure

React components are organized by feature:
- `src/components/accounts/` - Account CRUD, import/export, sync
- `src/components/gateway/` - API gateway configuration
- `src/components/settings/` - Provider management and switching
- `src/components/dashboard/` - Statistics and quick actions
- `src/components/layout/` - Sidebar navigation
- `src/components/ui/` - Reusable UI components

Routes are defined in `src/App.tsx` using React Router.

## Adding New Features

### Adding a New Tauri Command

1. Add the Rust function in `src-tauri/src/main.rs`:
```rust
#[tauri::command]
fn my_command(state: tauri::State<AppState>, param: String) -> Result<String, String> {
    // Implementation
    Ok("success".to_string())
}
```

2. Register it in the `main()` function's `invoke_handler`:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    my_command,
])
```

3. Call it from React:
```typescript
import { invoke } from "@tauri-apps/api/core";
const result = await invoke<string>("my_command", { param: "value" });
```

### Adding a New Page

1. Create component in `src/components/[feature]/`
2. Add route in `src/App.tsx`
3. Add navigation link in `src/components/layout/Sidebar.tsx`

## Key Patterns

### Tauri Invoke Calls

Always use the custom wrapper from `src/lib/tauri.ts` for better error handling:
```typescript
import { invoke } from "@/lib/tauri";
```

### Data Serialization

Rust structs use `#[serde(rename_all = "camelCase")]` to match JavaScript naming conventions. Frontend TypeScript interfaces should use camelCase.

### Account Import/Export

The import system supports multiple field name formats (camelCase and snake_case) for compatibility with different export sources. See `import_accounts_command` in main.rs.

## Kiro IDE Integration

The app can import accounts from Kiro IDE by reading `~/.aws/sso/cache/kiro-auth-token.json` (cross-platform path resolution). This is handled by the `import_from_kiro_ide` command.

## Configuration Files

- `src-tauri/tauri.conf.json` - Tauri app configuration (window size, bundle settings)
- `vite.config.ts` - Vite dev server runs on port 1420
- `tailwind.config.js` - TailwindCSS with custom theme extensions
- `package.json` - Frontend dependencies and npm scripts
- `src-tauri/Cargo.toml` - Rust dependencies

## Important Constraints

1. **No Persistent Storage**: All data is in-memory. Restarting the app clears everything.
2. **Single Backend File**: All Tauri commands are in one file (main.rs). Consider refactoring if adding many new commands.
3. **Mock Data**: The `sync_account` command returns mock usage data, not real API calls.
4. **No API Gateway Implementation**: Gateway configuration exists but the actual HTTP server is not implemented yet.

## Tauri 2.0 Specifics

This project uses Tauri 2.0, which has breaking changes from 1.x:
- Use `@tauri-apps/api/core` for invoke (not `@tauri-apps/api/tauri`)
- File system operations use `@tauri-apps/plugin-fs`
- Dialog operations use `@tauri-apps/plugin-dialog`
- The custom wrapper in `src/lib/tauri.ts` handles compatibility checks
