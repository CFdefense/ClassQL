# ClassQL

A terminal-based query language interface for class data.

## Custom Font Modifications

This project includes custom modifications to the `text-to-ascii-art` crate to improve the CLASSQL logo rendering. The modifications are included as a vendored dependency in the `vendor/` directory.

### What's Modified

- **Font rendering**: Custom adjustments to the default font for better logo display
- **Character spacing**: Improved spacing for the CLASSQL logo
- **Visual consistency**: Enhanced appearance in terminal environments

### How It Works

The project uses Cargo's patch feature to override the `text-to-ascii-art` dependency with a local modified version. This ensures that anyone who clones the repository will get the same custom font modifications.

### Building

```bash
cargo build
```

### Running

```bash
cargo run
```

The custom font modifications will be automatically applied when building the project.
