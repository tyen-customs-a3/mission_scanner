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

// First add items with explicit types
_unit addWeapon "rhs_weap_hk416d145";
_unit addWeapon "rhs_weap_m16a4_imod";
_unit addWeapon "rhs_weap_m4a1_m320";
_unit addWeapon "rhs_weap_M136";

_unit addMagazine "rhs_mag_30Rnd_556x45_M855A1_Stanag";
_unit addMagazine "rhsusf_200Rnd_556x45_M855_mixed_soft_pouch";

_unit addUniform "Tarkov_Uniforms_1";
_unit addVest "V_PlateCarrier2_blk";
_unit addBackpack "rhsusf_spcs_ocp_saw";

//Define Arsenal items
private _itemEquipment = 
[
	//Uniforms
		"Tarkov_Uniforms_1",
		
	
	//Vests
		"V_PlateCarrier2_blk"
];


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