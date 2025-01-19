import pytest
from src.mission_scanner.scanner import Scanner
import os

@pytest.fixture
def scanner():
    return Scanner()

def test_scan_directory(scanner, sample_data_dir):
    results = scanner.scan_directory(sample_data_dir)
    assert len(results) > 0
    assert all(isinstance(result, dict) for result in results)
    assert all("error" not in result for result in results)

def test_scan_multiple_file_types(scanner, sample_data_dir):
    files = [
        os.path.join(sample_data_dir, "arsenal.sqf"),
        os.path.join(sample_data_dir, "blufor_loadout.hpp"),
        os.path.join(sample_data_dir, "description.ext")
    ]
    results = scanner.scan_multiple(files)
    assert len(results) == 3
    
    # Verify each file type's specific content
    sqf_result = next(r for r in results if r["file"].endswith(".sqf"))
    assert len(sqf_result["equipment"]) > 0
    assert len(sqf_result["classes"]) == 0

    hpp_result = next(r for r in results if r["file"].endswith(".hpp"))
    assert len(hpp_result["classes"]) > 0
    assert len(hpp_result["equipment"]) > 0

    ext_result = next(r for r in results if r["file"].endswith(".ext"))
    assert "CfgLoadouts" in ext_result["classes"]

def test_error_handling(scanner, temp_test_dir):
    nonexistent_file = os.path.join(temp_test_dir, "nonexistent.sqf")
    with pytest.raises(FileNotFoundError):
        scanner.scan(nonexistent_file)

def test_invalid_file_type(scanner, create_test_file):
    invalid_file = create_test_file("test.invalid", "some content")
    with pytest.raises(ValueError, match="No analyzer found for file type"):
        scanner.scan(invalid_file)
