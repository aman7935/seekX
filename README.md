# seekX

## Clone and run

```bash
# Arch Linux
sudo pacman -S --needed rust cargo gtk4 gtk4-layer-shell

git clone https://github.com/aman7935/seekX.git seekX
cd seekX
cargo run --features layer-shell
```

## Web search (no hardcoded engine)

seekX opens links in your system default browser. By default, web searches use DuckDuckGo. To use a different engine, set a search URL template via an env var:

```bash
# examples:
export SEEKX_SEARCH_URL_TEMPLATE='https://www.google.com/search?q={query}'
export SEEKX_SEARCH_URL_TEMPLATE='https://duckduckgo.com/?q={query}'
export SEEKX_SEARCH_URL_TEMPLATE='https://search.brave.com/search?q={query}'
```

The template can use `{query}` or `%s` as the placeholder (it will be URL-encoded).

## Add to Applications menu and terminal

Build once:

```bash
cd seekX
cargo build --release --features layer-shell
```

Create desktop entry:

```bash
mkdir -p ~/.local/share/applications
cat > ~/.local/share/applications/seekx.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=SeekX
Exec=/home/your-user/seekX/target/release/seekX
Terminal=false
Categories=Utility;
EOF
```

Replace `your-user` with your Linux username.

To run from terminal as `seekX`:

```bash
mkdir -p ~/.local/bin
ln -sf /home/$USER/seekX/target/release/seekX ~/.local/bin/seekX
```
