pub const EMBEDDED_CSS: &str = r#"
window.seekx-window,
window.seekx-window.background,
window.seekx-window > * {
  background-color: transparent;
  background: none;
}

*,
*:focus,
*:focus-visible,
*:selected {
  outline: none;
  box-shadow: none;
}

.seekx-outer {
  background-color: transparent;
  background: none;
}

.seekx-search-box {
  background-color: #000000;
  background-color: rgba(0, 0, 0, 0.9);
  border: 1px solid #ffffff;
  border-radius: 14px;
  padding: 10px 18px;
}

.seekx-results-box {
  background-color: #000000;
  background-color: rgba(0, 0, 0, 0.75);
  border: 1px solid #ffffff;
  border-radius: 14px;
  padding: 10px 16px;
}

entry.seekx-entry,
entry.seekx-entry text {
  background: transparent;
  color: #ffffff;
  border: none;
  border-radius: 0;
  font-size: 18px;
  font-weight: 500;
  box-shadow: none;
  outline: none;
}

entry.seekx-entry {
  min-height: 40px;
  padding: 0 4px;
}

entry.seekx-entry:focus {
  outline: none;
  box-shadow: none;
  border: none;
}

entry.seekx-entry:focus-visible,
row.seekx-row:focus,
row.seekx-row:focus-visible,
list.seekx-list:focus,
list.seekx-list:focus-visible,
scrolledwindow.seekx-scroll:focus,
scrolledwindow.seekx-scroll:focus-visible {
  outline: none;
  box-shadow: none;
}

scrolledwindow.seekx-scroll,
scrolledwindow.seekx-scroll > viewport,
scrolledwindow.seekx-scroll > viewport > * {
  background: transparent;
  border: none;
  box-shadow: none;
}

scrolledwindow.seekx-scroll scrollbar {
  background: transparent;
  border: none;
}

scrolledwindow.seekx-scroll scrollbar slider {
  background-color: #ffffff;
  border-radius: 99px;
  min-width: 4px;
  min-height: 24px;
}

scrolledwindow.seekx-scroll scrollbar slider:hover {
  background-color: #cccccc;
}

list.seekx-list {
  background: transparent;
  border: none;
}

row.seekx-row {
  background-color: transparent;
  border: none;
  border-radius: 8px;
  margin-top: 1px;
  margin-bottom: 1px;
  padding: 8px 10px;
}

row.seekx-row:hover {
  background-color: #1a1a1a;
}

row.seekx-row:selected {
  background-color: #333333;
  border: none;
}

row.seekx-row:selected:hover {
  background-color: #4d4d4d;
}

label.seekx-label {
  color: #cccccc;
  font-size: 14px;
  font-weight: 400;
}

label.seekx-path {
  color: #808080;
  font-size: 11px;
  font-weight: 300;
}

row.seekx-row:selected label.seekx-label {
  color: #ffffff;
  font-weight: 500;
}

label.seekx-web-label {
  font-weight: bold;
  color: #8ab4f8;
}

row.seekx-row:selected label.seekx-web-label {
  color: #d2e3fc;
}

label.seekx-status {
  color: #808080;
  font-size: 11px;
  font-weight: 300;
  padding-top: 2px;
  padding-bottom: 4px;
  padding-left: 4px;
}

@keyframes skeleton-shimmer {
  0% {
    background-position: -200% 0;
  }
  100% {
    background-position: 200% 0;
  }
}

.seekx-skeleton-icon,
.seekx-skeleton-text {
  background: linear-gradient(90deg, #1a1a1a 25%, #333333 50%, #1a1a1a 75%);
  background-size: 200% 100%;
  animation: skeleton-shimmer 1.5s infinite linear;
}

.seekx-skeleton-icon {
  border-radius: 6px;
  min-width: 32px;
  min-height: 32px;
}

.seekx-skeleton-text {
  border-radius: 4px;
  margin-top: 8px;
  margin-bottom: 8px;
  max-width: 200px;
}
"#;
