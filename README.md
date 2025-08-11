# DMG EMU

A gameboy emulator project

## Prerequisites

This emulator currently works only with a Tetris ROM file. You must place a Tetris ROM file named `tetris.gb` in the `resources/` directory before running the emulator.

## Setup

1. Place your Tetris ROM file in the resources directory:
   ```
   resources/tetris.gb
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

## Running the Emulator

To launch the emulator:
```bash
cargo run --release
```

## Controls

- **WASD**: Directional pad (Up/Down/Left/Right)
- **Space**: A button
- **X**: B button
- **Enter**: Start button
- **C**: Select button
- **P**: Pause/Resume
- **F1**: Toggle debug view

## Documentation

- Opcodes : https://gbdev.io/gb-opcodes/optables/
