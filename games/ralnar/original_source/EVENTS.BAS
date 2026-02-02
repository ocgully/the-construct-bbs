DECLARE SUB TCHandler (TNum AS INTEGER)
DECLARE SUB BGMPlay ()
DECLARE SUB LoadMap (Name$)
DECLARE SUB DispAll ()
DECLARE SUB Inn (Price AS INTEGER)
DECLARE SUB MessBox (Text$)
DECLARE SUB ResetMessBox (Heading$)
DECLARE SUB EIHandler (X AS INTEGER, Y AS INTEGER, MapName$)
DECLARE SUB PAUSE ()
DECLARE SUB Items (Num AS INTEGER, Item AS ANY)
DECLARE SUB Shop (ShopName$, TotalItems AS INTEGER)
DECLARE SUB DispAir (D AS INTEGER, SX!, SY!, Amount AS INTEGER)
DECLARE SUB DispHero (D AS INTEGER, T!, SX!, SY!, Amount AS INTEGER)
DECLARE SUB DispScreen ()
DECLARE SUB TextFont (S$, X!, Y!, C!, S!)

TYPE ItemType
  Num AS INTEGER
  Iname AS STRING * 10
  Desc AS STRING * 40
  ItemType AS INTEGER '(1= Item, 2 = Armor, 3= Weapon, 4 = Specials)
  EquipWho AS STRING * 4 'Player # (1-4)
  HPEffect AS INTEGER
  MPEffect AS INTEGER
  SpeedEffect AS INTEGER
  StrengthEffect AS INTEGER
  DefenseEffect AS INTEGER
  StatusEffect AS STRING * 4
  Cost AS LONG
END TYPE

TYPE ShopItemType
  Cost AS LONG
  Item AS ItemType
END TYPE

TYPE InvType
  Amount AS INTEGER  'total # of that item
  Item AS ItemType
END TYPE


TYPE EnvironmentType
  Music AS INTEGER  'Music On or Off
  BGC AS INTEGER    'Backgound Color  1 = Blue
  FGC AS INTEGER    'Foreground Color 7 = Grey
  MS AS INTEGER 'Message Speed 1-9
  ML AS INTEGER 'Message Line
  Icon AS INTEGER
  LastB AS INTEGER 'Number of moves since the last battle
  Songnum AS INTEGER 'The Song Number for the music to play on
  SongLoc AS INTEGER 'The Line Location that the current song is on
END TYPE

TYPE LocationType
  X AS INTEGER
  Y AS INTEGER
  NX AS INTEGER
  NY AS INTEGER
  MapName AS STRING * 10
  MapDimX AS INTEGER
  MapDimY AS INTEGER
  Dir AS INTEGER
  Mode AS INTEGER
  ShipX AS INTEGER
  ShipY AS INTEGER
  ShipExist AS INTEGER
  AirX AS INTEGER
  AirY AS INTEGER
  AirExist AS INTEGER
  Enemies AS INTEGER
  FullSeeSize AS INTEGER
  HalfSeeSize AS INTEGER

END TYPE

TYPE PlayerType
  PLet AS STRING * 1
  PName AS STRING * 10
  LeftArm AS ItemType
  RightArm AS ItemType
  Armor AS ItemType
  HP AS INTEGER
  MaxHP AS INTEGER
  MP AS INTEGER
  MaxMp AS INTEGER
  Status  AS INTEGER   'Battle use only
  Strength AS INTEGER
  BaseDefense AS INTEGER
  MAttack AS INTEGER
  MDefense AS INTEGER
  PhysicalLearnRate AS INTEGER
  MagicLearnRate AS INTEGER
  PLimit AS INTEGER
  MLimit AS INTEGER
  XP AS LONG
  XPNext AS LONG
  Level AS INTEGER
  Speed AS INTEGER
  Gold AS LONG
END TYPE

TYPE SpecialType 'Used for keeping track of special items
  CKey AS INTEGER
END TYPE

 
CONST MaxInv = 50
CONST True = 1
CONST False = 0
CONST DirRight = 1, DirDown = 2, DirLeft = 3, DirUp = 4
CONST Down = 80, Up = 72, Left = 75, Right = 77

