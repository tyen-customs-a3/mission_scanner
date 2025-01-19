import pytest
from src.mission_scanner.analyzers.ext_analyzer import EXTAnalyzer

@pytest.fixture
def analyzer():
    return EXTAnalyzer()

def test_config_classes(analyzer, sample_data_dir):
    result = analyzer.analyze("sample_data/description.ext")
    
    # Check main configuration classes
    assert "CfgDebriefing" in result["classes"]
    assert "CfgLoadouts" in result["classes"]
    assert "CfgFunctions" in result["classes"]

def test_included_files(analyzer, sample_data_dir):
    result = analyzer.analyze("sample_data/description.ext")
    
    # Check included files
    assert "loadouts\\_macros.hpp" in result["equipment"]
    assert "loadouts\\blufor_loadout.hpp" in result["equipment"]
    assert "loadouts\\opfor_loadout.hpp" in result["equipment"]

def test_event_handlers(analyzer, sample_data_dir):
    result = analyzer.analyze("sample_data/description.ext")
    
    # Check event handler classes
    assert "Extended_PreInit_EventHandlers" in result["classes"]
    assert "Extended_PostInit_EventHandlers" in result["classes"]

def test_mission_settings(analyzer, create_test_file):
    content = """
    tmf_version[] = {1,1,1};
    cba_settings_hasSettingsFile = 1;
    enableDebugConsole = 1;
    """
    test_file = create_test_file("settings.ext", content)
    result = analyzer.analyze(test_file)
    
    assert len(result["classes"]) == 0  # No classes in this content
    assert len(result["equipment"]) == 0  # No equipment references
