//Spawn unit and apply loadout and skill
// [<unitData>, <group>, <spawnPos>] call jebus_fnc_spawnUnit;
// Returns the spawned unit

if (!isServer) exitWith {};

params 
[
	"_unitData",
	"_newGroup",
	"_spawnPos"
];

private ["_newUnit"];

_unitData params 
[
	"_unitType",
	"_unitPos",
	"_unitLoadout",
	"_unitFaction",
	//"_unitSkill",
	"_unitVarName"
];

//if (isNull _newGroup) then {diag_log format['Group is null. Count AllGroups = %1', count allGroups]};

_newUnit = _newGroup createUnit [_unitType, _spawnPos, [], 0, "CAN_COLLIDE"];
waitUntil {alive _newUnit};

_newUnit setVariable ["tmf_assignGear_role", _unitLoadout];
[_newUnit, _unitFaction, _unitLoadout] call tmf_assigngear_fnc_assigngear;

//_newUnit setUnitLoadout _unitLoadout;
//_newUnit setSkill _unitSkill;

if (_guard == true) then 
{
	_newUnit disableAI "PATH";
	_newUnit setPosATL _unitPos;
};

if (!(_unitVarName isEqualTo "")) then 
{
	[_newUnit, _unitVarName] remoteExec ["setVehicleVarName", 0, _newUnit];
	missionNamespace setVariable [_unitVarName, _newUnit, true];
};

//[_newUnit] call jebus_fnc_unlimitedAmmo;

_newUnit;