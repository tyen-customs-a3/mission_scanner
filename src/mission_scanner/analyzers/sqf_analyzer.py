from .base import BaseAnalyzer
import re

class SQFAnalyzer(BaseAnalyzer):
    def analyze(self, file_path):
        equipment = set()

        with open(file_path, 'r') as file:
            lines = file.readlines()

        for line in lines:
            equipment_match = re.findall(r'\"([^\"]+)\"', line)
            if equipment_match:
                equipment.update(equipment_match)

        return {
            "file": file_path,
            "classes": [],  # No class definitions in .sqf files
            "equipment": list(equipment)
        }
