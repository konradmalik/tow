[![Actions Status](https://github.com/konradmalik/tow/actions/workflows/linting.yml/badge.svg)](https://github.com/konradmalik/tow/actions)
[![Actions Status](https://github.com/konradmalik/tow/actions/workflows/tests.yml/badge.svg)](https://github.com/konradmalik/tow/actions)

# tow

**This is a nowhere near completion, work-in-progress project that I use to learn Rust!**

A tool to install, uninstall and upgrade binaries installed from github releases.

## configuration

Currently only via env variables:

| env var          | Description                          | Default      |
| ---------------- | ------------------------------------ | ------------ |
| TOW_BINARIES_DIR | Directory where to save the binaries | ~/.local/bin |
