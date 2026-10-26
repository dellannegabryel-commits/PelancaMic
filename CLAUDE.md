# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**PelancaMic** - Android smartphone → PC wireless studio microphone with zero latency. A monorepo with two native modules communicating over LAN:

| Module | Language | Framework | Role |
|--------|----------|-----------|------|
| **android/** | Kotlin | Amper + Compose Multiplatform | Audio capture (AudioRecord), UDP streaming, UI |
| **pc/** | Rust + TypeScript | Tauri v3 + React 18 | UDP receive, Jitter Buffer, virtual audio cable injection (cpal) |

### Android Module Structure (`android/`)
```
android/
├── androidApp/          # Android app entry point (MainActivity)
├── shared/              # KMP library (Compose UI, platform abstraction)
│   ├── src/             # Common code (expect/actual)
│   └── src@android/     # Android actual implementations
├── libs.versions.toml   # Version catalog
├── project.yaml         # Amper project config
├── module.yaml          # Module configs
└── kotlin(.bat)         # Amper wrapper script
```

- **Build system**: Amper (Kotlin CLI v0.13.0), not Gradle
- **UI**: Compose Multiplatform with Material 3
- **Architecture**: `expect`/`actual` pattern for platform-specific code (e.g., `Platform` interface)

## Network Topology (3 Independent Channels)

| Service | Protocol | Port | Direction |
|---------|----------|------|-----------|
| Discovery | UDP Broadcast | 41234 | PC → Android (beacon every 1s) |
| Audio | UDP Unicast | 41235 | Android → PC (10ms PCM frames, 16-byte header) |
| Control | WebSocket (TCP) | 41236 | Bidirectional (mute, volume, ping/pong) |

**Audio Frame Format** (Big-Endian, 16-byte header + 960-byte payload):
- `0-1`: Magic `0x4D53` (`MS`)
- `2`: Version `1`
- `3`: Flags (bit 0 = muted, bit 1 = silence)
- `4-7`: Sequence (u32, increments per frame)
- `8-11`: Capture timestamp (u32, for jitter calc)
- `12-13`: Payload length (usually 960)
- `14`: Channels (`1`)
- `15`: Codec (`0` = PCM)

Sample rate: **48 kHz**, mono, 16-bit PCM, **10 ms frames** (480 samples = 960 bytes).

## Development Commands

### PC (Server) — `pc/`
```bash
cd pc
npm install              # Install frontend deps
npm run tauri dev        # Dev server with hot reload (Rust + React)
npm run tauri build      # Production build
cargo test               # Rust unit tests (in src-tauri/)
```

### Android (Client) — `android/`
Open `android/` in **Android Studio (Koala+)**. Requires **physical device** (emulators block UDP broadcast).
- Run: `Run 'androidApp'` from toolbar
- USB or Wi-Fi debugging

### Amper (Kotlin CLI) Commands
```bash
cd android
./kotlin amper build     # Build all modules
./kotlin amper test      # Run tests
./kotlin amper sync      # Sync dependencies
./kotlin amper generate  # Generate code (resources, etc.)
```

## Architecture Highlights

### PC Backend (Rust, `pc/src-tauri/src/`)
- **tokio** for async UDP/WS servers
- **cpal** for cross-platform audio output (finds virtual cable by name: `CABLE Input (VB-Audio)` on Windows)
- **Adaptive Jitter Buffer**: target 3–8 frames (30–80 ms), auto-adjusts based on network conditions, reorders by sequence number

### Android (Kotlin, `android/`)
- **Amper** build system (Kotlin CLI v0.13.0), not Gradle
- **Compose Multiplatform** + **Material 3** for UI
- **expect/actual** pattern: common code in `shared/src/`, Android impls in `shared/src@android/`
- **AudioRecord** direct API (not MediaRecorder) for minimal latency — to be implemented in `shared/src@android/`
- **DatagramSocket** for UDP streaming — to be implemented
- **okhttp** WebSocket for control channel — to be implemented
- **Haze** (glassmorphism/backdrop blur) — to be added
- **Dynamic Color** (Material You) via `dynamicColorScheme` — to be added

## Key Files to Know

| Area | Path |
|------|------|
| PC audio sink + jitter buffer | `pc/src-tauri/src/audio/` |
| PC UDP/WS servers | `pc/src-tauri/src/network/` |
| Android app entry | `android/androidApp/src/MainActivity.kt` |
| Shared UI (Compose) | `android/shared/src/App.kt` |
| Platform abstraction | `android/shared/src/Platform.kt` + `src@android/Platform.android.kt` |
| Amper config | `android/project.yaml`, `android/*/module.yaml` |
| Version catalog | `android/libs.versions.toml` |
| Shared protocol constants | Defined in both codebases (keep in sync) |

## Prerequisites

- **PC**: Rust (`rustup`), Node.js 20+, **VB-CABLE** (Windows) or PipeWire loopback (Linux)
- **Android**: Android Studio Koala+, physical device (USB/Wi-Fi debug), Amper wrapper (`android/kotlin`)

## Common Tasks

| Task | Command / Action |
|------|------------------|
| Run PC dev server | `cd pc && npm run tauri dev` |
| Build PC release | `cd pc && npm run tauri build` |
| Run Rust tests | `cd pc/src-tauri && cargo test` |
| Run Android app | Open `android/` in Android Studio → Run `androidApp` |
| Build Android (CLI) | `cd android && ./kotlin amper build` |
| Sync protocol changes | Update header struct in both Rust and Kotlin |

## Notes

- The `pc/` directory is **not yet created** in the working tree — will be scaffolded. The README describes the intended structure.
- Protocol changes require updates in **both** codebases (Rust deserialization ↔ Kotlin serialization).
- Virtual audio cable must be installed and selected as input in target apps (Discord, OBS, games).
- Android uses **Amper** (Kotlin CLI), not Gradle — commands differ from standard Android projects.

## Git & Commit Practices

- **Use `gh` CLI**: Prefer `gh` for GitHub operations (PRs, issues, repo actions).
- **No co-authorship**: Do not add `Co-Authored-By` lines to commit messages.
- **Conventional commits**: Use `type(scope): description` format (e.g., `feat(audio): add jitter buffer auto-tune`).
- **Types**: `feat`, `fix`, `refactor`, `perf`, `docs`, `test`, `chore`, `build`, `ci`.
- **Scope**: Module or component affected (`audio`, `network`, `ui`, `android`, `pc`, `proto`).
- **Breaking changes**: Append `!` after type/scope and add `BREAKING CHANGE:` in body.
- **Keep commits atomic**: One logical change per commit.