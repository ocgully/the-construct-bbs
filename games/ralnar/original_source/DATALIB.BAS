TYPE ItemType
  Num AS INTEGER
  IName AS STRING * 10
  Desc AS STRING * 40
  ItemType AS INTEGER '(1= Item, 2 = Armor, 3= Weapon, 4 = Specials)
  EquipWho AS STRING * 4 'Player Letter (A-Z)
  HPEffect AS INTEGER
  MPEffect AS INTEGER
  SpeedEffect AS INTEGER
  StrengthEffect AS INTEGER
  DefenseEffect AS INTEGER
  StatusEffect AS STRING * 4
  Cost AS LONG
END TYPE

DEFSNG A-Z
SUB Items (Num AS INTEGER, Item AS ItemType)
'DIM Item AS ItemType
'  Num AS INTEGER
'  IName AS STRING * 10
'  Desc AS STRING * 40
'  ItemType AS INTEGER '(1= Item, 2 = Armor, 3= Weapon, 4 = Specials)
'  EquipWho AS STRING * 4 'Player Letter (A-Z)
'  HPEffect AS INTEGER
'  MPEffect AS INTEGER
'  SpeedEffect AS INTEGER
'  StrengthEffect AS INTEGER
'  DefenseEffect AS INTEGER
'  StatusEffect AS STRING * 6 (Z = All Members [Must Be First])
' (P = Poison, S = Stone, D = Dead, U = Unconcious, C = Confused, A = asleep)
' (N = Not For Battle Use Only) 'Both Battle and Inv
'  Cost

SELECT CASE Num
    CASE 0
      Item.Num = 0
      Item.IName = ""
      'Max Length "----------------------------------------"
      Item.Desc = ""
      Item.ItemType = 0 'Nothing
      Item.EquipWho = ""
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = ""
      Item.Cost = 0

