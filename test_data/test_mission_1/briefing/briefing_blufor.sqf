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

<font size='18'>ENEMY FORCES</font>
<br/>
Platoon strength infantry guarding the town. motorized, mechanized and heliborne infantry within the area will likely respond.
<br/><br/>

<font size='18'>FRIENDLY FORCES</font>
<br/>
an upsized squad of enthusiastic and eclectically armed guerrillas.

"]];

/* ===============================================
    MISSION
     - Describe any objectives that the team is expected to complete.
     - Summarize(!) the overall task. This MUST be short and clear.
*/

private _mission = ["diary", ["Mission","
<br/>
Destroy or steal all ammo caches in the town of abdera to the south.
<br/><br/>
Retreat to the north after mission completion
"]];

/* ===============================================
    EXECUTION
     - Provide an outline as to what the commander of the player's command might give.
*/

private _execution = ["diary", ["Execution","
<br/>
<font size='18'>COMMANDER'S INTENT</font>
<br/>
Raid the outpost set up within the town of abdera and destroy or steal all weapons and ammo caches found. retreat to the north after mission is acomplished, move quickly after contact is made to avoid getting caught by enemy reinforcements.
<br/><br/>

<font size='18'>MOVEMENT PLAN</font>
<br/>
Advance on foot at squad leads discretion.
<br/><br/>


<font size='18'>SPECIAL TASKS</font>
<br/>
due to state of our own weaponry, use of enemy weaponry from caches and soldiers may be recomended.
"]];

/* ===============================================
    ADMINISTRATION
     - Outline of logistics: available resources (equipment/vehicles) and ideally a summary of their capabilities.
     - Outline of how to use any mission specific features/scripts.
     - Seating capacities of each vehicle available for use.
*/

private _administration = ["diary", ["Administration","
<br/>
no logistics support, use what you can find.
"]];

player createDiaryRecord _administration;
player createDiaryRecord _execution;
player createDiaryRecord _mission;
player createDiaryRecord _situation;