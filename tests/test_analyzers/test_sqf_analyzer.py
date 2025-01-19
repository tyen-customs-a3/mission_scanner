import pytest
from src.mission_scanner.analyzers.sqf_analyzer import SQFAnalyzer

@pytest.fixture
def analyzer():
    return SQFAnalyzer()

def test_equipment_extraction(analyzer, create_test_file):
    content = '''
    items[] = {
        "Tarkov_Uniforms_10",
        "1Rnd_SmokeBlue_Grenade_shell",
        "ACE_fieldDressing"
    };
    '''
    test_file = create_test_file("test.sqf", content)
    result = analyzer.analyze(test_file)
    
    assert "Tarkov_Uniforms_10" in result["equipment"]
    assert "1Rnd_SmokeBlue_Grenade_shell" in result["equipment"]
    assert "ACE_fieldDressing" in result["equipment"]
    assert len(result["classes"]) == 0

def test_empty_file(analyzer, create_test_file):
    test_file = create_test_file("empty.sqf", "")
    result = analyzer.analyze(test_file)
    assert len(result["equipment"]) == 0
    assert len(result["classes"]) == 0

def test_uniforms_detection(analyzer, sample_data_dir):
    result = analyzer.analyze("sample_data/arsenal.sqf")
    
    # Check uniform references
    uniforms = [item for item in result["equipment"] if item.startswith("Tarkov_Uniforms_")]
    assert len(uniforms) > 0
    assert "Tarkov_Uniforms_10" in uniforms
    assert "Tarkov_Uniforms_359" in uniforms

def test_weapon_accessories(analyzer, sample_data_dir):
    result = analyzer.analyze("sample_data/arsenal.sqf")
    
    # Check weapon accessories
    assert "rhsusf_acc_eotech_552" in result["equipment"]
    assert "rhsusf_acc_compm4" in result["equipment"]
    assert "rhsusf_acc_grip1" in result["equipment"]

def test_ammunition_types(analyzer, sample_data_dir):
    result = analyzer.analyze("sample_data/arsenal.sqf")
    
    # Check ammunition
    assert "rhs_mag_30Rnd_556x45_M855A1_Stanag" in result["equipment"]
    assert "1Rnd_HE_Grenade_shell" in result["equipment"]
    
    # Check smoke grenades
    smokes = [item for item in result["equipment"] if "Smoke" in item]
    assert len(smokes) > 0
    assert "1Rnd_SmokeBlue_Grenade_shell" in smokes

def test_radio_equipment(analyzer, sample_data_dir):
    result = analyzer.analyze("sample_data/arsenal.sqf")
    
    # Check radio equipment
    assert "ACRE_PRC343" in result["equipment"]
    assert "ACRE_PRC148" in result["equipment"]
    assert "ACRE_PRC152" in result["equipment"]