'********** ITEMS ************ 1-99

    CASE 1
      Item.Num = 1
      Item.IName = "Cure"
      'Max Length "----------------------------------------"
      Item.Desc = "This potion heals a players HP 50 points"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 50
      Item.MPEffect = 0
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "N"
      Item.Cost = 10
 

    CASE 2
      Item.Num = 2
      Item.IName = "Cure 2"
      'Max Length "----------------------------------------"
      Item.Desc = "Cure's one player for 200 HP"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 200
      Item.MPEffect = 0
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "N"
      Item.Cost = 80

    CASE 3
      Item.Num = 3
      Item.IName = "Cure 3"
      'Max Length "----------------------------------------"
      Item.Desc = "Cure's one player for 1000 HP"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 1000
      Item.MPEffect = 0
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "N"
      Item.Cost = 200
   
    CASE 4
      Item.Num = 4
      Item.IName = "Cure 4"
      'Max Length "----------------------------------------"
      Item.Desc = "Cure's one player to MAX HP"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 9999
      Item.MPEffect = 0
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "N"
      Item.Cost = 500

    CASE 5
      Item.Num = 5
      Item.IName = "Cure All 1"
      'Max Length "----------------------------------------"
      Item.Desc = "Cure's all player's for 200 HP"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 50
      Item.MPEffect = 0
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "ZN"
      Item.Cost = 200

    CASE 6
      Item.Num = 6
      Item.IName = "Cure All 2"
      'Max Length "----------------------------------------"
      Item.Desc = "Cure's all player's to MAX HP"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 9999
      Item.MPEffect = 0
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "ZN"
      Item.Cost = 2000


    CASE 7
      Item.Num = 7
      Item.IName = "Multi Heal"
      'Max Length "----------------------------------------"
      Item.Desc = "This heals a players status in battle"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "CSPAU"
      Item.Cost = 25
   
    CASE 8
      Item.Num = 8
      Item.IName = "MultiHeal2"
      'Max Length "----------------------------------------"
      Item.Desc = "Heals all the players status in battle"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "ZCSPAU"
      Item.Cost = 100

    CASE 9
      Item.Num = 9
      Item.IName = "Life1"
      'Max Length "----------------------------------------"
      Item.Desc = "Brings a player back to life with 1 HP"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 1
      Item.MPEffect = 0
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "D"
      Item.Cost = 50

    CASE 10
      Item.Num = 10
      Item.IName = "Life2"
      'Max Length "----------------------------------------"
      Item.Desc = "Brings a player back to life with MAX HP"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 9999
      Item.MPEffect = 0
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "D"
      Item.Cost = 500

    CASE 11
      Item.Num = 11
      Item.IName = "Life3"
      'Max Length "----------------------------------------"
      Item.Desc = "Brings all players back to life w/ MAXHP"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 9999
      Item.MPEffect = 0
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "ZND"
      Item.Cost = 2250

    CASE 12
      Item.Num = 12
      Item.IName = "Elixer"
      'Max Length "----------------------------------------"
      Item.Desc = "Heals a players HP and MP completely"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 9999
      Item.MPEffect = 999
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "N"
      Item.Cost = 1000

    CASE 13
      Item.Num = 13
      Item.IName = "MegaElixer"
      'Max Length "----------------------------------------"
      Item.Desc = "Heals all players HP and MP completely"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 9999
      Item.MPEffect = 999
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "ZN"
      Item.Cost = 10000

    CASE 14
      Item.Num = 14
      Item.IName = "WillowSeed"
      'Max Length "----------------------------------------"
      Item.Desc = "Heals players MP 25"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 0
      Item.MPEffect = 25
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "N"
      Item.Cost = 50

    CASE 15
      Item.Num = 15
      Item.IName = "WalnutSeed"
      'Max Length "----------------------------------------"
      Item.Desc = "Heals players MP 150"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 0
      Item.MPEffect = 150
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "N"
      Item.Cost = 1000

    CASE 16
      Item.Num = 16
      Item.IName = "Acorn Seed"
      'Max Length "----------------------------------------"
      Item.Desc = "Heals players MP to MAX MP"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 0
      Item.MPEffect = 999
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "N"
      Item.Cost = 2500

    CASE 17
      Item.Num = 17
      Item.IName = "Acorn Root"
      'Max Length "----------------------------------------"
      Item.Desc = "Heals all players MP to MAX MP"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 0
      Item.MPEffect = 999
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = "ZN"
      Item.Cost = 2500

    CASE 18
      Item.Num = 18
      Item.IName = "MistPotion"
      'Max Length "----------------------------------------"
      Item.Desc = "Looks odd, feels cold."
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 2
      Item.StrengthEffect = 5
      Item.DefenseEffect = 3
      Item.StatusEffect = "N"
      Item.Cost = 2

    CASE 19
      Item.Num = 19
      Item.IName = "SpeedDrink"
      'Max Length "----------------------------------------"
      Item.Desc = "Increases a players speed in battle"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 50
      Item.StrengthEffect = 0
      Item.DefenseEffect = 0
      Item.StatusEffect = ""
      Item.Cost = 200

    CASE 20
      Item.Num = 20
      Item.IName = "SldDrink"
      'Max Length "----------------------------------------"
      Item.Desc = "Increases a players defense in battle"
      Item.ItemType = 1 'Items
      Item.EquipWho = ""
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 50
      Item.StatusEffect = ""
      Item.Cost = 200



