TYPE Equipment
  WName AS STRING * 15
  Attack AS INTEGER
  Defense AS INTEGER
' ^^ DUH!!
  Weight AS INTEGER
' ^ How much it decreases speed
  Enchanted AS INTEGER
  EquipWhere AS INTEGER
  EquipWho AS INTEGER
' ^^ Which players can equip it, and where
END TYPE

TYPE Item
  ItemName AS STRING * 10
  Desc AS STRING * 25
  EffectStat AS INTEGER
' ^ Which Player Asset it affects (HP, MP, Speed, Weight, Strength, Etc.)
  AmountOfEffect AS INTEGER
' ^ How much it changes that number
  EquipWhere AS INTEGER
  EquipWho AS INTEGER
' ^^ Which players can equip it, if any, and where
  Enchanted AS INTEGER
  Usable AS INTEGER
' ^ For potions, Etc.
END TYPE

TYPE Spell
  SpellName AS STRING * 10
  SpellDesc AS STRING * 30
  Effect AS INTEGER
  EStrength AS INTEGER
  MPCost AS INTEGER
' HPCost AS INTEGER 'Possibly for sacrifice-Spells,
'                    Take 100 Damage & kill Monster of choice?
END TYPE

TYPE LocationType
  WorldX AS INTEGER
  WorldY AS INTEGER
  ScreenX AS INTEGER
  ScreenY AS INTEGER
  IconOffsetX AS INTEGER
  IconOffsetY AS INTEGER
' ^^ When Player is beetween Icons, display purposes only.
  NFlag AS STRING * 2
  Facing AS INTEGER
  
END TYPE

TYPE Player
  LeftArm AS INTEGER
  RightArm AS INTEGER
  Armor AS INTEGER
  'Neck AS INTEGER
  'Wrists AS INTEGER
  'Boots AS INTEGER
  'Pants AS INTEGER
  'Belt AS INTEGER
' ^^ Weapons/Armors/Items are held in an arrays. Those are array subscripts.
  HP AS INTEGER
  HPMax AS INTEGER
  MP AS INTEGER
  MPMax AS INTEGER
  Status AS INTEGER
  Speed AS INTEGER
  'TotalWeight AS INTEGER
  Strength AS INTEGER
  BaseDefense AS INTEGER
  MAttack AS INTEGER
  MDefense AS INTEGER
  PhysicalLearnRate AS INTEGER
  MagicLearnRate AS INTEGER
  PLimit AS INTEGER
  MLimit AS INTEGER
' ^^^^ For LevelUps\/ ^^ A Wizard won't ever be strong,
'                  and a Warrior can only learn so much magic
  Location AS LocationType
  'MagicLearned(NumOfSpells) AS INTEGER, allocated separately
  XP AS LONG
  Gold

END TYPE

TYPE Monster
  HP AS INTEGER
  MP AS INTEGER
  HPMax AS INTEGER
  MPMax AS INTEGER
  Attack AS INTEGER
  Defense AS INTEGER
  MAttack AS INTEGER
  MDefense AS INTEGER
  Speed AS INTEGER
  Item1 AS INTEGER
  Item2 AS INTEGER
' ^^ Indices of items more & less likely to leave when killed
  ItemChanceRating AS INTEGER
  Ststus AS INTEGER
  Special1 AS Spell
  SChance1 AS INTEGER
  Special2 AS Spell
  SChance2 AS INTEGER
END TYPE
