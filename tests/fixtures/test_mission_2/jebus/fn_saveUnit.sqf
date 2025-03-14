//Save unit data
// [<unit>] call jebus_fnc_saveUnit;
// Returns an array
// [<unitType>, <unitLoadout>, <unitSkill>]

if (!isServer) exitWith {};

params ["_unit"];

private ["_unitData"];
//Save TMF role data
private _tmfRole = _unit getVariable ["tmf_assignGear_role", nil];
private _tmfFaction = _unit getVariable ["tmf_assigngear_faction", nil];

_unitData = [];

_unitData pushBack (typeOf _unit);
_unitData pushBack (getPosATL _unit);
_unitData pushBack (_tmfRole);
_unitData pushBack (_tmfFaction);
_unitData pushBack (vehicleVarName _unit);

_unitData;