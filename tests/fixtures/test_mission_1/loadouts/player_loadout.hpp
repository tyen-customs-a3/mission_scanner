/*
	*Example Loadout for players
	*This is an example loadout made to use as a template for creating loadouts.
	*Faction: Example NATO Forces (MTP)
*/

// Weaponless Baseclass
class baseMan 
{
	displayName = "Unarmed";
	// All Randomized.
	uniform[] = 
	{
		"U_I_L_Uniform_01_tshirt_sport_F",
		"U_I_L_Uniform_01_tshirt_skull_F",
		"CUP_I_B_PMC_Unit_19",
		"CUP_I_B_PMC_Unit_12",
		"U_I_C_Soldier_Para_1_F"
	};
	vest[] = 
	{
		"V_HarnessOGL_brn"
	};
	backpack[] = {
		"aegis_carryall_blk"
	};
	headgear[] = 
	{
		"aegis_beanie_red"
	};
	goggles[] = {};
	hmd[] = {};
	// Leave empty to not change faces and Insignias -> example: faces[] = {};
	faces[] = 
	{

	};
	insignias[] = {};
	
	//All Randomized. Add Primary Weapon and attachments.
	//Leave Empty to remove all. {"Default"} for using original items the character start with.
	primaryWeapon[] = {};
	scope[] = {};
	bipod[] = {};
	attachment[] = {};
	silencer[] = {};
	
	// *WARNING* secondaryAttachments[] arrays are NOT randomized.
	secondaryWeapon[] = {};
	secondaryAttachments[] = {};
	sidearmWeapon[] = {};
	sidearmAttachments[] = {};
	
	// These are added to the uniform or vest first - overflow goes to backpack if there's any.
	magazines[] = {};
	items[] = 
	{
		//Standard Items used in our sessions.
		LIST_10("ACE_fieldDressing"),
		LIST_5("ACE_packingBandage"),
		LIST_5("ACE_quikclot"),
		LIST_4("ACE_tourniquet"),
		LIST_2("ACE_morphine"),
		LIST_2("ACE_splint")
	};
	
	// These are added directly into their respective slots
	linkedItems[] = 
	{
		"ItemWatch",
		"ItemMap",
		"ItemCompass"
	};
	
	// These are put directly into the backpack.
	backpackItems[] = {};
	
	// This is executed after the unit init is complete. Argument: _this = _unit.
	code = "";
};


class r : baseMan
{
	displayName = "Rifleman";
	primaryWeapon[] = 
	{
		"rhs_weap_m1garand_sa43"
	};
	magazines[] = 
	{
		LIST_2("SmokeShell"),
		LIST_2("rhs_mag_m67"),
		LIST_13("rhsgref_8Rnd_762x63_M2B_M1rifle")
	};
};


class ar : r
{
	displayName = "Machinegunner";
	vest[] = 
	{
		"usm_vest_lbe_machinegunner"
	};
	backpack[] = 
	{
		"aegis_carryall_blk"
	};
	primaryWeapon[] = 
	{
		"CUP_lmg_PKM"	
	};
	
	sidearmWeapon[] =
	{
		"rhs_weap_makarov_pm"
	};
	magazines[] =
	{
		
		LIST_4("rhs_mag_9x18_8_57N181S")
	};
	backpackItems[] = 
	{
		LIST_4("rhs_100Rnd_762x54mmR"),
	};
};

class aar : r
{
	displayName = "Assistant MG";
	backpack[] = 
	{
		"aegis_carryall_blk"
	};
	backpackItems[] += 
	{
		LIST_4("rhs_100Rnd_762x54mmR")
	};
	linkedItems[] += 
	{
		"Binocular"
	};
};

