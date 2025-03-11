#[derive(Debug, Clone)]
pub struct ItemReference {
    pub item_id: String,
    pub kind: ItemKind,
}

#[derive(Debug, Clone)]
pub enum ItemKind {
    Backpack,
    Weapon,
    WeaponMagazine,
    Goggles,
    Headgear,
    Item,
} 