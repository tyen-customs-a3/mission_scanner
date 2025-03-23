```
class Attributes
{
    class Inventory
    {
        class primaryWeapon
        {
            name = "rhs_weap_ak74n";
            muzzle = "rhs_acc_dtk1983";
            firemode = "rhs_weap_ak74n:Single";
            class primaryMuzzleMag
            {
                name = "rhs_30Rnd_545x39_7N10_AK";
                ammoLeft = 30;
            };
        };
        class secondaryWeapon
        {
            name = "rhs_weap_fim92";
            class primaryMuzzleMag
            {
                name = "rhs_fim92_mag";
                ammoLeft = 1;
            };
        };
        class uniform
        {
            typeName = "wsx_uniform_gorka_emr";
            isBackpack = 0;
            class ItemCargo
            {
                items = 3;
                class Item0
                {
                    name = "ACE_fieldDressing";
                    count = 2;
                };
                class Item1
                {
                    name = "ACE_packingBandage";
                    count = 1;
                };
                class Item2
                {
                    name = "ACE_quikclot";
                    count = 1;
                };
            };
        };
        class vest
        {
            typeName = "rhs_6b23_digi_6sh92";
            isBackpack = 0;
            class MagazineCargo
            {
                items = 3;
                class Item0
                {
                    name = "rhs_mag_rgd5";
                    count = 1;
                    ammoLeft = 1;
                };
                class Item1
                {
                    name = "rhs_mag_rdg2_white";
                    count = 1;
                    ammoLeft = 1;
                };
                class Item2
                {
                    name = "rhs_30Rnd_545x39_7N10_AK";
                    count = 6;
                    ammoLeft = 30;
                };
            };
        };
        class backpack
        {
            typeName = "wsx_umbts_emr";
            isBackpack = 1;
        };
        map = "ItemMap";
        compass = "ItemCompass";
        watch = "ItemWatch";
        headgear = "rhs_6b26_digi";
    };
```

These are the class structures we want to search for and the data we want to extract

FIND Inventory
EXTRACT map, compass, watch, headgear, nvg

FIND Inventory/primaryWeapon
EXTRACT name, muzzle

FIND Inventory/secondaryWeapon
EXTRACT name, muzzle

FIND Inventory/primaryMuzzleMag
EXTRACT name

FIND Inventory/uniform
EXTRACT typename

FIND Inventory/uniform/ItemCargo
ITERATE over the child classes based on the `items` count
FIND Item* class names
EXTRACT name

FIND Inventory/vest
EXTRACT typename

FIND Inventory/vest/MagazineCargo
ITERATE over the child classes based on the `items` count
FIND Item* class names
EXTRACT name

FIND Inventory/backpack
EXTRACT typeName

