from ..utils.file_utils import read_file_lines

class BaseAnalyzer:
    def analyze(self, file_path):
        try:
            lines = read_file_lines(file_path)
            return self._analyze_content(file_path, lines)
        except Exception as e:
            return {
                "file": file_path,
                "classes": [],
                "equipment": [],
                "error": str(e)
            }

    def _analyze_content(self, file_path, lines):
        raise NotImplementedError("Subclasses should implement this method")
