<p align=center>
<img width="256" height="256" alt="opsis icon" src="https://github.com/MrKowo/Opsis/blob/main/assets/branding/logo.png" />
</p>

# Opsis

> **_Opsis_**: From Ancient Greek _ὄψις_, meaning aspect, appearance, vision, spectacle.

Opsis is a Fast, minimalist and mudular image viewer built in rust. It's built from the ground up to be portable and support community plugins to extend its functionality in order to perfectly adapt to your needs. Nothing more, nothing less. 

**AI use disclosure**: Opsis is built making large use of LLM technology. Human contributions will always be welcome!

---

## Building

clone this repo with `git clone https://github.com/MrKowo/Opsis.git`

change the working directory to the fnewly created folder

```bash
cd Opsis
```

Start building:

```bash
# Release build (recommended for performance, longer build times)
cargo build --release

# Debug build (fast builds, with terminal logging)
cargo build
```

> Right now only Windows builds are available, Linux and macOS are planned in the near future.

## Running

the compiled binary will be located at `./target/release/opsis.exe`. This can be moved and run from any location on your machine, but it is recommende to put it into your dedicated application folder or program files.

```bash
# From command line
cargo run --release

# Open a specific file
cargo run --release -- path\to\image.png
```

## Dependencies

- `eframe` - egui native application framework
- `egui` - Immediate mode GUI
- `image` - Image loading and decoding
- `rfd` - Cross-platform file dialogs
- `pollster` - Async runtime for Windows
- `serde` - Serialization
- `toml` - Configuration file parsing
- `env_logger` - Logging

## Acknowledgments

Thank you to [Oculante](https://github.com/woelper/oculante) for providing the base and inspiration for this project!
