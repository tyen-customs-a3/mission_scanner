
private _briefing = "ADMIN BRIEFING<br/><br/>";


/* In this briefing page you should provide the admin with any information that will aid them doing their job.
	- If mission has no automatic ending system. All conditions for the mission ending should be mentioned here so the session host knows what to do.


*/

// Insert custom text
_briefing = _briefing + 
"

additional enemy reinforcents can be sent after initial mechanized spawn at admin discretion.
Mission ends after squad retreats to the south.
Mission victory if all 3 ammo caches and truck are destroyed or stolen.
";

player createDiaryRecord ["diary", ["Admin",_briefing]];