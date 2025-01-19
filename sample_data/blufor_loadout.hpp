class baseMan 
{
	displayName = "Unarmed";
	// All Randomized
	uniform[] = {};
	vest[] = {};
	backpack[] = {};
	headgear[] = {};
	goggles[] = {};
	hmd[] = {};
	// Leave empty to not change faces and Insignias
	faces[] = {};
	insignias[] = {};
	// All Randomized. Add Primary Weapon and attachments
	// Leave Empty to remove all. {"Default"} for using original items the character start with
	primaryWeapon[] = {};
	scope[] = {};
	bipod[] = {};
	attachment[] = {};
	silencer[] = {};
	// SecondaryAttachments[] arrays are NOT randomized
	secondaryWeapon[] = {};
	secondaryAttachments[] = {};
	sidearmWeapon[] = {};
	sidearmAttachments[] = {};
	// These are added to the uniform or vest first - overflow goes to backpack if there's any
	magazines[] = {};
	items[] = {};
	// These are added directly into their respective slots
	linkedItems[] = 
	{
		"ItemWatch",
		"ItemMap",
		"ItemCompass"
	};
	// These are put directly into the backpack
	backpackItems[] = {};
	// This is executed after the unit init is complete. Argument: _this = _unit
	code = "";
};

class rm : baseMan
{
	displayName = "Rifleman";
	vest[] = 
	{
		"pca_vest_invisible_plate"
	};
	backpack[] = 
	{
		"pca_backpack_invisible_large"
	};
	sidearmWeapon[] = 
	{
		"CUP_hgun_Mac10"
	};
	secondaryWeapon[] = 
	{
		"rhs_weap_rpg7"
	};
	items[] =
	{
		"ACRE_PRC343",
		LIST_20("ACE_fieldDressing"),
		LIST_20("ACE_packingBandage"),
		LIST_2("ACE_epinephrine"),
		LIST_2("ACE_morphine"),
		LIST_2("ACE_bloodIV"),
		LIST_4("ACE_splint"),
		LIST_1("ACE_surgicalKit")
	};
	magazines[] = 
	{
		LIST_10("rhs_rpg7_PG7VL_mag"),
		LIST_10("CUP_30Rnd_45ACP_Green_Tracer_MAC10_M")
	};
};