COMMON SHARED NumberMembers AS INTEGER, Player   AS PlayerType
COMMON SHARED Enemies AS INTEGER, Map   AS INTEGER
COMMON SHARED Env AS EnvironmentType, Location AS LocationType
COMMON SHARED MapIcons   AS INTEGER, Hero   AS INTEGER, Boat   AS INTEGER
COMMON SHARED Air   AS INTEGER, ShopItems   AS ShopItemType
COMMON SHARED PlayerInv   AS InvType, Special AS SpecialType
COMMON SHARED Tchest   AS INTEGER

DIM SHARED ItemNum AS INTEGER
DIM SHARED TempItem AS ItemType

DEFSNG A-Z
SUB EIHandler (X AS INTEGER, Y AS INTEGER, MapName$)
TX = X
TY = Y

IF Location.Dir = Left THEN X = X - 1
IF Location.Dir = Right THEN X = X + 1
IF Location.Dir = Up THEN Y = Y - 1
IF Location.Dir = Down THEN Y = Y + 1



SELECT CASE MapName$

'*** TOWN 1            '----------"  10 characters
  CASE IS = "TOWN1     "
    
     IF ((X = 18) AND (Y = 18)) THEN 'Middle Tomb
        CALL ResetMessBox("Old Mans Tombstone")
        CALL MessBox("Here lies an old mans body.")
        CALL MessBox("That is it, just the body, well")
        CALL MessBox("it did have some clothes on but")
        CALL MessBox("they have probably dissapeared")
        CALL MessBox("over the years and all.")
      END IF

     IF ((X = 17) AND ((Y = 19) OR (Y = 17))) OR ((X = 19) AND ((Y = 17) OR (Y = 19))) THEN 'Middle Tomb
       CALL ResetMessBox("")
        CALL MessBox("This is a tombstone.")
        PSET (217, 63), Env.FGC: DRAW "d2r3gehf"
        CALL PAUSE
      END IF

     IF ((X = 7) AND (Y = 17)) THEN 'Sign
        CALL ResetMessBox("Town Sign")
        CALL MessBox("Welcome to Arkania.")
        PSET (217, 63), Env.FGC: DRAW "d2r3gehf"
        CALL PAUSE
      END IF

     IF ((X = 12) AND (Y = 30)) THEN ' "AL"
        CALL ResetMessBox("Jaylo")
        CALL MessBox("Hi im Jaylo i am a black smith.")
        CALL MessBox("I am the weapon shop owners")
        CALL MessBox("son.  For some reason people")
        CALL MessBox("call me Al i don`t understand.")
        PSET (217, 63), Env.FGC: DRAW "d2r3gehf"
        CALL PAUSE
      END IF

     IF ((X = 13) AND (Y = 24)) THEN ' "Old lady"
        CALL ResetMessBox("Old lady")
        CALL MessBox("I twat I taw a putty cat.")
        CALL MessBox("I did ,I did taw a putty cat.")
        PSET (217, 63), Env.FGC: DRAW "d2r3gehf"
        CALL PAUSE
      END IF

     IF ((X = 18) AND (Y = 19)) THEN ' "Young Woman"
        CALL ResetMessBox("Young Woman")
        CALL MessBox("My grandfather died many years.")
        CALL MessBox("ago. Every week i put fresh")
        CALL MessBox("flowers on his grave, although")
        CALL MessBox("lately, all the flowers have")
        CALL MessBox("been dead, I don't understand.")
      END IF
    
     IF ((X = 15) AND (Y = 7)) THEN ' "Little Girl"
        CALL ResetMessBox("Little girl in pink dress")
        CALL MessBox("My mommy is always looking for")
        CALL MessBox("pretty flowers, but ever since")
        CALL MessBox("the monsters came, nothing")
        CALL MessBox("has been very pretty. Mommy says")
        CALL MessBox("the plaque on the wall tells")
        CALL ResetMessBox("Little girl in pink dress")
        CALL MessBox("a story about monsters, and")
        CALL MessBox("how a a boy of light will come")
        CALL MessBox("and help us.  I hope the boy will")
        CALL MessBox("be really cute, cuz all the boys")
        CALL MessBox("here are ugly.")
      END IF
  
     IF ((X = 23) AND (Y = 12)) THEN ' "Old Man at inn
      'If God of Shrine one is NOT done THen
        CALL ResetMessBox("Crazy Looking Old Man ")
        CALL MessBox("Shhh.. don't tell any one this, but")
        CALL MessBox("I am the god of time, and i have")
        CALL MessBox("come to you for assistance.  Ever since")
        CALL MessBox("the monsters came us gods have had a hard")
        CALL MessBox("time, taking care of everything, but the")
        CALL ResetMessBox("God of Time")
        CALL MessBox("cause of the monsters is a man in the")
        CALL MessBox("tower of the shrine to the south west.")
        CALL MessBox("I will meet you there, and after we take")
        CALL MessBox("care of him I will give you a precious")
        CALL MessBox("item.")
       'ELSEIF Shine one is done Then
        'CALL ResetMessBox("God of Time ")
        'CALL MessBox("Thanks for saving me, I will follow you")
        'CALL MessBox("from town to town now, and assist you when")
        'CALL MessBox("the time comes.")
        'PSET (217, 63), Env.FGC: DRAW "d2r3gehf"
        'CALL PAUSE
       'END IF
      END IF

