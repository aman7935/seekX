pub const EMBEDDED_CSS: &str = r#"
window.seekx-window,
window.seekx-window.background,
window.seekx-window > * {
  background-color: transparent;
  background: none;
  border-radius: 0 !important;
}

/* FORCE RECTANGLES EVERYWHERE */
* {
  border-radius: 0 !important;
  outline: none;
  box-shadow: none;
}

*:focus,
*:focus-visible,
*:selected {
  outline: none;
  box-shadow: none;
  border-radius: 0 !important;
}

.seekx-outer {
  background-color: transparent;
  background: none;
  border-radius: 0 !important;
}

.seekx-search-box {
  background-color: rgba(0, 0, 0, 0.75);
  border: 1px solid #ffffff;
  border-radius: 0 !important;
  padding: 10px 18px;
}

.seekx-results-box {
  background-color: rgba(0, 0, 0, 0.75);
  border: 1px solid #ffffff;
  border-radius: 0 !important;
  padding: 10px 16px;
}

entry.seekx-entry,
entry.seekx-entry text {
  background: transparent;
  color: #ffffff;
  border: none;
  border-radius: 0 !important;
  font-size: 18px;
  font-weight: 500;
  box-shadow: none;
  outline: none;
}

entry.seekx-entry {
  min-height: 40px;
  padding: 0 4px;
  border-radius: 0 !important;
}

list.seekx-list {
  background: transparent;
  border: none;
  border-radius: 0 !important;
}

row.seekx-row {
  background-color: transparent;
  border: none;
  border-radius: 0 !important;
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
  border-radius: 0 !important;
}

row.seekx-row:selected:hover {
  background-color: #4d4d4d;
}

scrolledwindow.seekx-scroll,
scrolledwindow.seekx-scroll > viewport,
scrolledwindow.seekx-scroll > viewport > * {
  background: transparent;
  border: none;
  box-shadow: none;
  border-radius: 0 !important;
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
"#;
