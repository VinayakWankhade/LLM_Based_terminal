# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

This is a modern terminal emulator built from scratch using Rust (Tauri) for the backend and React/TypeScript for the frontend. It replicates key features of the original Warp terminal including AI integration, shell hooks, workflows, and modern terminal emulation.

## Development Commands

### Build & Development
- **Start development server**: `cargo tauri dev` (runs both frontend and backend)
- **Frontend only**: `npm run dev` (Vite dev server on port 3000)
- **Build for production**: `cargo tauri build`
- **Install dependencies**: `npm install` (frontend) and `cd src-tauri && cargo check` (backend)

### Testing
- **Rust backend tests**: `cd src-tauri && cargo test`
- **Frontend build check**: `npm run build`
- **TypeScript type checking**: `npx tsc --noEmit`

### Code Quality
- **Rust formatting**: `cd src-tauri && cargo fmt`
- **Rust linting**: `cd src-tauri && cargo clippy`
- **Frontend formatting**: `npx prettier --write src/`

## Architecture Overview

### Backend Structure (Rust + Tauri)
The Rust backend is organized into specialized modules:

- **`terminal.rs`**: Core terminal grid state, character rendering, and ANSI command execution
- **`pty.rs`**: Pseudoterminal management and process spawning (cross-platform)
- **`ansi.rs`**: VT100/ANSI escape sequence parsing and command generation
- **`shell_hooks.rs`**: Smart shell integration (command history, prompt detection, autocompletion)
- **`commands.rs`**: Tauri IPC commands exposing backend functionality to frontend
- **`ai.rs`**: AI client supporting both mock and OpenAI-compatible providers
- **`workflows.rs`**: User-defined command workflows with parameterization
- **`search.rs`**: Terminal scrollback and command history search
- **`settings.rs`**: Application settings and configuration management

### Frontend Structure (React + TypeScript)
- **`App.tsx`**: Main application with tab management, pane splitting, and global shortcuts
- **`components/Terminal.tsx`**: Core terminal display using xterm.js
- **`components/PaneLayout.tsx`**: Recursive pane splitting layout system
- **`components/AIPanel.tsx`**: AI assistant panel with command generation
- **`components/WorkflowsPanel.tsx`**: Workflow management and execution interface
- **`components/CommandHistory.tsx`**: Interactive command history browser
- **`components/SettingsPanel.tsx`**: Application settings and theme configuration

### Key Design Patterns

**State Management**: The terminal manager maintains all terminal sessions with async Rust channels for I/O. Frontend uses React state with Tauri IPC for backend communication.

**Pane System**: Recursive tree structure (`PaneNode`) enables arbitrary terminal splitting (vertical/horizontal). Each leaf node contains a terminal session ID.

**Shell Integration**: Output parsing identifies prompts, commands, and working directories. Maintains command history with metadata (timestamp, duration, exit code) for intelligent autocompletion.

**AI Integration**: Configurable AI providers (mock/OpenAI-compatible) with context-aware command generation, error explanation, and next-step suggestions.

## Development Workflow

### Adding New Features
1. **Backend**: Add new module in `src-tauri/src/`, expose via commands in `commands.rs`, register in `lib.rs`
2. **Frontend**: Create component in `src/components/`, integrate with main App.tsx
3. **IPC**: Use `#[tauri::command]` for backend functions, `invoke()` from frontend

### Terminal Emulation Changes
- Modify `ansi.rs` for new escape sequences
- Update `terminal.rs` for grid operations
- Test with various shells and ANSI test suites

### Shell Integration Enhancements
- Extend prompt patterns in `shell_hooks.rs`
- Add new command detection logic
- Update completion algorithms

## Environment Setup

### Prerequisites
- **Rust**: Latest stable (1.77.2+)
- **Node.js**: 20.19+ or 22.12+
- **Tauri CLI**: Install via `cargo install tauri-cli`

### Configuration Files
- **`tauri.conf.json`**: Tauri app configuration, window settings, security policies
- **`Cargo.toml`**: Rust dependencies and build configuration
- **`package.json`**: Node.js dependencies and npm scripts
- **`vite.config.ts`**: Frontend build configuration with Tauri integration

### AI Configuration
Set environment variables for AI features:
```bash
AI_PROVIDER=openai-compatible
AI_BASE_URL=https://api.openai.com/v1
AI_API_KEY=your_api_key
AI_MODEL=gpt-4o-mini
```

## Key Implementation Details

### PTY Management
Cross-platform terminal process spawning using `portable-pty` crate. Handles shell detection, environment setup, and bidirectional I/O with proper signal handling.

### ANSI Processing
VTE-based parser processes terminal output into commands. Maintains character attributes (colors, styles) and cursor state. Supports scrollback buffer and search.

### IPC Architecture
Tauri's async runtime manages terminal I/O in background tasks. Output events stream to frontend via `emit()`. Commands use stateful terminal manager with Arc<Mutex<>> for thread safety.

### Workflow System
YAML-like workflows with parameter substitution (`{{param}}`). Stored in `~/.warp-terminal/workflows.json` with plugin system for extensibility.

## Common Debugging

### Terminal Issues
- Check PTY process status and shell executable paths
- Verify ANSI parser handles escape sequences correctly
- Monitor terminal grid state and cursor positioning

### Frontend Issues
- Inspect Tauri IPC communication in browser dev tools
- Check React component state and re-rendering patterns
- Verify xterm.js integration and terminal sizing

### AI Integration
- Test with mock provider first before real API
- Check request/response formats and error handling
- Verify context extraction from terminal state

## Architecture Principles

- **Performance**: Async I/O with efficient terminal rendering
- **Cross-platform**: Works on Windows, macOS, and Linux
- **Extensibility**: Plugin system and modular architecture
- **Modern UX**: React components with keyboard shortcuts and themes
- **Shell Intelligence**: Context-aware features that understand shell state

This terminal serves as both a functional terminal emulator and a demonstration of modern terminal architecture patterns.