class g : r 
{
	displayName = "Grenadier";
	vest[] = 
	{
		"usm_vest_lbe_gr"
	};
	backpack[] = 
	{
		"B_AssaultPack_mcamo"
	};
	primaryWeapon[] =
	{
		"rhs_weap_m79"
	};
	sidearmWeapon[] =
	{
		"CUP_hgun_TEC9"
	};
	bipod[] = {};
	magazines[] += 
	{
		LIST_10("CUP_32Rnd_9x19_TEC9"),
		LIST_10("rhs_mag_M441_HE"),
		LIST_3("rhs_mag_m576"),
		LIST_5("rhs_mag_m714_White")
	};
};

class rat : r
{
	displayName = "anti-material rifleman";
	primaryWeapon[] = 
	{
		"rhs_weap_m82a1"
	};
	scope[] = 
	{
		"sma_acc_vortex_blk"
	};
	sidearmWeapon[] =
	{
		"CUP_hgun_TEC9"
	};
	magazines[] = 
	{
		LIST_2("SmokeShell"),
		LIST_2("rhs_mag_m67"),
		LIST_2("rhsusf_mag_10Rnd_STD_50BMG_M33"),
		LIST_2("rhsusf_mag_10Rnd_STD_50BMG_mk211"),
		LIST_10("CUP_32Rnd_9x19_TEC9")
	};
};

class aamr : r
{
	displayName = "assistant anti-material rifleman";
	backpack[] = 
	{
		"aegis_carryall_blk"
	};
	magazines[] +=
	{
		LIST_3("rhsusf_mag_10Rnd_STD_50BMG_M33"),
		LIST_3("rhsusf_mag_10Rnd_STD_50BMG_mk211")
	};
	linkedItems[] += 
	{
		"Binocular"
	};
};

class tl : r
{
	displayName = "Team Leader";

	primaryWeapon[] = 
	{
		"rhs_weap_akms"
	};

	magazines[] = 
	{
		LIST_4("SmokeShell"),
		LIST_2("rhs_mag_m67"),
		LIST_5("rhs_30Rnd_762x39mm_bakelite"),
		LIST_5("rhs_30Rnd_762x39mm_bakelite_tracer"),
	};
	items[] += 
	{
		"ACRE_PRC148"
	};
	linkedItems[] += 
	{
		"Binocular"
	};
};

class sl : tl
{
	displayName = "Squad Leader";
	backpack[] = 
	{
		"B_TacticalPack_mcamo"
	};
	primaryWeapon[] = 
	{
		"rhs_weap_akms"
	};
	magazines[] = 
	{
		LIST_2("SmokeShell"),
		LIST_2("rhs_mag_m67"),
		LIST_5("rhs_30Rnd_762x39mm_bakelite"),
		LIST_5("rhs_30Rnd_762x39mm_bakelite_tracer")
	};
};

class r_fa : r
{
	displayName = "Rifleman (First-Aid)";
	traits[] = {"medic"};
	backpack[] = 
	{
		"rhsusf_assault_eagleaiii_ocp"
	};
	backpackItems[] +=
	{
		LIST_30("ACE_elasticBandage"),
		LIST_10("ACE_epinephrine"),
		LIST_10("ACE_morphine"),
		LIST_6("ACE_bloodIV"),
		LIST_4("ACE_splint"),
		LIST_4("ACE_tourniquet"),
		LIST_2("ACE_adenosine")
	};
};

class cls : r
{
	displayName = "Combat Life Saver";
	traits[] = {"medic"};
	backpack[] = 
	{
		"rhsusf_assault_eagleaiii_ocp"
	};
	backpackItems[] +=
	{
		"ACE_surgicalKit",
		LIST_20("ACE_elasticBandage"),
		LIST_15("ACE_fieldDressing"),
		LIST_15("ACE_packingBandage"),
		LIST_15("ACE_epinephrine"),
		LIST_15("ACE_morphine"),
		LIST_12("ACE_bloodIV"),
		LIST_10("ACE_splint"),
		LIST_4("ACE_tourniquet"),
		LIST_2("ACE_adenosine")
	};
};