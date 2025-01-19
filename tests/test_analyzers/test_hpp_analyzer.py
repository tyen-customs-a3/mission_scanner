import pytest
from src.mission_scanner.analyzers.hpp_analyzer import HPPAnalyzer

@pytest.fixture
def analyzer():
    return HPPAnalyzer()

def test_class_inheritance(analyzer, sample_data_dir):
    result = analyzer.analyze("sample_data/blufor_loadout.hpp")
    
    # Check base class
    assert "baseMan" in result["classes"]
    # Check derived class
    assert "rm" in result["classes"]

def test_equipment_detection(analyzer, sample_data_dir):
    result = analyzer.analyze("sample_data/blufor_loadout.hpp")
    
    # Check basic equipment
    assert "ItemWatch" in result["equipment"]
    assert "ItemMap" in result["equipment"]
    assert "ItemCompass" in result["equipment"]
    
    # Check weapon references
    assert "CUP_hgun_Mac10" in result["equipment"]
    assert "rhs_weap_rpg7" in result["equipment"]
    
    # Check items with LIST macro
    assert "ACE_fieldDressing" in result["equipment"]
    assert "ACE_packingBandage" in result["equipment"]
    assert "ACE_bloodIV" in result["equipment"]
    
    # Check magazines
    assert "rhs_rpg7_PG7VL_mag" in result["equipment"]
    assert "CUP_30Rnd_45ACP_Green_Tracer_MAC10_M" in result["equipment"]

def test_vest_backpack_equipment(analyzer, sample_data_dir):
    result = analyzer.analyze("sample_data/blufor_loadout.hpp")
    
    # Check vest and backpack
    assert "pca_vest_invisible_plate" in result["equipment"]
    assert "pca_backpack_invisible_large" in result["equipment"]

def test_empty_arrays(analyzer, create_test_file):
    content = """
    class emptyClass {
        uniform[] = {};
        vest[] = {};
        backpack[] = {};
    };
    """
    test_file = create_test_file("empty.hpp", content)
    result = analyzer.analyze(test_file)
    
    assert "emptyClass" in result["classes"]
    assert len(result["equipment"]) == 0