'********** ARMOR ************  100-199
   
    CASE 100
      Item.Num = 100
      Item.IName = "Tunic"
      'Max Length "----------------------------------------"
      Item.Desc = "This a piece of clothing"
      Item.ItemType = 2 'Armor
      Item.EquipWho = "ABCD"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 1
      Item.StrengthEffect = 0
      Item.DefenseEffect = 3
      Item.StatusEffect = "C"
      Item.Cost = 50
   
    CASE 101
      Item.Num = 101
      Item.IName = "LeatherAmr"
      'Max Length "----------------------------------------"
      Item.Desc = "Leather armor sure beats a tunic!"
      Item.ItemType = 2 'Armor
      Item.EquipWho = "AD"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 3
      Item.StrengthEffect = 0
      Item.DefenseEffect = 10
      Item.StatusEffect = ""
      Item.Cost = 100
   
    CASE 102
      Item.Num = 102
      Item.IName = "Chain Mail"
      'Max Length "----------------------------------------"
      Item.Desc = "Suit of armor which is pretty cool."
      Item.ItemType = 2 'Armor
      Item.EquipWho = "ADF"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 0
      Item.StrengthEffect = 0
      Item.DefenseEffect = 25
      Item.StatusEffect = ""
      Item.Cost = 1530

    CASE 103
      Item.Num = 103
      Item.IName = "Full Mail"
      'Max Length "----------------------------------------"
      Item.Desc = "Thicker chain mail, with protective hood"
      Item.ItemType = 2 'Armor
      Item.EquipWho = "AD"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 5
      Item.StrengthEffect = 0
      Item.DefenseEffect = 35
      Item.StatusEffect = ""
      Item.Cost = 11800

    CASE 104
      Item.Num = 104
      Item.IName = "Plate Mail"
      'Max Length "----------------------------------------"
      Item.Desc = "Thick plates offer great protection."
      Item.ItemType = 2 'Armor
      Item.EquipWho = "ADF"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 4
      Item.StrengthEffect = 0
      Item.DefenseEffect = 50
      Item.StatusEffect = ""
      Item.Cost = 18000

    CASE 105
      Item.Num = 105
      Item.IName = "Silver Amr"
      'Max Length "----------------------------------------"
      Item.Desc = "Silver Armor, similar to plate mail."
      Item.ItemType = 2 'Armor
      Item.EquipWho = "A"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 9
      Item.StrengthEffect = 0
      Item.DefenseEffect = 65
      Item.StatusEffect = ""
      Item.Cost = 35000&
   
    CASE 106
      Item.Num = 106
      Item.IName = "Spiked Amr"
      'Max Length "----------------------------------------"
      Item.Desc = "Stupid armor that was forced in the game"
      Item.ItemType = 2 'Armor
      Item.EquipWho = "A"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 14
      Item.StrengthEffect = 5
      Item.DefenseEffect = 66
      Item.StatusEffect = ""
      Item.Cost = 2

    CASE 107
      Item.Num = 107
      Item.IName = "Moon Amr"
      'Max Length "----------------------------------------"
      Item.Desc = "Created with the moon rock!?"
      Item.ItemType = 2 'Armor
      Item.EquipWho = "A"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 20
      Item.StrengthEffect = 10
      Item.DefenseEffect = 85
      Item.StatusEffect = ""
      Item.Cost = 60000&

    CASE 108
      Item.Num = 108
      Item.IName = "Fire Amr"
      'Max Length "----------------------------------------"
      Item.Desc = "Created with the lava rock!?"
      Item.ItemType = 2 'Armor
      Item.EquipWho = "F"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 20
      Item.StrengthEffect = 10
      Item.DefenseEffect = 85
      Item.StatusEffect = ""
      Item.Cost = 60000&

    CASE 109
      Item.Num = 109
      Item.IName = "Cloth Amr"
      'Max Length "----------------------------------------"
      Item.Desc = " 25% Cotton, 75% Polyester (Ichy!)"
      Item.ItemType = 2 'Armor
      Item.EquipWho = "BCE"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 2
      Item.StrengthEffect = 0
      Item.DefenseEffect = 6
      Item.StatusEffect = ""
      Item.Cost = 40

    CASE 110
      Item.Num = 110
      Item.IName = "Padded Clk"
      'Max Length "----------------------------------------"
      Item.Desc = "Soft, shock dampering material."
      Item.ItemType = 2 'Armor
      Item.EquipWho = "BC"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 3
      Item.StrengthEffect = 0
      Item.DefenseEffect = 10
      Item.StatusEffect = ""
      Item.Cost = 250

    CASE 111
      Item.Num = 111
      Item.IName = "Full Clk"
      'Max Length "----------------------------------------"
      Item.Desc = "IT HAS A HOOD!"
      Item.ItemType = 2 'Armor
      Item.EquipWho = "C"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 4
      Item.StrengthEffect = 0
      Item.DefenseEffect = 15
      Item.StatusEffect = ""
      Item.Cost = 750

    CASE 112
      Item.Num = 112
      Item.IName = "Mage Clk"
      'Max Length "----------------------------------------"
      Item.Desc = "Top of the line in protective wear!"
      Item.ItemType = 2 'Armor
      Item.EquipWho = "C"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 10
      Item.StrengthEffect = 0
      Item.DefenseEffect = 45
      Item.StatusEffect = ""
      Item.Cost = 3750

    CASE 113
      Item.Num = 113
      Item.IName = "White Robe"
      'Max Length "----------------------------------------"
      Item.Desc = "Umm let's see, it's white! (like snow)"
      Item.ItemType = 2 'Armor
      Item.EquipWho = "E"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 5
      Item.StrengthEffect = 0
      Item.DefenseEffect = 20
      Item.StatusEffect = ""
      Item.Cost = 1750

    CASE 114
      Item.Num = 114
      Item.IName = "White Gown"
      'Max Length "----------------------------------------"
      Item.Desc = "Umm let's see, it's ALSO white!"
      Item.ItemType = 2 'Armor
      Item.EquipWho = "E"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 15
      Item.StrengthEffect = 0
      Item.DefenseEffect = 30
      Item.StatusEffect = ""
      Item.Cost = 8750
   
    CASE 115
      Item.Num = 115
      Item.IName = "Lucid Gown"
      'Max Length "----------------------------------------"
      Item.Desc = "Heh heh :) (It's Clear...hmmmmm)"
      Item.ItemType = 2 'Armor
      Item.EquipWho = "E"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 30
      Item.StrengthEffect = 0
      Item.DefenseEffect = 50
      Item.StatusEffect = ""
      Item.Cost = 20000

    CASE 116
      Item.Num = 116
      Item.IName = "Fire Amr"
      'Max Length "----------------------------------------"
      Item.Desc = "(It's a HOT commodity, literally)"
      Item.ItemType = 2 'Armor
      Item.EquipWho = "F"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 30
      Item.StrengthEffect = 20
      Item.DefenseEffect = 80
      Item.StatusEffect = ""
      Item.Cost = 60000&
   
    CASE 117
      Item.Num = 117
      Item.IName = "Crstl Gown"
      'Max Length "----------------------------------------"
      Item.Desc = "A beautiful crystal gown."
      Item.ItemType = 2 'Armor
      Item.EquipWho = "H"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 30
      Item.StrengthEffect = 10
      Item.DefenseEffect = 70
      Item.StatusEffect = ""
      Item.Cost = 70000
   
    CASE 118
      Item.Num = 118
      Item.IName = "Grass Skrt"
      'Max Length "----------------------------------------"
      Item.Desc = "Just the skirt, but VERY EFFECTIVE!"
      Item.ItemType = 2 'Armor
      Item.EquipWho = "G"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 27
      Item.StrengthEffect = 8
      Item.DefenseEffect = 45
      Item.StatusEffect = ""
      Item.Cost = 10

    CASE 119
      Item.Num = 119
      Item.IName = "CottonRobe"
      'Max Length "----------------------------------------"
      Item.Desc = "100% Cotton / 100% Coton / 100% Algodon"
      Item.ItemType = 2 'Armor
      Item.EquipWho = "G"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 25
      Item.StrengthEffect = 5
      Item.DefenseEffect = 55
      Item.StatusEffect = ""
      Item.Cost = 2

    CASE 120
      Item.Num = 120
      Item.IName = "Ash Robe"
      'Max Length "----------------------------------------"
      Item.Desc = "Say it while holding your tounge!"
      Item.ItemType = 2 'Armor
      Item.EquipWho = "G"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 35
      Item.StrengthEffect = 10
      Item.DefenseEffect = 65
      Item.StatusEffect = ""
      Item.Cost = 50000&

