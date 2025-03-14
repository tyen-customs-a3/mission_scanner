// 0 = [this, <other parameters>] spawn jebus_fnc_main;
// 0 = [this, "LIVES=",1] spawn jebus_fnc_main;

waitUntil {time > 0};
if (!isServer) exitWith {};

private 
[
	"_unit",
	"_cacheRadius",
	"_debug",
	"_exitTrigger",
	"_firstLoop",
	"_initString",
	"_initialDelay",
    "_lives",
	"_newGroup",
	"_pauseRadius",
	"_reduceRadius",
	"_respawnDelay",
	"_respawnMarkers",
	"_respawnPos",
	"_special",
	"_tmpRespawnPos",
	"_tmpZone",
	"_trigger",
	"_unitSide",
	"_synchronizedObjectsList"
];

_unit = _this select 0;

// Make sure unit is a unit and not a group (Thanks to S.Crowe)
if (typeName _unit == "GROUP") then {_unit = leader _unit;};

_respawnPos = getPos _unit;
_unitSide = side _unit;

_synchronizedObjectsList = [];
_respawnPosList = [];
_respawnPosList pushBack _respawnPos;
_respawnMarkerList = [];

//Set up default parameters
_lives = -1;
_cacheRadius = 0;
_reduceRadius = 0;
_pauseRadius = 0;
_respawnDelay = 30;
_initialDelay = 0;
_guard = false;
_special = "NONE";
_respawnMarkers = [];
_initString = "";
_debug = false;

//Get parameters
for "_parameterIndex" from 1 to (count _this - 1) do 
{
	switch (_this select _parameterIndex) do 
	{
		case "LIVES=" : {_lives = _this select (_parameterIndex + 1)};
		case "DELAY=" : {_respawnDelay =  _this select (_parameterIndex + 1)};
		case "START=" : {_initialDelay = _this select (_parameterIndex + 1)};
		case "CACHE=" : {_cacheRadius =  _this select (_parameterIndex + 1)};
		case "REDUCE=" : {_reduceRadius =  _this select (_parameterIndex + 1)};
		case "GUARD" : {_guard = true};
		case "FLYING" : {_special = "FLY"};
		case "RESPAWNMARKERS=" : {_respawnMarkers = _this select (_parameterIndex + 1)};
		case "PAUSE=" : {_pauseRadius = _this select (_parameterIndex + 1)};
		case "EXIT=" : {_exitTrigger = _this select (_parameterIndex + 1)};
		case "INIT=" : {_initString = _this select (_parameterIndex + 1)};
		case "DEBUG" : {_debug = true};
	};
};

//Add additional Respawn positions where applicable
{
	_respawnPosList pushBack (getMarkerPos _x);
	_respawnMarkerList pushBack (getMarkerPos _x);
} forEach _respawnMarkers;

//Determine number of lives if passed an array
if (typeName _lives == "ARRAY") then 
{
	_minLives = _lives select 0;
	_maxLives = _lives select 1;
	_lives = _minLives + floor random (_maxLives - _minLives);
};

_syncs = synchronizedObjects _unit;

{
	if (_x isKindOf "EmptyDetector") then
	{
		_trigger = _x;
		if (_debug) then {systemChat "Synchronized trigger activation present"};
	}
	else
	{
		_synchronizedObjectsList append [_x];
	};
} forEach _syncs;

//Save unit data and delete
_groupData =  [_unit] call jebus_fnc_saveGroup;
[group _unit] call jebus_fnc_deleteGroup;

_firstLoop = true;

//Main loop
while { _lives != 0 } do 
{
	sleep 5;
	
	//Wait for trigger activation (Thanks to pritchardgsd)
	if (! isNil "_trigger") then 
	{
		waituntil 
		{
			if (_debug && (floor ((time % 10)) == 0)) then {systemChat "Waiting for trigger activation"};
			sleep 5;
			(triggerActivated _trigger);
		};
	};
	
	if (_firstLoop && _initialDelay > 0) then 
	{
		sleep _initialDelay;
		_firstLoop = false;
		if (_debug) then {systemChat "First Loop!"};
	};

	if (count _respawnMarkers > 0) then 
	{
		_tmpRespawnPos = selectRandom _respawnMarkerList;
	}
	else
	{
		_tmpRespawnPos = selectRandom _respawnPosList;
	};
	
	while {[_tmpRespawnPos, _unitSide, _pauseRadius] call jebus_fnc_enemyInRadius} do 
	{
		if (_debug) then {systemChat format["%1 - Enemies in pause radius", _displayName]};
		sleep 10;
	};
	
	//Spawn group
	_newGroup = [_groupData, _tmpRespawnPos, _special] call jebus_fnc_spawnGroup;
	_displayName = str _newGroup;
	
	if (_debug) then {systemChat format["Spawning group: %1", _displayName]};
	
	_newGroup deleteGroupWhenEmpty true;
	
	//Initiate caching
	if ("CACHE=" in _this) then 
	{
		[_newGroup, _cacheRadius, _debug] spawn jebus_fnc_cache;
	};
	
	//Initiate reducing
	if ("REDUCE=" in _this) then 
	{
		[_newGroup, _reduceRadius, _debug] spawn jebus_fnc_reduce;
	};
	
	_newGroup allowfleeing 0;
	
	//Add synchonizations and execute init string
	_proxyThis = leader _newgroup;
	
	{
		_proxyThis synchronizeObjectsAdd [_x];
	} forEach _synchronizedObjectsList;
	
	call compile format [_initString];
	
	sleep 1;
	
	//Check every 10 seconds to see if group is eliminated
	/*
	waituntil 
	{
		sleep 10;
		{alive _x} count (units _newGroup) < 1;
	};
	
	if (_debug) then {systemChat format ["%1 eliminated. Waiting for respawn.", _displayName]};
	*/
	
	//Respawn delay
	if (typeName _respawnDelay == "ARRAY") then 
	{
		_minTime = _respawnDelay select 0;
		_maxTime = _respawnDelay select 1;
		_tempRespawnDelay = _minTime + random (_maxTime - _minTime);
		if (_debug) then {systemChat format ["Respawn delay = %1 seconds", _tempRespawnDelay]};
		sleep _tempRespawnDelay;
	}
	else
	{
		if (_debug) then {systemChat format ["Respawn delay = %1 seconds", _respawnDelay]};
		sleep _respawnDelay;
	};
	
	//Check if exit trigger has been activated
	if (! isNil "_exitTrigger") then 
	{
		if (triggerActivated _exitTrigger) then 
		{
			if (_debug) then {systemChat "Exit trigger activated"};
			_lives = 1;
		};
	};
	
	_lives = _lives - 1;
	
	//Clean up empty group
	deleteGroup _newGroup;
};

if (_debug) then {systemChat "Exiting script."};