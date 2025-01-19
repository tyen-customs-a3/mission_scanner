import os
import logging

def get_file_type(file_path):
    _, ext = os.path.splitext(file_path)
    return ext.lstrip('.').lower()

def validate_file(file_path):
    if not os.path.exists(file_path):
        raise FileNotFoundError(f"File not found: {file_path}")
    if not os.path.isfile(file_path):
        raise ValueError(f"Path is not a file: {file_path}")
    return True

def read_file_lines(file_path):
    validate_file(file_path)
    try:
        with open(file_path, 'r', encoding='utf-8') as file:
            return file.readlines()
    except UnicodeDecodeError:
        logging.warning(f"UTF-8 decode failed for {file_path}, trying with 'latin-1'")
        with open(file_path, 'r', encoding='latin-1') as file:
            return file.readlines()

def resolve_sample_path(file_path):
    """Resolves the path to sample data files"""
    if os.path.exists(file_path):
        return file_path
        
    # Try to find in sample_data directory
    sample_path = os.path.join(os.path.dirname(__file__), '..', '..', '..', 'sample_data', os.path.basename(file_path))
    if os.path.exists(sample_path):
        return sample_path
        
    raise FileNotFoundError(f"Could not find file: {file_path}")