'*** TOWN 2
  CASE IS = "TOWN2     "
   
     IF ((X = 6) AND (Y = 11)) THEN 'Sign
        CALL ResetMessBox("Town Sign")
        CALL MessBox("Welcome to Thalos.")
        PSET (217, 63), Env.FGC: DRAW "d2r3gehf"
        CALL PAUSE
      END IF
    
     IF ((X = 16) AND (Y = 15)) THEN ' "Old Man"
        CALL ResetMessBox("Kind Old Man")
        CALL MessBox("We have been married over 50 years")
        CALL MessBox("and nothing will ever seperate us")
        CALL MessBox("not even if a 20 year old, extremely")
        CALL MessBox("beautiful, woman, with long blonde")
        CALL MessBox("hair, deep blue eyes, sleak curves,")
        CALL ResetMessBox("Kind Old Man with big grin")
        CALL MessBox("lucious lips, and a... hmmm well,")
        CALL MessBox("lets not go that far.  But you")
        CALL MessBox("get the idea.")
        PSET (217, 63), Env.FGC: DRAW "d2r3gehf"
        CALL PAUSE
      END IF
    
     IF ((X = 15) AND (Y = 15)) THEN ' "Old lady"
        CALL ResetMessBox("Kind Old Woman")
        CALL MessBox("Who is this strange man next to me")
        CALL MessBox("with drool hanging out of his mouth.")
        CALL MessBox("OH, wait, he is my loving husband, how")
        CALL MessBox("could I ever forget.")
        PSET (217, 63), Env.FGC: DRAW "d2r3gehf"
        CALL PAUSE
      END IF

     IF ((X = 9) AND (Y = 11)) THEN ' "Young Woman"
        CALL ResetMessBox("Young Woman")
        CALL MessBox("The desert to the north of here is very")
        CALL MessBox("hot.  I heard of a lush forest beyond it.")
        CALL MessBox("Although the forest is so thick it is")
        CALL MessBox("like a maze.")
        CALL MessBox("Be careful if you venture near there.")
      END IF
    
     IF ((X = 17) AND (Y = 9)) THEN ' "Old Man at inn
        CALL ResetMessBox("God of Time")
        CALL MessBox("Be brave young warrior, and travel")
        CALL MessBox("to the forest north of here.  There you")
        CALL MessBox("will find someone who will aid you more")
        CALL MessBox("in your quest.")
        PSET (217, 63), Env.FGC: DRAW "d2r3gehf"
        CALL PAUSE
      END IF

     IF ((X = 17) AND (Y = 5)) THEN ' Hidden Treasure chest
       CALL TCHandler(1)
      END IF
      
'*** CAVE 1
  CASE IS = "CAVE1     "
    
     IF ((X = 20) AND (Y = 22)) THEN ' Hidden Treasure chest
       CALL TCHandler(2)
      END IF
    
     IF ((X = 25) AND (Y = 13)) THEN ' Hidden Treasure chest
       CALL TCHandler(3)
      END IF
    
     IF ((X = 7) AND (Y = 15)) THEN ' Hidden Treasure chest
       CALL TCHandler(4)
      END IF



END SELECT

X = TX
Y = TY
CALL DispScreen
CALL DispAll
END SUB

