# seekX - App Launcher for Linux

seekX is a fast and minimal **application launcher for Linux** that allows you to search:

* Installed applications
* Files and directories
* The web

Results update **instantly while you type**, making it a fast way to launch apps, open files, or search the web.

---

# Features

## 🔎 Search Installed Applications

Start typing the name of an installed application and seekX will instantly show matching results.

Example:

```
firefox
code
discord
```

---

## 📁 Search Directories

Use `/` before a path to search directories directly.

Example:

```
/bin
/usr
/home
```

---

## 📄 Search Files

Use `//` to search files in the system.

Example:

```
/image.png
/document.pdf
/script.sh
```

---

## 🌐 Web Search

seekX always provides an option to search your query on the web.

Even if:

* the app exists in the system
* the file exists in the system

you can still choose to **search the same query on the web**.

---

## 🌍 Smart URL Detection

If the query looks like a URL, seekX automatically opens it in the **system's default browser**.

Example:

```
google.com
github.com
```

---

## ⚡ Instant Results

Results update **as you type**, giving a fast and smooth launcher experience.

---

# Installation

## Prerequisites

You need **Rust / Cargo** and the **GTK4 development stack**.

If you want **layer shell support**, install `gtk4-layer-shell`.

---

## Debian / Ubuntu

```
sudo apt install build-essential pkg-config libgtk-4-dev libglib2.0-dev
```

---

## Fedora

```
sudo dnf install gcc-c++ pkgconf-pkg-config gtk4-devel
```

---

## Arch Linux

```
sudo pacman -S --needed rust cargo gtk4 gtk4-layer-shell
```

---

# Clone and Run

Clone the repository and build seekX.

```
git clone https://github.com/aman7935/seekX.git seekX
cd seekX

cargo build --release --features layer-shell

./target/release/seekX
```

---

# Upcoming Features

More features are coming soon.

---