' (Armor) ******* SHEILDS ************
    CASE 150
      Item.Num = 150
      Item.IName = "Wood Sld"
      'Max Length "----------------------------------------"
      Item.Desc = "A handy shield carved from dead oak."
      Item.ItemType = 3 'Technically Armor, but used in a weapon slot
      Item.EquipWho = "AD"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 1
      Item.StrengthEffect = 0
      Item.DefenseEffect = 6
      Item.StatusEffect = ""
      Item.Cost = 50

    CASE 151
      Item.Num = 151
      Item.IName = "Leathr Sld"
      'Max Length "----------------------------------------"
      Item.Desc = "A sturdy Leather Shield"
      Item.ItemType = 3 'Technically Armor, but used in a weapon slot
      Item.EquipWho = "AD"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 2
      Item.StrengthEffect = 0
      Item.DefenseEffect = 10
      Item.StatusEffect = ""
      Item.Cost = 150
   
    CASE 152
      Item.Num = 152
      Item.IName = "Iron Sld"
      'Max Length "----------------------------------------"
      Item.Desc = "Might slow ya down but worth the Defense"
      Item.ItemType = 3 'Technically Armor, but used in a weapon slot
      Item.EquipWho = "ADF"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = -2
      Item.StrengthEffect = 0
      Item.DefenseEffect = 18
      Item.StatusEffect = ""
      Item.Cost = 500
   
    CASE 153
      Item.Num = 153
      Item.IName = "SilverSld"
      'Max Length "----------------------------------------"
      Item.Desc = "A nice Silver Shield made from the best!"
      Item.ItemType = 3 'Technically Armor, but used in a weapon slot
      Item.EquipWho = "ADF"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = -1
      Item.StrengthEffect = 0
      Item.DefenseEffect = 23
      Item.StatusEffect = ""
      Item.Cost = 1300
   
    CASE 154
      Item.Num = 154
      Item.IName = "Gold SLD"
      'Max Length "----------------------------------------"
      Item.Desc = "A Solid Gold Sheid worth some BIG BUCKS!"
      Item.ItemType = 3 'Technically Armor, but used in a weapon slot
      Item.EquipWho = "D"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = -5
      Item.StrengthEffect = 0
      Item.DefenseEffect = 30
      Item.StatusEffect = ""
      Item.Cost = 2990
   
    CASE 155
      Item.Num = 155
      Item.IName = "Ice Sld"
      'Max Length "----------------------------------------"
      Item.Desc = "Strong but its frozen water (Go Fig)"
      Item.ItemType = 3 'Technically Armor, but used in a weapon slot
      Item.EquipWho = "A"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 3
      Item.StrengthEffect = 0
      Item.DefenseEffect = 26
      Item.StatusEffect = ""
      Item.Cost = 2750
   
    CASE 156
      Item.Num = 156
      Item.IName = "SpikedSld"
      'Max Length "----------------------------------------"
      Item.Desc = "A spiked shield that improves your str!"
      Item.ItemType = 3 'Technically Armor, but used in a weapon slot
      Item.EquipWho = "A"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 5
      Item.StrengthEffect = 9
      Item.DefenseEffect = 28
      Item.StatusEffect = ""
      Item.Cost = 2850

    CASE 157
      Item.Num = 157
      Item.IName = "Fire Sld"
      'Max Length "----------------------------------------"
      Item.Desc = "Another HOT item."
      Item.ItemType = 3 'Technically Armor, but used in a weapon slot
      Item.EquipWho = "F"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 3
      Item.StrengthEffect = 3
      Item.DefenseEffect = 28
      Item.StatusEffect = ""
      Item.Cost = 2850




