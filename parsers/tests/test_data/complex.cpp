////////////////////////////////////////////////////////////////////
//DeRap: config.bin
//Produced from mikero's Dos Tools Dll version 9.98
//https://mikero.bytex.digital/Downloads
//'now' is Fri Feb 28 16:15:23 2025 : 'file' last modified on Wed Sep 04 21:24:41 2024
////////////////////////////////////////////////////////////////////

#define _ARMA_

enum {
    _fnc_sizeex = 0,
    _fnc_colorhextorgba_4 = 0,
    _axiscolor1 = 0,
    destructengine = 2,
    _axiscolor2 = 0,
    destructdefault = 6,
    destructwreck = 7,
	0 = 0,
    _fnc_colorrgbatohex = 0,
    _fnc_colorhextorgba_6 = 0,
    _axiscolor3 = 0,
    destructtree = 3,
    destructtent = 4,
    stabilizedinaxisx = 1,
    stabilizedinaxesxyz = 4,
    stabilizedinaxisy = 2,
	3 = 3,
    _fnc_colorhextorgba = 0,
    stabilizedinaxesboth = 3,
    destructno = 0,
    stabilizedinaxesnone = 0,
    _fnc_colorhextorgba_0 = 0,
    destructman = 5,
    _fnc_colorhextorgba_2 = 0,
    destructbuilding = 1
};

class CfgPatches
{
    class A3_3DEN
    {
        author = "$STR_A3_Bohemia_Interactive";
        name = "Arma 3 Eden Update - Eden Editor";
        url = "https://www.arma3.com";
        requiredAddons[] = {"A3_Data_F_Exp_B"};
        requiredVersion = 0.1;
        units[] = {"Sphere_3DEN","SphereNoGround_3DEN"};
        weapons[] = {};
    };
};
class CfgAddons
{
    class PreloadAddons
    {
        class 3DEN
        {
            list[] = {"A3_3DEN","A3_3DEN_Language","3DEN"};
        };
    };
};
class RscText;
class RscTitle;
class RscListbox;
class RscControlsGroupNoScrollbars;
class ctrlDefault
{
    access = 0;
    x = 0;
    tooltip = "";
    tooltipMaxWidth = 0.5;
    tooltipColorShade[] = {0,0,0,1};
    tooltipColorText[] = {1,1,1,1};
    class ScrollBar
    {
        width = 0;
        border = "\a3\3DEN\Data\Controls\ctrlDefault\border_ca.paa";
    };
};
class RscDisplayDebriefingTacops
{
	idd = -1;
	scriptName = "RscDisplayDebriefingTacops";
	scriptPath = "TacopsDisplays";
	onLoad = "[""onLoad"",_this,""RscDisplayDebriefingTacops"",'TacopsDisplays'] call 	(uinamespace getvariable 'BIS_fnc_initDisplay')";
	onUnload = "[""onUnload"",_this,""RscDisplayDebriefingTacops"",'TacopsDisplays'] call 	(uinamespace getvariable 'BIS_fnc_initDisplay')";
	class Controls
	{
		class Title: RscTitle
		{
			colorBackground[] = {"(profilenamespace getvariable ['GUI_BCG_RGB_R',0.13])","(profilenamespace getvariable ['GUI_BCG_RGB_G',0.54])","(profilenamespace getvariable ['GUI_BCG_RGB_B',0.21])","(profilenamespace getvariable ['GUI_BCG_RGB_A',0.8])"};
			idc = 34604;
			text = "$STR_DISP_DEBRIEFING";
			x = "1 * 					(			((safezoneW / safezoneH) min 1.2) / 40) + 		(safezoneX + (safezoneW - 					((safezoneW / safezoneH) min 1.2))/2)";
			y = "0.9 * 					(			(			((safezoneW / safezoneH) min 1.2) / 1.2) / 25) + 		(safezoneY + (safezoneH - 					(			((safezoneW / safezoneH) min 1.2) / 1.2))/2)";
			w = "38 * 					(			((safezoneW / safezoneH) min 1.2) / 40)";
			h = "1 * 					(			(			((safezoneW / safezoneH) min 1.2) / 1.2) / 25)";
		};
	};
};
class CivilianPresence_OnCreated
{
    property = "#onCreated";
    validate = "string";
    control = "EditCodeMulti5";
    displayName = "$STR_a3_to_basicCivilianPresence21";
    expression = "_this setVariable [""#onCreated"",compile _value]";
    tooltip = "$STR_a3_to_basicCivilianPresence22";
};
class RscTitles
{
	class RscAnimatedScreen
	{
		idd = -1;
		duration = 1e+11;
		fadeIn = 0;
		fadeOut = 0;
		onLoad = "uiNamespace setVariable ['bis_animatedScreen_displayMain',_this select 0];";
		class Controls{};
	};
	class RscAnimatedScreenOverlay
	{
		idd = -1;
		duration = 1e+11;
		fadeIn = 0;
		fadeOut = 0;
		onLoad = "uiNamespace setVariable ['bis_animatedScreen_displayOverlay',_this select 0];";
		class Controls{};
	};
};
class CfgRespawnTemplates
{
	delete Revive;
};
class CfgTimeTrials
{
	pointTimeMultiplier = 0.1;
	penaltyMissed = 100;
	iconsMedals[] = {"A3\modules_f_beta\data\FiringDrills\medal_bronze_ca","A3\modules_f_beta\data\FiringDrills\medal_silver_ca","A3\modules_f_beta\data\FiringDrills\medal_gold_ca"};
	colorsMedals[] = {"#A0522D","#C0C0C0","#FFD700"};
	music[] = {"BackgroundTrack01_F","BackgroundTrack01_F_EPB","BackgroundTrack01_F_EPC","BackgroundTrack02_F_EPC","BackgroundTrack03_F","BackgroundTrack04_F_EPC"};
	class Helpers
	{
		class Sign_Circle_F
		{
			triggerRadius = 14;
			autoOrient = 1;
			3DIcon = "badge_simple";
		};
	};
};
