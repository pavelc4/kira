<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/Svelte-4A5548?style=for-the-badge&logo=svelte&logoColor=white" alt="Svelte">
  <img src="https://img.shields.io/badge/Tauri-222222?style=for-the-badge&logo=tauri&logoColor=white" alt="Tauri">
  <a href="LICENSE"><img src="https://img.shields.io/badge/GPL--v3-white?style=for-the-badge&logo=gnu&logoColor=white&label=License&labelColor=222" alt="License"></a>
</p>


<h1 align="center">Kira</h1>

<p align="center">
  A modern Android device management tool built with Rust, Tauri, and Svelte.
  <br>
  <em>Currently in active development.</em>
</p>

---

## About

**Kira** is a desktop application designed to make Android device management simple and efficient. Built on a performant Rust core with a clean Svelte frontend, Kira aims to be the go-to tool for developers and power users alike.

## Features

| Feature | Description |
|---|---|
| **Device Dashboard** | Real-time device status and overview |
| **Mi Flash** | Xiaomi ROM flashing support |
| **Live Logcat** | Stream Android logs in real-time |
| **File Manager** | Push and pull files with ease |
| **App Manager** | Install and uninstall APKs |
| **Batch Commands** | Automate workflows with bulk operations |

## Quick Start

```bash
git clone https://github.com/pavelc4/kira
cd kira
bun install
cargo tauri dev
```

## Platform Support

| Platform | Status |
|---|---|
| Linux (Arch) | In Development |
| Windows | Planned |
| macOS | Planned |
| Android | Planned |

## Tech Stack

```
Core:  Rust + adb_client + fastboot-protocol
UI:    Tauri  + Svelte + TypeScript
Build: Bun + Vite
```

## Resources

- [Releases](https://github.com/pavelc4/kira/releases)
- [Issues](https://github.com/pavelc4/kira/issues)

## License

Kira is open-source software licensed under the [GNU GPL v3](LICENSE).