'********** WEAPONS ************  200-299

    CASE 200
      Item.Num = 200
      Item.IName = "SDagger"
      'Max Length "----------------------------------------"
      Item.Desc = "A small dagger"
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "AD"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 2
      Item.StrengthEffect = 5
      Item.DefenseEffect = 0
      Item.StatusEffect = ""
      Item.Cost = 30
     
    CASE 201
      Item.Num = 201
      Item.IName = "LDagger"
      'Max Length "----------------------------------------"
      Item.Desc = "A small dagger"
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "AD"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 6
      Item.StrengthEffect = 10
      Item.DefenseEffect = 2
      Item.StatusEffect = ""
      Item.Cost = 100

    CASE 202
      Item.Num = 202
      Item.IName = "Long Swd"
      'Max Length "----------------------------------------"
      Item.Desc = "A long 54 inch blade."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "A"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 16
      Item.StrengthEffect = 20
      Item.DefenseEffect = 8
      Item.StatusEffect = ""
      Item.Cost = 700

    CASE 203
      Item.Num = 203
      Item.IName = "Mace"
      'Max Length "----------------------------------------"
      Item.Desc = "Sharp ball attached to a 2 ft. chain"
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "A"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 26
      Item.StrengthEffect = 25
      Item.DefenseEffect = 7
      Item.StatusEffect = ""
      Item.Cost = 1000

    CASE 204
      Item.Num = 204
      Item.IName = "B Spear"
      'Max Length "----------------------------------------"
      Item.Desc = "5 ft. spear with a sharp barbed end"
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "A"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 30
      Item.StrengthEffect = 38
      Item.DefenseEffect = 12
      Item.StatusEffect = ""
      Item.Cost = 3000

    CASE 205
      Item.Num = 205
      Item.IName = "Cross Bow"
      'Max Length "----------------------------------------"
      Item.Desc = "High powered cross bow w/ unlimited shot"
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "A"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 46
      Item.StrengthEffect = 60
      Item.DefenseEffect = 25
      Item.StatusEffect = ""
      Item.Cost = 10000

    CASE 206
      Item.Num = 206
      Item.IName = "Moon Swd"
      'Max Length "----------------------------------------"
      Item.Desc = "Sword crafted from moon rock."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "A"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 80
      Item.StrengthEffect = 150
      Item.DefenseEffect = 45
      Item.StatusEffect = ""
      Item.Cost = 50000&

    CASE 207
      Item.Num = 207
      Item.IName = "Moon Orb"
      'Max Length "----------------------------------------"
      Item.Desc = "Glowing moon rock, shaped by time."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "A"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 80
      Item.StrengthEffect = 80
      Item.DefenseEffect = 45
      Item.StatusEffect = ""
      Item.Cost = 100000

    CASE 208
      Item.Num = 208
      Item.IName = "Iron Sword"
      'Max Length "----------------------------------------"
      Item.Desc = "Skillfully crafted sword."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "D"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 15
      Item.StrengthEffect = 20
      Item.DefenseEffect = 7
      Item.StatusEffect = ""
      Item.Cost = 1200

    CASE 209
      Item.Num = 209
      Item.IName = "Gold Sword"
      'Max Length "----------------------------------------"
      Item.Desc = "Heavy but effective gold plated sword"
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "D"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 8
      Item.StrengthEffect = 50
      Item.DefenseEffect = 27
      Item.StatusEffect = ""
      Item.Cost = 12000

    CASE 210
      Item.Num = 210
      Item.IName = "SwiftSword"
      'Max Length "----------------------------------------"
      Item.Desc = "Strong but light sword w/ a magical feel"
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "D"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 30
      Item.StrengthEffect = 70
      Item.DefenseEffect = 35
      Item.StatusEffect = ""
      Item.Cost = 23000

    CASE 211
      Item.Num = 211
      Item.IName = "Dirk"
      'Max Length "----------------------------------------"
      Item.Desc = "Blunt object for begining mages."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "BCE"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 3
      Item.StrengthEffect = 5
      Item.DefenseEffect = 1
      Item.StatusEffect = ""
      Item.Cost = 100

    CASE 212
      Item.Num = 212
      Item.IName = "Ivory Staff"
      'Max Length "----------------------------------------"
      Item.Desc = "Delicately carved ivory staff."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "CG"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 7
      Item.StrengthEffect = 10
      Item.DefenseEffect = 2
      Item.StatusEffect = ""
      Item.Cost = 500
   
    CASE 213
      Item.Num = 213
      Item.IName = "Mage Scptr"
      'Max Length "----------------------------------------"
      Item.Desc = "Magical Scepter w/ special powers."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "C"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 10
      Item.StrengthEffect = 20
      Item.DefenseEffect = 8
      Item.StatusEffect = ""
      Item.Cost = 1000

    CASE 214
      Item.Num = 214
      Item.IName = "CrystalRod"
      'Max Length "----------------------------------------"
      Item.Desc = "Short rod made from quartz crystal."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "E"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 7
      Item.StrengthEffect = 10
      Item.DefenseEffect = 2
      Item.StatusEffect = ""
      Item.Cost = 500

    CASE 215
      Item.Num = 215
      Item.IName = "Crystal"
      'Max Length "----------------------------------------"
      Item.Desc = "Palmsize crystal rock."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "E"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 9
      Item.StrengthEffect = 20
      Item.DefenseEffect = 5
      Item.StatusEffect = ""
      Item.Cost = 2000

    CASE 216
      Item.Num = 216
      Item.IName = "PowerStaff"
      'Max Length "----------------------------------------"
      Item.Desc = "Magical Staff"
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "E"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 11
      Item.StrengthEffect = 30
      Item.DefenseEffect = 10
      Item.StatusEffect = ""
      Item.Cost = 13000

    CASE 218
      Item.Num = 218
      Item.IName = "DiamondRod"
      'Max Length "----------------------------------------"
      Item.Desc = "Staff studded with powerful diamonds."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "E"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 31
      Item.StrengthEffect = 40
      Item.DefenseEffect = 15
      Item.StatusEffect = ""
      Item.Cost = 33000&

    CASE 219
      Item.Num = 219
      Item.IName = "Pick Axe"
      'Max Length "----------------------------------------"
      Item.Desc = "Narrow, but sharp axe."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "F"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 10
      Item.StrengthEffect = 10
      Item.DefenseEffect = 10
      Item.StatusEffect = ""
      Item.Cost = 600

    CASE 220
      Item.Num = 220
      Item.IName = "Sldg Hmr"
      'Max Length "----------------------------------------"
      Item.Desc = "Heavy iron sledge hammer"
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "F"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 8
      Item.StrengthEffect = 32
      Item.DefenseEffect = 14
      Item.StatusEffect = ""
      Item.Cost = 900

    CASE 221
      Item.Num = 221
      Item.IName = "War Hmr"
      'Max Length "----------------------------------------"
      Item.Desc = "Lighter than sledge, and more effective. "
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "F"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 14
      Item.StrengthEffect = 45
      Item.DefenseEffect = 24
      Item.StatusEffect = ""
      Item.Cost = 2400

    CASE 222
      Item.Num = 222
      Item.IName = "Double Axe"
      'Max Length "----------------------------------------"
      Item.Desc = "Double Bladed, extra sharp battle axe."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "F"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 25
      Item.StrengthEffect = 57
      Item.DefenseEffect = 34
      Item.StatusEffect = ""
      Item.Cost = 17000

    CASE 223
      Item.Num = 223
      Item.IName = "Fire Axe"
      'Max Length "----------------------------------------"
      Item.Desc = "Blade of axe consumed by an icy flame."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "F"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 45
      Item.StrengthEffect = 85
      Item.DefenseEffect = 45
      Item.StatusEffect = ""
      Item.Cost = 45000&

    CASE 224
      Item.Num = 224
      Item.IName = "Fire Orb"
      'Max Length "----------------------------------------"
      Item.Desc = "Glowing lava rock, shaped by time."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "F"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 45
      Item.StrengthEffect = 85
      Item.DefenseEffect = 45
      Item.StatusEffect = ""
      Item.Cost = 30000

    CASE 225
      Item.Num = 225
      Item.IName = "Air Staff"
      'Max Length "----------------------------------------"
      Item.Desc = "Extra Light, magically floaing staff."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "H"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 25
      Item.StrengthEffect = 35
      Item.DefenseEffect = 20
      Item.StatusEffect = ""
      Item.Cost = 30000
   
    CASE 226
      Item.Num = 226
      Item.IName = "CrystalOrb"
      'Max Length "----------------------------------------"
      Item.Desc = "Diamond Crystal formed over time."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "H"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 20
      Item.StrengthEffect = 30
      Item.DefenseEffect = 20
      Item.StatusEffect = ""
      Item.Cost = 20000

    CASE 227
      Item.Num = 227
      Item.IName = "Pet Staff"
      'Max Length "----------------------------------------"
      Item.Desc = "Petrified wooden staff."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "G"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 30
      Item.StrengthEffect = 50
      Item.DefenseEffect = 25
      Item.StatusEffect = ""
      Item.Cost = 35000&

    CASE 228
      Item.Num = 228
      Item.IName = "Ash Orb"
      'Max Length "----------------------------------------"
      Item.Desc = "Orb created by 1000 year old trees."
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "G"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 30
      Item.StrengthEffect = 45
      Item.DefenseEffect = 23
      Item.StatusEffect = ""
      Item.Cost = 25000

    CASE 229
      Item.Num = 229
      Item.IName = "Power Ring"
      'Max Length "----------------------------------------"
      Item.Desc = "Increases strength by 40"
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "A"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 0
      Item.StrengthEffect = 40
      Item.DefenseEffect = 0
      Item.StatusEffect = ""
      Item.Cost = 4000

    CASE 230
      Item.Num = 230
      Item.IName = "Quartz Ring"
      'Max Length "----------------------------------------"
      Item.Desc = "Quartz stone placed on a gold ring"
      Item.ItemType = 3  'Weapons
      Item.EquipWho = "EGH"
      Item.HPEffect = 0
      Item.MPEffect = 0
      Item.SpeedEffect = 30
      Item.StrengthEffect = 0
      Item.DefenseEffect = 30
      Item.StatusEffect = ""
      Item.Cost = 25000





END SELECT
END SUB