SUB EPHandler (X AS INTEGER, Y AS INTEGER, MapName$)
'Event Passable Handler
'All Map Names are 10 character strings
SOUND 65, 2   'Audible acknowledgement of an event being triggered

SELECT CASE MapName$

            '----------"  10 characters
  CASE IS = "TOWN1     "
     IF (X = 11) AND (Y = 8) THEN  'Armor Shop
        CALL Items(100, ShopItems(1).Item)
           ShopItems(1).Cost = 50
        CALL Items(101, ShopItems(2).Item)
           ShopItems(2).Cost = 150
        CALL Items(110, ShopItems(3).Item)
           ShopItems(3).Cost = 250
        CALL Items(150, ShopItems(4).Item)
           ShopItems(4).Cost = 250
        CALL Shop("A", 4)

    
     ELSEIF (X = 25) AND (Y = 12) THEN  'Inn
       CALL Inn(20)
    
     ELSEIF (X = 27) AND (Y = 28) THEN  'Old Man's house
      IF Player(2).PLet <> "B" THEN
        CALL LoadMap("OldManH")
        MapName$ = "OLDMANH"
        Location.X = 11
        Location.Y = 9
        Location.Mode = 1 'Walking
        Location.Dir = Up
        CALL DispAll
      END IF
     ELSEIF (X = 13) AND (Y = 28) THEN 'Weapon Shop
        CALL Items(200, ShopItems(1).Item)
           ShopItems(1).Cost = 30
        CALL Items(201, ShopItems(2).Item)
           ShopItems(2).Cost = 180
        CALL Items(211, ShopItems(3).Item)
           ShopItems(3).Cost = 100
        CALL Shop("W", 3)
      
     ELSEIF (X = 9) AND (Y = 29) THEN 'Item Shop
        CALL Items(1, ShopItems(1).Item)
           ShopItems(1).Cost = 10
        CALL Items(7, ShopItems(2).Item)
           ShopItems(2).Cost = 25
        CALL Items(9, ShopItems(3).Item)
           ShopItems(3).Cost = 50
        CALL Shop("I", 3)
    
     ELSE 'Exit Town
       CALL LoadMap("WORLD")
       MapName$ = "WORLD"
       Location.X = 130
       Location.Y = 110
       Location.Mode = 1 'Walking
       Location.Dir = Left
       CALL DispAll
     END IF
 
  CASE IS = "TOWN2     "
     IF (X = 8) AND (Y = 5) THEN  'Armor Shop
        CALL Items(101, ShopItems(1).Item)
           ShopItems(1).Cost = 100
        CALL Items(102, ShopItems(2).Item)
           ShopItems(2).Cost = 1250
        CALL Items(111, ShopItems(3).Item)
           ShopItems(3).Cost = 750
        CALL Items(150, ShopItems(4).Item)
           ShopItems(4).Cost = 50
        CALL Items(151, ShopItems(5).Item)
           ShopItems(5).Cost = 200
        CALL Shop("A", 5)

   
     ELSEIF (X = 14) AND (Y = 7) THEN  'Inn
       CALL Inn(75)
    
     ELSEIF (X = 8) AND (Y = 8) THEN 'Weapon Shop
        CALL Items(201, ShopItems(1).Item)
           ShopItems(1).Cost = 200
        CALL Items(212, ShopItems(2).Item)
           ShopItems(2).Cost = 450
        CALL Shop("W", 2)
     
     ELSEIF (X = 6) AND (Y = 15) THEN 'Item Shop
        CALL Items(1, ShopItems(1).Item)
           ShopItems(1).Cost = 10
        CALL Items(2, ShopItems(2).Item)
           ShopItems(2).Cost = 80
        CALL Items(7, ShopItems(3).Item)
           ShopItems(3).Cost = 25
        CALL Items(9, ShopItems(4).Item)
           ShopItems(4).Cost = 50
        CALL Shop("I", 4)
   
     ELSEIF (X = 6) AND (Y = 9) THEN
       Location.X = 7
       Location.Y = 6
       Location.Mode = 1 'Walking
       Location.Dir = Right
       CALL DispAll
    
     ELSEIF (X = 6) AND (Y = 6) THEN
       Location.X = 6
       Location.Y = 10
       Location.Mode = 1 'Walking
       Location.Dir = Down
       CALL DispAll
    
     ELSE 'Exit Town
       CALL LoadMap("World")
       MapName$ = "WORLD"
       Location.X = 186
       Location.Y = 129
       Location.Mode = 1 'Walking
       Location.Dir = Left
       CALL DispAll
     END IF

  CASE IS = "TOWN3     "
     IF (X = 39) AND (Y = 17) THEN  'Armor Shop
        CALL Items(102, ShopItems(1).Item)
           ShopItems(1).Cost = 1150
        CALL Items(111, ShopItems(2).Item)
           ShopItems(2).Cost = 750
        CALL Items(112, ShopItems(3).Item)
           ShopItems(3).Cost = 3750
        CALL Items(113, ShopItems(4).Item)
           ShopItems(4).Cost = 1750
        CALL Items(152, ShopItems(5).Item)
           ShopItems(5).Cost = 500
        CALL Shop("A", 5)

  
     ELSEIF (X = 21) AND (Y = 31) THEN  'Inn
       CALL Inn(150)
   
     ELSEIF (X = 29) AND (Y = 17) THEN 'Weapon Shop
        CALL Items(214, ShopItems(1).Item)
           ShopItems(1).Cost = 500
        CALL Items(202, ShopItems(2).Item)
           ShopItems(2).Cost = 700
        CALL Items(213, ShopItems(3).Item)
           ShopItems(3).Cost = 1000
        CALL Items(208, ShopItems(4).Item)
           ShopItems(4).Cost = 1200
        CALL Shop("W", 4)
    
     ELSEIF (X = 34) AND (Y = 15) THEN 'Item Shop
        CALL Items(1, ShopItems(1).Item)
           ShopItems(1).Cost = 5
        CALL Items(2, ShopItems(2).Item)
           ShopItems(2).Cost = 80
        CALL Items(7, ShopItems(3).Item)
           ShopItems(3).Cost = 25
        CALL Items(9, ShopItems(4).Item)
           ShopItems(4).Cost = 50
        CALL Items(14, ShopItems(5).Item)
           ShopItems(5).Cost = 50
        CALL Shop("I", 5)
  
     ELSE 'Exit Town
       CALL LoadMap("World")
       MapName$ = "WORLD"
       Location.X = 169
       Location.Y = 76
       Location.Mode = 1 'Walking
       Location.Dir = Left
       CALL DispAll
     END IF

  CASE IS = "TOWN4     "
     IF (X = 7) AND (Y = 11) THEN  'Armor Shop
        CALL Items(103, ShopItems(1).Item)
           ShopItems(1).Cost = 11800
        CALL Items(152, ShopItems(2).Item)
           ShopItems(2).Cost = 500
        CALL Shop("A", 2)

     ELSEIF (X = 23) AND (Y = 26) THEN  'Inn
       CALL Inn(100)
  
     ELSEIF (X = 3) AND (Y = 11) THEN 'Weapon Shop
        CALL Items(215, ShopItems(1).Item)
           ShopItems(1).Cost = 2000
        CALL Items(203, ShopItems(2).Item)
           ShopItems(2).Cost = 1000
        CALL Shop("W", 2)
   
     ELSEIF (X = 10) AND (Y = 18) THEN 'Item Shop
        CALL Items(1, ShopItems(1).Item)
           ShopItems(1).Cost = 5
        CALL Items(2, ShopItems(2).Item)
           ShopItems(2).Cost = 75
        CALL Items(7, ShopItems(3).Item)
           ShopItems(3).Cost = 25
        CALL Items(9, ShopItems(4).Item)
           ShopItems(4).Cost = 50
        CALL Items(10, ShopItems(5).Item)
           ShopItems(5).Cost = 300
        CALL Items(14, ShopItems(6).Item)
           ShopItems(6).Cost = 50
        CALL Shop("I", 6)
 
     ELSE 'Exit Town
       MapName$ = "WORLD"
       Location.X = 166
       Location.Y = 38
       Location.Mode = 1 'Walking
       Location.Dir = Down
       CALL DispAll
     END IF



  CASE IS = "OLDMANH   "
     IF (X = 11) AND (Y = 11) THEN 'Back to Town
       Location.X = 27
       Location.Y = 29
       Location.Mode = 1 ' Walking
       Location.Dir = Down
       CALL LoadMap("TOWN1")
       MapName$ = "TOWN1"
       CALL DispAll
     END IF


