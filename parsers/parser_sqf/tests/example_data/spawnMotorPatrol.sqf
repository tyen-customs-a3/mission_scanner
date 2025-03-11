if (!isServer) exitWith {};
private _trigger = selectRandom (missionNamespace getVariable "RATSNAKE_MotorPatrolSpawns");
private _hmmvw = createVehicle ["I_C_Offroad_02_LMG_F", getPos _trigger];
_hmmvw setDir (triggerArea _trigger) # 2;
private _thisPatrol = createGroup independent;
for "_i" from 1 to 3 do {
	_thisPatrol createUnit ["I_G_Soldier_F", getPos _trigger, [], 10, "NONE"]
};
{_x moveInAny _hmmvw} forEach (units _thisPatrol);
_thisPatrol copyWaypoints (_trigger getVariable "RATSNAKE_waypoints");
_thisPatrol deleteGroupWhenEmpty true;
clearWeaponCargoGlobal _hmmvw;
clearMagazineCargoGlobal _hmmvw;
clearItemCargoGlobal _hmmvw;
clearBackpackCargoGlobal _hmmvw;
["Land_CanisterFuel_Red_F", _hmmvw] call ace_cargo_fnc_loadItem;