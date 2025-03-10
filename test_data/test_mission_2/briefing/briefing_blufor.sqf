/* ===============================================
	GENERAL BRIEFING NOTES
	 - Uses HTML style syntax. All supported tags can be found here - https://community.bistudio.com/wiki/createDiaryRecord
	 - For images use <img image='FILE'></img> (for those familiar with HTML note it is image rather than src).
	 - Note that using the " character inside the briefing block is forbidden use ' instead of ".
*/

/* ===============================================
	SITUATION
	 - Outline of what is going on, where we are we and what has happened before the mission has started? This needs to contain any relevant background information.
	 - Draw attention to friendly and enemy forces in the area. The commander will make important decisions based off this information.
	 - Outline present weather conditions, players will typically assume that it is daylight with sunny weather.
*/

private _situation = ["diary", ["Situation","
<br/>
<font size='20'>SITUATION</font>
<br/>
Thuh fuh-armers ovher at Petrovka 'ave bee-yn stealin' our 'ay bales for a while now. Jus' last weekey-nd thay done it again, this is thuh final strayw! We gathered all thuh fuh-armers awf Grishino eend after searchin' for a needle ina 'aystack we 'ave found where thay 'ave bee-yn hidin' our 'ay. Do not git yo-wr pitchforks, wer goin' strayaahyt for thuh shoat'uns!
<br/><br/>
<font size='20'>ENEMY FORCES</font>
<br/>
All awf thuh fuh-armers from Petrovka, thuh ay werrn' dem yellow.
<br/><br/>
<font size='20'>FRIENDLY FORCES</font>
<br/>
All awf thuh fuh-armers from Grishino.
<br/>
"]];

/* ===============================================
	MISSION
	 - Describe any objectives that the team is expected to complete.
	 - Summarize(!) the overall task. This MUST be short and clear.
*/

private _mission = ["diary", ["Mission","
<br/>
<font size='20'>PRIMARY OBJECTIVES</font>
<br/>
Git shed awf all thuh Petrovka fuh-armers an secyhaw thuh barn where thay 'ave bee-yn hidin' our 'ay so we kay-yun move in our trayctors in thuh afternoon.
<br/><br/>
(Kill Yellow Farmers, secure barn)
<br/><br/>
"]];

/* ===============================================
	ADMINISTRATION
	 - Outline of logistics: available resources (equipment/vehicles) and ideally a summary of their capabilities.
	 - Outline of how to use any mission specific features/scripts.
	 - Seating capacities of each vehicle available for use.
*/

private _administration = ["diary", ["Administration","
<br/>
Wer all speakin' thuh same languayge.
<br/><br/>
"]];

player createDiaryRecord _administration;
player createDiaryRecord _execution;
player createDiaryRecord _mission;
player createDiaryRecord _situation;