'***   E N D   O F   T O W N S ***
'************ CAVES *******************
  CASE IS = "CAVE1     "
     IF (X = 10) AND (Y = 3) THEN  'Entrance
       CALL LoadMap("World")
       MapName$ = "WORLD"
       Location.X = 112
       Location.Y = 115
       Location.Mode = 1 'Walking
       Location.Dir = Down
       CALL DispAll
      ELSEIF (X = 5) AND (Y = 22) THEN 'Exit
       CALL LoadMap("World")
       MapName$ = "WORLD"
       Location.X = 110
       Location.Y = 118
       Location.Mode = 1 'Walking
       Location.Dir = Down
       CALL DispAll
     END IF
 
 
' ** **** *** * * WORLD * * *** **** **

  CASE IS = "WORLD     "
     IF (X = 131) AND (Y = 110) THEN  'Town 1
       Location.X = 18
       Location.Y = 31
       Location.Mode = 1 ' Walking
       Location.Dir = Up
       CALL LoadMap("TOWN1")
       MapName$ = "TOWN1"
       CALL DispAll
       Env.Songnum = 1
       Env.SongLoc = 1
       IF Music = True THEN CALL BGMPlay

    
     ELSEIF (X = 112) AND (Y = 114) THEN 'Cave 1 (front)
       Location.X = 10
       Location.Y = 4
       Location.Mode = 1 ' Walking
       Location.Dir = Down
       CALL LoadMap("CAVE1")
       MapName$ = "CAVE1"
       CALL DispAll
       
     ELSEIF (X = 110) AND (Y = 117) THEN 'Cave 1 (back)
       Location.X = 5
       Location.Y = 21
       Location.Mode = 1 ' Walking
       Location.Dir = Up
       CALL LoadMap("CAVE1")
       MapName$ = "CAVE1"
       CALL DispAll

     ELSEIF (X = 113) AND (Y = 120) THEN 'Time Shrine
     ELSEIF (X = 187) AND (Y = 129) THEN 'Town 2
     ELSEIF (X = 179) AND (Y = 90) THEN 'ForestMaze E
     ELSEIF (X = 173) AND (Y = 80) THEN 'ForestMaze X
     ELSEIF (X = 168) AND (Y = 76) THEN  'Town 3
     ELSEIF (X = 166) AND (Y = 76) THEN 'Town 3 Bridge
     ELSEIF (X = 155) AND (Y = 91) THEN 'Quick Sand Cave
     ELSEIF (X = 166) AND (Y = 37) THEN 'Town 4
     ELSEIF (X = 192) AND (Y = 48) THEN 'Desert Cave E
     ELSEIF (X = 207) AND (Y = 42) THEN 'Desert Cave X
     ELSEIF (X = 200) AND (Y = 57) THEN 'Desert Shrine
     ELSEIF (X = 133) AND (Y = 156) THEN 'Town 5
     ELSEIF (X = 88) AND (Y = 153) THEN 'IceCave1 E
     ELSEIF (X = 86) AND (Y = 166) THEN 'IceCave1 X
     ELSEIF (X = 82) AND (Y = 166) THEN 'IceCave2 E
     ELSEIF (X = 76) AND (Y = 164) THEN 'IceCave2 X
     ELSEIF (X = 69) AND (Y = 155) THEN 'Town 6
     ELSEIF (X = 77) AND (Y = 141) THEN 'IceCave3
     ELSEIF (X = 46) AND (Y = 158) THEN 'IceCave4 E
     ELSEIF (X = 44) AND (Y = 148) THEN 'IceCave4 X
     ELSEIF (X = 42) AND (Y = 151) THEN 'Ice Shrine
     ELSEIF (X = 55) AND (Y = 46) THEN 'Town 8
     ELSEIF (X = 64) AND (Y = 46) THEN 'Floating Island
     ELSEIF ((X = 72) OR (X = 73)) AND (Y = 96) THEN 'Castle
     ELSEIF (X = 114) AND (Y = 32) THEN 'Secret Cave
     ELSEIF (X = 130) AND (Y = 120) THEN 'Town 7
     ELSEIF (X = 168) AND (Y = 62) THEN 'Town 9
     END IF








     
   




