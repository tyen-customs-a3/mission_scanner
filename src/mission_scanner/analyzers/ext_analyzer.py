from .base import BaseAnalyzer
import re

class EXTAnalyzer(BaseAnalyzer):
    def analyze(self, file_path):
        classes = []
        equipment = set()

        with open(file_path, 'r') as file:
            lines = file.readlines()

        for line in lines:
            # Match class definitions like "class CfgLoadouts" or "class Success"
            class_match = re.match(r'^\s*class\s+(\w+)', line)
            if class_match:
                classes.append(class_match.group(1))

            # Look for equipment references in includes
            include_match = re.search(r'#include\s+"([^"]+)"', line)
            if include_match:
                # Store included files as they might contain equipment definitions
                equipment.add(include_match.group(1))

        return {
            "file": file_path,
            "classes": classes,
            "equipment": list(equipment)
        }
