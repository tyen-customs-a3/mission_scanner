from .analyzers.sqf_analyzer import SQFAnalyzer
from .analyzers.hpp_analyzer import HPPAnalyzer
from .analyzers.ext_analyzer import EXTAnalyzer
from .utils.file_utils import get_file_type
import os

class Scanner:
    def __init__(self):
        self.analyzers = {
            'sqf': SQFAnalyzer(),
            'hpp': HPPAnalyzer(),
            'ext': EXTAnalyzer()
        }

    def scan(self, file_path):
        file_type = get_file_type(file_path)
        analyzer = self.analyzers.get(file_type)
        if analyzer:
            return analyzer.analyze(file_path)
        else:
            raise ValueError(f"No analyzer found for file type: {file_type}")

    def scan_multiple(self, file_paths):
        results = []
        for file_path in file_paths:
            try:
                result = self.scan(file_path)
                results.append(result)
            except Exception as e:
                results.append({
                    "file": file_path,
                    "error": str(e),
                    "classes": [],
                    "equipment": []
                })
        return results

    def scan_directory(self, directory_path):
        file_paths = []
        for root, _, files in os.walk(directory_path):
            for file in files:
                ext = get_file_type(file)
                if ext in self.analyzers:
                    file_paths.append(os.path.join(root, file))
        return self.scan_multiple(file_paths)

# Example usage
if __name__ == "__main__":
    scanner = Scanner()
    result = scanner.scan("sample_data/arsenal.sqf")
    print(result)
