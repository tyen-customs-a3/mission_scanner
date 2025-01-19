import argparse
from pathlib import Path
from .scanner import Scanner

def main():
    parser = argparse.ArgumentParser(description='Scan mission files for classes and equipment')
    parser.add_argument('path', type=str, help='Path to scan (file or directory)')
    parser.add_argument('--format', choices=['text', 'json'], default='text', help='Output format')
    parser.add_argument('--output', '-o', type=str, help='Output file path')
    
    args = parser.parse_args()
    scanner = Scanner()
    
    path = Path(args.path)
    if path.is_file():
        results = [scanner.scan(str(path))]
    else:
        results = scanner.scan_directory(str(path))
    
    if args.format == 'json':
        import json
        output = json.dumps(results, indent=2)
    else:
        output = format_text_output(results)
    
    if args.output:
        with open(args.output, 'w') as f:
            f.write(output)
    else:
        print(output)

def format_text_output(results):
    output = []
    for result in results:
        output.append(f"\nFile: {result['file']}")
        output.append("Classes:")
        for cls in result['classes']:
            output.append(f"  - {cls}")
        output.append("Equipment:")
        for eq in result['equipment']:
            output.append(f"  - {eq}")
    return '\n'.join(output)

if __name__ == '__main__':
    main()
