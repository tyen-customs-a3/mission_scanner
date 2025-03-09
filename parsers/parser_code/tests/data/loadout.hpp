class baseMan 
{
	displayName = "Unarmed";
	uniform[] = {};
	vest[] = {};
	backpack[] = {};
	headgear[] = {};
	goggles[] = {};
	hmd[] = {};
	faces[] = {};
	insignias[] = {};
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
		LIST_2("usp_g3c_kp_mx_aor2"),
		"usp_g3c_rs_kp_mx_aor2",
		"usp_g3c_rs2_kp_mx_aor2"
	};
	vest[] = 
	{
		"s4_lbt_comms_aor2",
		"s4_lbt_operator_aor2"
	};
	headgear[] = 
	{
		"pca_opscore_cover_ct_aor2_alt",
		"pca_opscore_cover_ct_cb_aor2_alt",
		"pca_opscore_cover_ct_cm_aor2_alt",
		"pca_opscore_cover_ct_cw_aor2_alt"
	};
	backpack[] = 
	{
		"bear_eagleaiii_aor2"
	};
	primaryWeapon[] = 
	{
		"rhs_weap_m4a1_blockII_KAC"
	};
	scope[] = 
	{
		"rhsusf_acc_g33_xps3"
	};
	bipod[] = 
	{
		"rhsusf_acc_rvg_blk"
	};
	items[] =
	{
		"ACRE_PRC343",
		LIST_10("ACE_fieldDressing"),
		LIST_10("ACE_packingBandage"),
		LIST_4("ACE_tourniquet"),
		LIST_2("ACE_epinephrine"),
		LIST_2("ACE_morphine"),
		LIST_2("ACE_splint")
	};
	magazines[] = 
	{
		LIST_2("SmokeShell"),
		LIST_2("rhs_mag_m67"),
		LIST_13("rhs_mag_30Rnd_556x45_M855A1_PMAG")
	};
	backpackItems[] = 
	{
		LIST_4("rhs_mag_30Rnd_556x45_M855A1_PMAG")
	};
};

class ar : rm 
{
	displayName = "Automatic Rifleman";
	primaryWeapon[] = 
	{
		"rhs_weap_m249_light_S"
	};
	bipod[] = 
	{
		"rhsusf_acc_grip4_bipod"
	};
	sidearmWeapon[] = 
	{
		"rhsusf_weap_glock17g4"
	};
	magazines[] = 
	{
		LIST_2("SmokeShell"),
		LIST_2("rhs_mag_m67"),
		LIST_2("rhsusf_mag_17Rnd_9x19_JHP"),
		LIST_3("rhsusf_200Rnd_556x45_mixed_soft_pouch")
	};
	backpackItems[] = 
	{
		LIST_4("rhsusf_200Rnd_556x45_mixed_soft_pouch")
	};
};

class aar : rm 
{
	displayName = "Assistant Automatic Rifleman";
	backpackItems[] += 
	{
		LIST_4("rhsusf_200Rnd_556x45_mixed_soft_pouch")
	};
	linkedItems[] += 
	{
		"ACE_Vector"
	};
};

class rm_lat : rm 
{
	displayName = "Rifleman (LAT)";
	secondaryWeapon[] = 
	{
		"rhs_weap_m72a7"
	};
	backpackItems[] += 
	{
		LIST_2("rhs_weap_m72a7")
	};
};

class gren : rm 
{
	displayName = "Grenadier";
	vest[] = 
	{
		"s4_lbt_weapons_aor2"
	};
	primaryWeapon[] = 
	{
		"bear_weap_m4a1_blockII_m203"
	};
	bipod[] = {};
	backpackItems[] = 
	{
		LIST_21("rhs_mag_M441_HE"),
		LIST_21("rhs_mag_M433_HEDP"),
		LIST_4("1Rnd_Smoke_Grenade_shell")
	};
};

class tl : rm 
{
	displayName = "Team Leader";
	vest[] = 
	{
		"s4_lbt_teamleader_aor2"
	};
	backpackItems[] = 
	{
		LIST_2("rhs_mag_30Rnd_556x45_M855A1_PMAG"),
		LIST_2("rhs_mag_30Rnd_556x45_M855A1_PMAG_Tracer_Red"),
		LIST_2("SmokeShell")
	};
	linkedItems[] += 
	{
		"ACE_Vector"
	};
};

class sl : tl 
{
	displayName = "Squad Leader";
	items[] += 
	{
		"ACRE_PRC148"
	};
	primaryWeapon[] = 
	{
		"rhs_weap_mk18_m320"
	};
	bipod[] = {};
	sidearmWeapon[] = 
	{
		"rhsusf_weap_glock17g4"
	};
	magazines[] += 
	{
		LIST_2("rhsusf_mag_17Rnd_9x19_JHP")
	};
	backpackItems[] =
	{
		LIST_2("rhs_mag_30Rnd_556x45_M855A1_PMAG_Tracer_Red"),
		LIST_21("rhs_mag_M441_HE"),
		LIST_10("rhs_mag_M433_HEDP"),
		LIST_6("1Rnd_Smoke_Grenade_shell"),
		LIST_2("1Rnd_SmokeBlue_Grenade_shell"),
		LIST_2("1Rnd_SmokeGreen_Grenade_shell"),
		LIST_2("1Rnd_SmokeRed_Grenade_shell")
	};
};

class co : sl 
{
	displayName = "Platoon Commander";
	primaryWeapon[] = 
	{
		"rhs_weap_mk18_KAC"
	};
	bipod[] = 
	{
		"rhsusf_acc_rvg_blk"
	};
	backpackItems[] = 
	{
		LIST_2("rhs_mag_30Rnd_556x45_M855A1_PMAG_Tracer_Red"),
		LIST_2("SmokeShellBlue"),
		LIST_2("SmokeShellGreen"),
		LIST_2("SmokeShellRed")
	};
};

class rm_fa : rm 
{
	displayName = "Rifleman (First-Aid)";
	traits[] = {"medic"};
	vest[] = 
	{
		"s4_lbt_medical_aor2"
	};
	backpackItems[] =
	{
		LIST_20("ACE_fieldDressing"),
		LIST_15("ACE_packingBandage"),
		LIST_15("ACE_elasticBandage"),
		LIST_10("ACE_epinephrine"),
		LIST_10("ACE_morphine"),
		LIST_8("ACE_bloodIV"),
		LIST_4("ACE_splint"),
		LIST_4("ACE_tourniquet")
	};
};

class cls : rm_fa
{
	displayName = "Combat Life Saver";
	primaryWeapon[] = 
	{
		"rhs_weap_mk18_KAC"
	};
	sidearmWeapon[] = 
	{
		"rhsusf_weap_glock17g4"
	};
	magazines[] += 
	{
		LIST_2("rhsusf_mag_17Rnd_9x19_JHP")
	};
	backpackItems[] =
	{
		"ACE_surgicalKit",
		LIST_30("ACE_elasticBandage"),
		LIST_30("ACE_packingBandage"),
		LIST_20("ACE_fieldDressing"),
		LIST_20("ACE_epinephrine"),
		LIST_20("ACE_morphine"),
		LIST_12("ACE_bloodIV"),
		LIST_10("ACE_splint"),
		LIST_4("ACE_tourniquet")
	};
};