from .base import BaseAnalyzer
from ..utils.file_utils import resolve_sample_path
import re

class HPPAnalyzer(BaseAnalyzer):
    def analyze(self, file_path):
        classes = []
        equipment = set()

        with open(resolve_sample_path(file_path), 'r') as file:
            content = file.read()
            
        # Updated pattern to match class definitions in HPP files
        class_pattern = r'class\s+(\w+)\s*(?::\s*\w+)?\s*{'
        class_matches = re.finditer(class_pattern, content)
        
        for match in class_matches:
            class_name = match.group(1)
            classes.append(class_name)
            
            # Find equipment in the class scope
            class_start = match.end()
            bracket_count = 1
            class_end = class_start
            
            while bracket_count > 0 and class_end < len(content):
                if content[class_end] == '{':
                    bracket_count += 1
                elif content[class_end] == '}':
                    bracket_count -= 1
                class_end += 1
                
            class_content = content[class_start:class_end]
            
            # Find equipment references in arrays
            equipment_matches = re.finditer(r'\"([^\"]+)\"', class_content)
            equipment.update(match.group(1) for match in equipment_matches)
            
            # Find LIST macro references
            list_matches = re.finditer(r'LIST_\d+\(\"([^\"]+)\"\)', class_content)
            equipment.update(match.group(1) for match in list_matches)

        return {
            "file": file_path,
            "classes": classes,
            "equipment": list(equipment)
        }
