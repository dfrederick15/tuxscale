# Tuxscale

A native Linux GUI for the [Tailscale](https://tailscale.com) VPN client, built entirely in Rust using [iced](https://github.com/iced-rs/iced).

Tuxscale does not bundle or replace Tailscale — it wraps the `tailscale` CLI already installed on your system, providing a clean desktop interface with a system tray icon.

---

## Features

- **Dashboard** — connection status, your Tailscale IP, one-click connect/disconnect
- **Peers** — live list of all tailnet nodes with online/offline status, ping, and copy IP
- **Exit Nodes** — browse and activate exit nodes on your tailnet
- **Netcheck** — run a network diagnostics report with DERP region latencies
- **Options** — toggle key Tailscale settings (Accept DNS, Accept Routes, Shields Up, SSH server, Exit Node LAN access, Advertise as Exit Node)
- **System tray** — minimize to tray, show/hide window, quit from tray menu
- **Auto-refresh** — status polls every 5 seconds automatically

## Screenshot

> *(coming soon)*

## Requirements

- Linux (X11 or Wayland)
- [Tailscale](https://tailscale.com/download/linux) installed at `/usr/bin/tailscale`
- System libraries: `libayatana-appindicator3`, `libgtk-3`

### Install system dependencies (Debian/Ubuntu)

```sh
sudo apt install libayatana-appindicator3-1 libgtk-3-0 libgtk-3-dev libayatana-appindicator3-dev
```

## Installation

### From crates.io

```sh
cargo install tuxscale
```

### From source

```sh
git clone https://github.com/dfrederick15/tuxscale
cd tuxscale
cargo build --release
./target/release/tuxscale
```

## How it works

Tuxscale contains zero Tailscale source code. It communicates entirely by spawning the `tailscale` CLI binary as a subprocess and parsing its output:

| Action | CLI call |
|---|---|
| Status | `tailscale status --json` |
| Connect | `tailscale up` |
| Disconnect | `tailscale down` |
| Ping peer | `tailscale ping <ip>` |
| Netcheck | `tailscale netcheck --format=json` |
| Exit nodes | `tailscale exit-node list` |
| Set option | `tailscale set --flag=value` |
| Read prefs | `tailscale debug prefs` |

## License

MIT — see [LICENSE](LICENSE)
