# tilde
A minimal, keyboard-driven web browser built with GTK4 and WebKit, designed around a fast command palette and Vim-inspired navigation.

## Overview
tilde is an experimental browser that prioritizes speed, simplicity, and keyboard-first interaction. Instead of traditional UI elements like tab bars and address bars, tilde centers everything around a command palette and intuitive shortcuts.
It blends ideas from tools like Vim, Spotlight, and modern command palettes to create a focused browsing experience.

## Features
### Command Palette
- Triggered with ~
- Search open tabs
- Open URLs directly
- Perform web searches (DuckDuckGo)
- Execute internal commands (:q, :reload, etc.)
### Vim-like Navigation
- j / k → smooth scroll
- f → hint mode (click elements via keyboard)
- Shift + H / Shift + L → back / forward
- Shift + J / Shift + K → cycle tabs
### Tabs
- Multiple tabs via GtkNotebook
- Fast switching through shortcuts or palette
- Close tab with x
### Dock (Status Bar)
- Floating bottom dock
- Displays:
  - current profile
  - active URI
  - tab count
### Performance Instrumentation
- Page load time tracking
- First Contentful Paint (FCP)
- Time to Interactive (TTI)
- Memory usage tracking
- CPU usage estimation
### Smart Input Handling
- Detects editable fields (input, textarea, contenteditable)
- Disables shortcuts when typing
- Escape key blurs focused inputs
### Built-in Hint Mode
- Keyboard-driven link selection (inspired by Vimium)
- Overlay hints on clickable elements
- Fully injected via JavaScript

## Tech Stack
- Rust
- GTK4
- WebKit6
- glib / gio
- Custom JavaScript injection

## Keybindings
| Key | Action |
|-----|--------|
| `~` | Toggle command palette |
| `f` | Enter hint mode |
| `j / k` | Scroll down / up |
| `Shift + H` | Go back |
| `Shift + L` | Go forward |
| `Shift + J` | Next tab |
| `Shift + K` | Previous tab |
| `x` | Close current tab |
| `r` | Reload page |
| `Esc` | Close palette / blur input |


## Commands
Inside the command palette:
- :q → Quit
- :r → Reload
- :d → Close tab

## Building
### Requirements
- Rust (stable)
- GTK4 development libraries
- WebKitGTK (webkit6)
- pkg-config
### Build & Run
cargo run


## Design Philosophy
- tilde is built around a few core ideas:
- Keyboard-first: Everything should be accessible without a mouse
- Minimal UI: No clutter, only essential overlays
- Fast interaction: Command palette as the central interface
- Hackable: Simple structure, easy to extend


