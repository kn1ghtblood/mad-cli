# MAD-cli

> A simple media download tool.

## 🛠 Build Instructions

### 1. FFMPEG Libraries
This project requires custom FFMPEG-based libraries to process the final output. You may use prebuilt shared libraries or compile them manually.

**Environment Setup:**
Set the `FFMPEG_DIR` environment variable to point to your library path before building:
```bash
export FFMPEG_DIR="/path/to/the/library"
```
### 2. Build the Binary

Once the environment is configured, use Cargo to compile the project.

**Standard Build**

For native desktop builds, run:

```bash
cargo build --release
```

**Android Build**

To cross-compile for Android, ensure **FFMPEG_DIR** is set and use **cargo-ndk**:

```bash
cargo ndk -t <target_architecture> -p <target_android_platform> build --release
```