import pytest
import os
import tempfile

@pytest.fixture
def sample_data_dir():
    return os.path.join(os.path.dirname(__file__), '..', 'sample_data')

@pytest.fixture
def temp_test_dir():
    with tempfile.TemporaryDirectory() as tmpdirname:
        yield tmpdirname

@pytest.fixture
def create_test_file(temp_test_dir):
    def _create_file(filename, content):
        filepath = os.path.join(temp_test_dir, filename)
        with open(filepath, 'w') as f:
            f.write(content)
        return filepath
    return _create_file
