# Mission Scanner

A Python module for analyzing mission files and extracting class definitions and equipment references.

## Purpose

This module scans Arma mission files (`.sqf`, `.hpp`, `.ext`) to:
- Extract class definitions
- Identify equipment references
- Track dependencies between files
- Provide a comprehensive report of mission content

## Features

- Multi-format file scanning
- Class definition extraction
- Equipment reference detection
- Directory recursive scanning
- Detailed reporting
- Test coverage

## File Type Support

### SQF Files
- Equipment references in string literals
- Array contents
- Function parameters

### HPP Files
- Class definitions
- Equipment in arrays
- LIST_X macro expansions
- Included file references

### EXT Files
- Configuration classes
- Include directives
- Equipment references

## Usage

```python
from mission_scanner.scanner import Scanner

# Initialize scanner
scanner = Scanner()

# Scan single file
result = scanner.scan("path/to/file.sqf")

# Scan multiple files
results = scanner.scan_multiple([
    "arsenal.sqf",
    "blufor_loadout.hpp",
    "description.ext"
])

# Scan entire directory
results = scanner.scan_directory("mission_folder")
```

## Example Output

```python
{
    "file": "arsenal.sqf",
    "classes": [],
    "equipment": [
        "Tarkov_Uniforms_10",
        "1Rnd_SmokeBlue_Grenade_shell",
        "ACE_fieldDressing"
    ]
}
```

## Development

### Requirements
- Python 3.7+
- pytest
- pytest-cov

### Setup
```bash
pip install -r requirements.txt
pip install -e .
```

### Running Tests
```bash
pytest tests/
```

### VS Code Integration
- Install Python extension
- Open command palette (Ctrl+Shift+P)
- Select "Python: Configure Tests"
- Choose "pytest"
- Run tests from Testing sidebar

## Project Structure

```
mission_scanner/
├── src/
│   └── mission_scanner/
│       ├── scanner.py
│       ├── analyzers/
│       │   ├── sqf_analyzer.py
│       │   ├── hpp_analyzer.py
│       │   └── ext_analyzer.py
│       └── utils/
│           └── file_utils.py
├── tests/
│   ├── test_scanner.py
│   └── test_analyzers/
│       ├── test_sqf_analyzer.py
│       ├── test_hpp_analyzer.py
│       └── test_ext_analyzer.py
└── sample_data/
    ├── arsenal.sqf
    ├── blufor_loadout.hpp
    └── description.ext
```