# seekX - App Launcher for Linux

## Apps and Web Search

seekX can search your installed apps as well as the web. App results update as you type, and queries that look like URLs or when you invoke the web search action open in whatever browser the system reports as the default.

## Prerequisites

Ensure you have Rust/Cargo plus the GTK4 development stack (`libgtk-4-dev`/`gtk4-devel`, `pkg-config`, `glib`, etc.). If you plan to enable the layer shell integration, install `gtk4-layer-shell` (or the distro’s `-dev` package) before building.

On Debian/Ubuntu:
```
sudo apt install build-essential pkg-config libgtk-4-dev libglib2.0-dev
```
On Fedora:
```
sudo dnf install gcc-c++ pkgconf-pkg-config gtk4-devel
```
On Arch Linux:
```
sudo pacman -S --needed rust cargo gtk4 gtk4-layer-shell
```

## Clone and run

```bash
git clone https://github.com/aman7935/seekX.git seekX
cd seekX
cargo build --release --features layer-shell
./target/release/seekX
```
