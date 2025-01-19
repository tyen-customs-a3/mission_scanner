/**
	* Adds curated arsenal to player that disables itself under specified conditions.
	*
	* Faction:
	*
	* Usage - under initPlayerLocal.sqf
	* 0 = execVM 'loadouts\arsenal.sqf';
*/

//Variables


arsenal = "building" createVehicleLocal [0,0,0];
player setVariable ["startpos", getPosASL player];


//Define Arsenal items
private _itemEquipment = 
[
	//Uniforms
		"Tarkov_Uniforms_1",
		"Tarkov_Uniforms_2",
		"Tarkov_Uniforms_3",
		"Tarkov_Uniforms_4",
		"Tarkov_Uniforms_5",
		"Tarkov_Uniforms_6",
		"Tarkov_Uniforms_7",
		"Tarkov_Uniforms_8",
		"Tarkov_Uniforms_9",
		"Tarkov_Uniforms_10",
		"Tarkov_Uniforms_11",
		"Tarkov_Uniforms_12",
		"Tarkov_Uniforms_13",
		"Tarkov_Uniforms_358",
		"Tarkov_Uniforms_359",
		"Tarkov_Uniforms_348",
	
	//Vests
		"V_PlateCarrier2_blk",
		"V_PlateCarrier1_blk",
	//Helmets
		"H_HelmetSpecB_blk",
		"H_HelmetSpecB_snakeskin",
		"H_HelmetSpecB",
	//Backpacks
		"rhs_tortila_black",
	
	//Cosmetics

	
	//HMD
	
	"rhsusf_ANPVS_15",
	//MISC
	//ACRE
	"ACRE_PRC343",
	
	//ACE
	"ACE_Flashlight_XL50",
	"ACE_MapTools",
	"ACE_RangeCard",
	"greenmag_item_speedloader",
	
	//BIS
	"ItemCompass",
	"ItemMap",
	"ItemWatch",
	"Laserbatteries",
	"ToolKit",
	"FirstAidKit",
	"diw_armor_plates_main_plate",



	//Binoculars
	//ACE
	
	//BIS
	"Binocular",
	"Rangefinder",
	
	//RHS


	
	
	//Explosives
	//ACE
	"ACE_40mm_Flare_green",
	"ACE_40mm_Flare_red",
	"ACE_40mm_Flare_white",
	"ACE_40mm_Flare_ir",
	
	//BIS
	"UGL_FlareGreen_F",
	"UGL_FlareRed_F",
	"UGL_FlareWhite_F",
	"UGL_FlareYellow_F",
	"1Rnd_SmokeBlue_Grenade_shell",
	"1Rnd_SmokeGreen_Grenade_shell",
	"1Rnd_SmokeOrange_Grenade_shell",
	"1Rnd_SmokePurple_Grenade_shell",
	"1Rnd_SmokeRed_Grenade_shell",
	"1Rnd_SmokeYellow_Grenade_shell",
	"SmokeShellBlue",
	"SmokeShellGreen",
	"SmokeShellOrange",
	"SmokeShellPurple",
	"SmokeShellRed",
	"SmokeShellYellow",
	
	//RHS
	"rhs_mag_M585_white",
	"rhs_mag_m661_green",
	"rhs_mag_m662_red",
	"rhs_mag_m713_Red",
	"rhs_mag_m714_White",
	"rhs_mag_m715_Green",
	"rhs_mag_m716_yellow",
	
	//CUP
	"CUP_1Rnd_StarCluster_Green_M203",
	"CUP_1Rnd_StarCluster_Red_M203",
	"CUP_1Rnd_StarCluster_White_M203",
	"CUP_1Rnd_StarFlare_Green_M203",
	"CUP_1Rnd_StarFlare_Red_M203",
	"CUP_1Rnd_StarFlare_White_M203",
	
	
	//Radios
	"ACRE_PRC148",
	"ACRE_PRC152",
	"ACRE_PRC117F"



private _itemMod =
[	
	//Optics
	"rhsusf_acc_eotech_552",
	"rhsusf_acc_compm4",
	//Muzzle Devices
	
	
	//Bipod & Foregrips
		"rhsusf_acc_grip1",
		"rhsusf_acc_grip2",
		"rhsusf_acc_grip3",
		"rhsusf_acc_grip4",
		"rhsusf_acc_grip4_bipod",
		"rhsusf_acc_saw_lw_bipod"
	
	//Tactical Devices
];

private _itemWeaponRifle =
[
		"rhs_weap_hk416d145",
		"rhs_weap_m16a4_imod",
		"rhsusf_spcs_ocp_saw",
		"rhs_weap_m4a1_m320"

];

private _itemWeaponLAT = 
[
	"rhs_weap_M136"
];


private _itemWeaponAmmo =
[
	//Rifle Ammo
	"rhs_mag_30Rnd_556x45_M855A1_Stanag",
	"greenmag_ammo_556x45_M855A1_60Rnd",
	"rhsusf_200Rnd_556x45_M855_mixed_soft_pouch",
	
	//Explosives
	
	//ACE
	"ACE_HandFlare_Green",
	"ACE_HandFlare_Red",
	"ACE_HandFlare_White",
	"ACE_HandFlare_Yellow",
	
	//BIS
	"1Rnd_HE_Grenade_shell",
	"1Rnd_Smoke_Grenade_shell",
	"HandGrenade",
	"SmokeShell"
	
	//RHS

];



//Add Existing Player Items
{
    _itemEquipment pushBackUnique _x;
}forEach (primaryWeaponItems player);

{
    _itemEquipment pushBackUnique _x;
}forEach (handgunItems player);

_itemEquipment pushBack uniform player;
_itemEquipment pushBack vest player;
_itemEquipment pushBack backpack player;
_itemEquipment pushBack headgear player;

{
    _itemEquipment pushBackUnique _x;
} forEach (assignedItems player);

//Match unitrole name with the classnames in loadout.
	[arsenal, (_itemEquipment + _itemMod + _itemLAT + _itemWeaponRifle + _itemWeaponAmmo)] call ace_arsenal_fnc_initBox;


_action = 
[
	"personal_arsenal","Personal Arsenal","\A3\ui_f\data\igui\cfg\weaponicons\MG_ca.paa",
	{
		[arsenal, _player] call ace_arsenal_fnc_openBox
	},
	{ 
		(player distance2d (player getVariable ["startpos",[0,0,0]])) < 200
	},
	{},
	[],
	[0,0,0],
	3
] call ace_interact_menu_fnc_createAction;

["CAManBase", 1, ["ACE_SelfActions"], _action, true] call ace_interact_menu_fnc_addActionToClass;