END SELECT

       
END SUB

'Treasure Chest handler
'# greater than 500 is Gold
SUB TCHandler (TNum AS INTEGER)
ItemNum = 0
SELECT CASE TNum
  CASE 1 'First Treasure Chest
    ItemNum = 1000 'The item in the first chest (Town 2)

  CASE 2
    ItemNum = 9  'Cave 1  * Life 1
 
  CASE 3
    ItemNum = 1  'Cave 1  * Cure 1
 
  CASE 4
    ItemNum = 7  'Cave 1  * Multi Heal



END SELECT

IF ItemNum < 500 THEN  'If it is an item not gold
IF (ItemNum = 0) OR (Tchest(TNum) = False) THEN 'Emty Chest
  CALL ResetMessBox("Discovery")
  CALL MessBox("You find nothing!")
ELSE
  CALL Items(ItemNum, TempItem)
  CALL ResetMessBox("Discovery")
  CALL MessBox("You found a " + TempItem.Iname + "!")
  TotalInv = 0
  FOR T = 1 TO MaxInv   'Get total # of items in inv.
    IF PlayerInv(T).Item.Num <> 0 THEN TotalInv = TotalInv + 1
  NEXT T
  IF TotalInv = MaxInv THEN  'Inv Full
    CALL MessBox("Inventory Full!")
    
   ELSE 'Room left in inv
    PlayerInv(TotalInv).Amount = 1
    PlayerInv(TotalInv).Item = TempItem
    Tchest(TNum) = False
  END IF
