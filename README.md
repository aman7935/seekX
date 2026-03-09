# seekX

`seekX` is a Rust-based Linux app launcher inspired by Walker/fuzzel flow, with GTK4 UI and optional Wayland layer-shell mode.

## Features

- Fast `.desktop` app discovery from user/system/Flatpak locations
- Search ranking for names, comments, categories, and keywords
- Keyboard-first UX: `Up/Down`, `Enter`, `Alt+Enter`, `Esc`
- GTK4 UI with app icons
- Optional `layer-shell` mode for compositor-style launcher behavior (recommended on Wayland/niri)

## Project layout

- `src/main.rs`: app boot
- `src/desktop.rs`: `.desktop` scanning and parsing
- `src/search.rs`: fuzzy matching/scoring engine
- `src/launcher.rs`: process launch and web search fallback
- `src/ui.rs`: GTK UI and interaction logic

## Requirements

- Rust + Cargo
- GTK4 development libraries
- For `--features layer-shell`: `gtk4-layer-shell` development library

### Fedora (including Fedora Asahi)

```bash
sudo dnf install -y rust cargo gtk4-devel gtk4-layer-shell-devel
```

## Run (development)

```bash
cd ~/project/seekX
cargo run --features layer-shell
```

## Build and run (release)

```bash
cd ~/project/seekX
cargo build --release --features layer-shell
./target/release/seekX
```

## Install as command (`seekX`)

```bash
cd ~/project/seekX
cargo build --release --features layer-shell
mkdir -p ~/.local/bin
ln -sf /home/$USER/project/seekX/target/release/seekX ~/.local/bin/seekX
```

Ensure `~/.local/bin` is on your `PATH`:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

Now launch from terminal:

```bash
seekX
```

## Desktop launcher entry

Create:

`~/.local/share/applications/seekx.desktop`

```ini
[Desktop Entry]
Type=Application
Name=SeekX
Exec=/home/your-user/project/seekX/target/release/seekX
Terminal=false
Categories=Utility;
```

Replace `your-user` with your Linux username.

Refresh desktop entries:

```bash
update-desktop-database ~/.local/share/applications
```

Test launcher entry:

```bash
gtk-launch seekx
```

## Install on another PC

```bash
git clone <your-repo-url> seekX
cd seekX
cargo install --path . --features layer-shell
seekX
```

## Key Controls

- `Type`: filter app list
- `Enter`: launch selected app
- `Alt+Enter`: open web search with current query
- `Enter` (with no matching apps): open web search with current query
- `Esc`: close launcher
