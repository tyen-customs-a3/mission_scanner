# Mission Scanner Examples

This directory contains example code showing how to use the mission scanner library.

## Basic Example

The `basic.rs` example demonstrates two common use cases:

1. Scanning a single mission directory
2. Scanning multiple missions with custom configuration

To run the example:

```bash
# Set the RUST_LOG environment variable to see debug output
export RUST_LOG=debug

# Run the example
cargo run --example basic
```

Note: You'll need to modify the paths in the example to point to your actual mission directories:
- `path/to/your/mission` - Path to a single mission directory
- `path/to/your/missions` - Path to a directory containing multiple missions

## Example Output

The example will output:
- Number of files found (SQF, CPP/HPP)
- Number of class dependencies
- Example dependencies with their reference types
- For multiple missions, a summary of each mission's dependencies

## Customization

You can modify the example to:
- Change the file extensions to scan
- Adjust the number of threads used
- Add custom filtering of dependencies
- Export results to different formats 