END IF
ELSE  'Gold in Chest
  IF Tchest(TNum) = True THEN
    CALL ResetMessBox("Discovery")
    CALL MessBox("You find" + STR$(ItemNum) + " GP!")
    Player(1).Gold = Player(1).Gold + ItemNum
    Tchest(TNum) = False
   ELSE
    CALL ResetMessBox("Discovery")
    CALL MessBox("You find nothing!")
 END IF
END IF
CALL PAUSE
CALL DispAll
END SUB

SUB TestEventTriggered

Trigger = True
SELECT CASE Location.Dir
  CASE IS = Up
    IF Location.Y = 1 THEN
      Location.NY = Location.MapDimY
      Location.NX = Location.X
     ELSE
      Location.NY = Location.Y - 1
      Location.NX = Location.X
     END IF
           
  CASE IS = Down
    IF Location.Y = Location.MapDimY THEN
      Location.NY = 1
      Location.NX = Location.X
     ELSE
      Location.NY = Location.Y + 1
      Location.NX = Location.X
     END IF
            
  CASE IS = Right
    IF Location.X = Location.MapDimX THEN
      Location.NX = 1
      Location.NY = Location.Y
     ELSE
      Location.NX = Location.X + 1
      Location.NY = Location.Y
     END IF
          
  CASE IS = Left
    IF Location.X = 1 THEN
      Location.NX = Location.MapDimX
      Location.NY = Location.Y
     ELSE
      Location.NX = Location.X - 1
      Location.NY = Location.Y
     END IF
     
END SELECT
IF (Location.Mode = 1) AND (Location.X = Location.AirX) AND (Location.Y = Location.AirY) THEN
  Location.AirX = 0
  Location.AirY = 0
  Location.Mode = 3
  CALL DispScreen
  CALL DispAir(2, 105, 85, 20)
ELSEIF (Location.Mode = 3) AND (Map(Location.X, Location.Y, 2) = 1) THEN
  Location.AirX = Location.X
  Location.AirY = Location.Y
  Location.Mode = 1
  CALL DispScreen
  CALL DispAir(5, 105, 85, 20)
  CALL DispHero(1, 1, 105, 85, 20)

ELSEIF Map(Location.NX, Location.NY, 2) = 4 THEN
  SOUND 63, 3   'Event Triggered
  CALL EIHandler(Location.X, Location.Y, Location.MapName)
 ELSE
  SOUND 56, .5
END IF
  
'LOCATE 1, 31: PRINT "X:"; Location.X; " "
'LOCATE 2, 31: PRINT "Y:"; Location.Y; " "
'LOCATE 3, 31: PRINT "NX:"; Location.NX; " "
'LOCATE 4, 31: PRINT "NY:"; Location.NY; " "
'LOCATE 5, 31: PRINT "NA:"; Map(Location.NX, Location.NY, 2); " "

'GOSUB CheckDir

END SUB