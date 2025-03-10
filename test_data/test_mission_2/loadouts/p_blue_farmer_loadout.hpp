class baseMan 
{
	displayName = "Unarmed";
	uniform[] = {};
	vest[] = {};
	backpack[] = {};
	headgear[] = {};
	goggles[] = {};
	hmd[] = {};
	primaryWeapon[] = {};
	scope[] = {};
	bipod[] = {};
	attachment[] = {};
	silencer[] = {};
	secondaryWeapon[] = {};
	secondaryAttachments[] = {};
	sidearmWeapon[] = {};
	sidearmAttachments[] = {};
	magazines[] = {};
	items[] = {};
	linkedItems[] = 
	{
		"ItemWatch",
		"ItemMap",
		"ItemCompass"
	};
	backpackItems[] = {};
	code = "";
};

class rm : baseMan
{
	displayName = "Rifleman";
	uniform[] =
	{
		"CUP_U_C_Villager_01"
	};
	vest[] =
	{
		"rhs_6sh46"
	};
	headgear[] =
	{
		"CUP_H_C_Beret_01",
		"CUP_H_C_Beret_02",
		"CUP_H_C_Beret_03",
		"CUP_H_C_Beret_04"
	};
	primaryWeapon[] =
	{
		"rhs_weap_Izh18"
	};
	items[] =
	{
		"ACRE_PRC343",
		LIST_10("ACE_fieldDressing"),
		LIST_5("ACE_packingBandage"),
		LIST_5("ACE_quikclot"),
		LIST_4("ACE_tourniquet"),
		LIST_2("ACE_epinephrine"),
		LIST_2("ACE_morphine"),
		LIST_2("ACE_splint")
	};	
	magazines[] =
	{
		LIST_2("SmokeShell"),
		LIST_21("rhsgref_1Rnd_Slug"),
		LIST_20("rhsgref_1Rnd_Slug")
	};
};

class tl : rm
{
	displayName = "Team Leader";
	items[] +=
	{
		"ACRE_PRC148"
	};
	magazines[] +=
	{
		LIST_2("SmokeShell")
	};
	linkedItems[] +=
	{
		"rhssaf_zrak_rd7j"
	};
};

class cls : rm
{
	displayName = "Combat Life Saver";
	traits[] = {"medic"};
	backpack[] =
	{
		"rhs_sidor"
	};
	backpackItems[] =
	{
		"ACE_surgicalKit",
		LIST_10("ACE_elasticBandage"),
		LIST_10("ACE_fieldDressing"),
		LIST_10("ACE_packingBandage"),
		LIST_10("ACE_epinephrine"),
		LIST_10("ACE_morphine"),
		LIST_6("ACE_bloodIV"),
		LIST_4("ACE_splint"),
		LIST_4("ACE_tourniquet"),
		LIST_2("ACE_adenosine")
	};
};