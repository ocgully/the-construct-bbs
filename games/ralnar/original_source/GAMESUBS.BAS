DECLARE SUB DispHero (D AS INTEGER, T!, SX!, SY!, Amount AS INTEGER)
DECLARE SUB DispScreen ()
DECLARE SUB DispMainScreen ()
DECLARE SUB UseItem (ItemNum, PNum, UsedItem)
DECLARE SUB Items (Num AS INTEGER, Item AS ANY)
DECLARE SUB SwapItem (Item1 AS ANY, Item2 AS ANY)
DECLARE SUB BCPrint (Text$, FGC AS INTEGER, BGC AS INTEGER)
DECLARE SUB Options ()
DECLARE SUB PAUSE ()
DECLARE SUB GetKey (Option$)
DECLARE SUB PrintL (Text$, X%, Y%, Col%)

TYPE EnvironmentType
  Music AS INTEGER  'Music On or Off
  BGC AS INTEGER    'Backgound Color  1 = Blue
  FGC AS INTEGER    'Foreground Color 7 = Grey
  MS AS INTEGER 'Message Speed 1-9
  ML AS INTEGER 'Message Line (for displaying messages using TextFont)
  Icon AS INTEGER 'Player Icon
  LastB AS INTEGER 'Number of moves since the last battle
  SongNum AS INTEGER 'The Song Number for the music to play on
  SongLoc AS INTEGER 'The Line Location that the current song is on
END TYPE

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
  Cost AS LONG   'Price of Item (Default Price)
END TYPE

TYPE ShopItemType
  Cost AS LONG  'For Changing prices
  Item AS ItemType
END TYPE

TYPE InvType
  Amount AS INTEGER  'total # of that item
  Item AS ItemType
END TYPE

TYPE LocationType
  X AS INTEGER   'Map X Coordinate
  Y AS INTEGER   'Map Y Coordinate
  NX AS INTEGER  'When facing a directing, the next X Coordinate
  NY AS INTEGER  'When facing a directing, the next Y Coordinate
  MapName AS STRING * 10
  MapDimX AS INTEGER   'X dimensions of current map
  MapDimY AS INTEGER   'Y dimensions of current map
  Dir AS INTEGER  'Direction player is facing
  Mode AS INTEGER  'Land, ship, or Air Modes (1-3 respectively)
  ShipX AS INTEGER
  ShipY AS INTEGER
  ShipExist AS INTEGER
  AirX AS INTEGER
  AirY AS INTEGER
  AirExist AS INTEGER
  Enemies AS INTEGER    'Enemies on current map? (True/False)
  FullSeeSize AS INTEGER  'Number of pixels for icon attribute
  HalfSeeSize AS INTEGER  'Number of pixels for icon attribute
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
  MaxMP AS INTEGER
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

COMMON SHARED NumberMembers AS INTEGER, Player   AS PlayerType
COMMON SHARED Enemies AS INTEGER, Map   AS INTEGER
COMMON SHARED Env AS EnvironmentType, Location AS LocationType
COMMON SHARED MapIcons   AS INTEGER, Hero   AS INTEGER, Boat   AS INTEGER
COMMON SHARED Air   AS INTEGER, ShopItems   AS ShopItemType
COMMON SHARED PlayerInv   AS InvType, Special AS SpecialType
COMMON SHARED TChest   AS INTEGER

' * CONST

CONST PicPath$ = "C:\bg_rpg\pics\"
CONST True = 1
CONST False = 0
CONST MaxInv = 50  'Max number of items in the inventory
                   'Max inv also in GameSubs Module
'CONST DirRight = 1, DirDown = 2, DirLeft = 3, DirUp = 4
CONST ESC = 27, Down = 80, Up = 72, Left = 75, Right = 77, ENTER = 13


DEFSNG A-Z
SUB FileScreen
SCREEN 9: COLOR Env.FGC, Env.BGC: CLS
SelLoc = 1
GOSUB DispSelChoices
GOSUB UpdateScrBar
DoneMenu = 0
DO
  CALL GetKey(Key$)
  IF (Key$ = CHR$(13)) OR (Key$ = CHR$(0) + CHR$(77)) OR (Key$ = " ") THEN Key$ = MID$(STR$(SelLoc), 2, 1)
  SELECT CASE Key$
    CASE IS = "Q", "5", "B", CHR$(ESC)
      DoneMenu = True
      Key$ = CHR$(27)

    CASE IS = "L", "3"
      'CALL Load
      DoneMenu = True

    CASE IS = "S", "2"
      'CALL Save
      DoneMenu = True

    CASE IS = "O", "4"
      CALL Options
      COLOR Env.FGC, Env.BGC
      CLS
      GOSUB DispSelChoices
      GOSUB UpdateScrBar
 
    CASE IS = "H", "1", CHR$(0) + ";" 'Help/F1
      'CALL Help
      DoneMenu = DownMenu

    CASE IS = CHR$(0) + CHR$(Down) 'Down
      IF SelLoc = 5 THEN
          SelLoc = 1
        ELSE
          SelLoc = SelLoc + 1
      END IF
      GOSUB UpdateScrBar
   
    CASE IS = CHR$(0) + CHR$(Up) 'Up
      IF SelLoc = 1 THEN
          SelLoc = 5
        ELSE
          SelLoc = SelLoc - 1
      END IF
      GOSUB UpdateScrBar


  END SELECT


LOOP UNTIL (DoneMenu = True) OR (Key$ = CHR$(27)) 'ESC

SCREEN 13: CLS
EXIT SUB  'Leaves Sub

UpdateScrBar: 'Gosub Label
FOR T = 1 TO 5
LINE (45, 27 + (T * 28))-STEP(145, 15), Env.BGC, B
NEXT T
LINE (45, 27 + (SelLoc * 28))-STEP(145, 15), Env.FGC, B
GOSUB UpdateMessage:
RETURN

UpdateMessage:
SELECT CASE SelLoc
  CASE 1
     Message$ = "Goes to the help menu"
  CASE 2
     Message$ = "Goes to the save screen"
  CASE 3
     Message$ = "Goes to the load screen"
  CASE 4
     Message$ = "Goes to the options screen"
  CASE 5
     Message$ = "Exits back to game"
END SELECT
LOCATE 23, 1: PRINT STRING$(79, " ")
LOCATE 23, (40 - CINT(LEN(Message$) / 2)): PRINT Message$
RETURN

DispSelChoices:
CALL PrintL("Help", 10, 5, Env.FGC)
CALL PrintL("Save", 10, 7, Env.FGC)
CALL PrintL("Load", 10, 9, Env.FGC)
CALL PrintL("Options", 10, 11, Env.FGC)
CALL PrintL("Back to Game", 10, 13, Env.FGC)

RETURN

END SUB

SUB Inn (Price AS INTEGER)
SCREEN 9: CLS
COLOR Env.FGC, Env.BGC
YesNoOp = 1    'Location (1 [YES] or 2 [NO])
PAINT (1, 1), Env.BGC
LOCATE 1, 1: PRINT "Rooms are" + STR$(Price) + " GP a night, do you want to stay?"
LOCATE 3, 1: PRINT " YES   NO"
LINE (1 + (40 * (YesNoOp - 1)), 25)-STEP(40, 20), Env.FGC, B
DO
  CALL GetKey(Key$)
  SELECT CASE Key$
    CASE IS = CHR$(0) + CHR$(Right), CHR$(0) + CHR$(Left)
    LINE (1 + (40 * (YesNoOp - 1)), 25)-STEP(40, 20), Env.BGC, B
       IF YesNoOp = 2 THEN
         YesNoOp = 1
        ELSE
         YesNoOp = 2
       END IF
    LINE (1 + (40 * (YesNoOp - 1)), 25)-STEP(40, 20), Env.FGC, B
  END SELECT

LOOP UNTIL Key$ = CHR$(ENTER)

IF YesNoOp = 1 THEN   'Staying at the Inn
  IF Player(1).Gold >= Price THEN
    SCREEN 13: CLS
    SLEEP (3) 'pauses for 3 seconds
    Player(1).Gold = Player(1).Gold - Price
    FOR T = 1 TO NumberMembers
      Player(T).HP = Player(T).MaxHP
      Player(T).MP = Player(T).MaxMP
      Player(T).Status = 1  '1 = Okay, 0 = Dead
    NEXT T
   ELSE
    LOCATE 5, 1: PRINT "NOT ENOUGH GOLD!"
    SLEEP (3)
  END IF

END IF

CALL DispMainScreen
CALL DispScreen
CALL DispHero(Location.Dir, 4, XI + 105, YI + 85, Location.FullSeeSize)

END SUB

SUB Inventory
SCREEN 9
MenSel = 1
InvMode = 0 ' 1 = Equip, 2 = Using
BodLoc = 1 '1 = Left Hand, 2 = Right hand, 3= Body  ; for equiping
QuitInv = False
DIM LocItem AS ItemType  'Used in displaying changes in stats
DIM TempItem AS InvType
DIM TempEquip AS ItemType
GOSUB ShowInvScreen:
GOSUB UpdateInvMen:

DO
  CALL GetKey(Key$)
  SELECT CASE Key$
    CASE IS = CHR$(0) + CHR$(Right)
      IF MenSel < 4 THEN
        MenSel = MenSel + 1
       ELSE
        MenSel = 1
      END IF
      GOSUB UpdateInvMen
   
    CASE IS = CHR$(0) + CHR$(Left)
      IF MenSel > 1 THEN
        MenSel = MenSel - 1
       ELSE
        MenSel = 4
      END IF
      GOSUB UpdateInvMen

  CASE IS = "Q", CHR$(ESC)
    QuitInv = True

  CASE IS = "E"
    MenSel = 1
    GOSUB UpdateInvMen
    InvMode = 1  'Equip
    GOSUB DispInvStuff

  CASE IS = "S"
    MenSel = 3
    GOSUB UpdateInvMen
    GOSUB Special
 
  CASE IS = "U"
    MenSel = 2
    GOSUB UpdateInvMen
    InvMode = 2
    GOSUB DispInvStuff

  CASE IS = " ", CHR$(ENTER), CHR$(0) + CHR$(Down)
    SELECT CASE MenSel
      CASE 1
        InvMode = 1  'Equip
        GOSUB DispInvStuff
      CASE 2
        InvMode = 2  'Use
        GOSUB DispInvStuff
      CASE 3
        GOSUB Special
      CASE 4
        QuitInv = True
    END SELECT

  END SELECT

LOOP UNTIL QuitInv = True
'Add the following lines after debugging inv screen
'SCREEN 13: CLS
'CALL DispMainScreen
'CALL DispScreen
'CALL DispHero(Location.Dir, 4, XI + 105, YI + 85, Location.FullSeeSize)

EXIT SUB

ShowInvScreen:
  CLS
  PAINT (1, 1), Env.BGC  'paints the screen the back ground color
  COLOR Env.FGC, Env.BGC
  LOCATE 1, 36: PRINT "INVENTORY"
  LOCATE 2, 3: PRINT "Equip      Use      Special      Quit"
RETURN

UpdateInvMen:    'displays box around the current option
  FOR L = 1 TO 4
    FOR T = 1 TO 4
      LINE ((L * 80) - 77 + T, 13)-STEP(80, 15), Env.BGC, B
    NEXT
  NEXT L
  FOR T = 1 TO 4
    LINE ((MenSel * 80) - 77 + T, 13)-STEP(80, 15), Env.FGC, B
  NEXT
  SELECT CASE MenSel  'Displays the descriptions
    CASE 1
      CALL BCPrint("Equip weapons, armor, or special items", Env.FGC, Env.BGC)
    CASE 2
      CALL BCPrint("Use an item from your inventory", Env.FGC, Env.BGC)
    CASE 3
      CALL BCPrint("Organize your list of items", Env.FGC, Env.BGC)
    CASE 4
      CALL BCPrint("Returns to previous menu", Env.FGC, Env.BGC)
  END SELECT
RETURN

Equip:    'Equips item into BodLoc slot and removes current item
Equipable = False
I = 0
EW$ = PlayerInv(UseSel).Item.EquipWho
IT = PlayerInv(UseSel).Item.ItemType  'Item Type
DO
  I = I + 1
  IF (Player(PNum).PLet = MID$(EW$, I, 1)) THEN
    IF ((IT = 2) AND (BodLoc = 3)) OR ((IT = 3) AND (BodLoc < 3)) THEN
      Equipable = True
    END IF
  END IF
LOOP UNTIL (Equipable = True) OR (I = 4)
IF (Equipable = True) THEN
 IT = PlayerInv(UseSel).Item.ItemType  'Item Type [shrink length of var]
 TotalInv = 1
 WHILE (TotalInv < MaxInv) AND (PlayerInv(TotalInv).Amount <> 0)
  TotalInv = TotalInv + 1
 WEND
  IF (TotalInv = MaxInv) AND (PlayerInv(TotalInv).Amount <> 0) THEN
    Full = True
   ELSE
    Full = False
  END IF
 IF ((IT = 2) AND (BodLoc = 3)) OR ((IT = 3) AND (BodLoc < 3)) THEN
   IF BodLoc = 1 THEN
     LocItem = Player(PNum).LeftArm
    ELSEIF BodLoc = 2 THEN
     LocItem = Player(PNum).RightArm
    ELSEIF BodLoc = 3 THEN
     LocItem = Player(PNum).Armor
   END IF
   IF (LocItem.ItemType = 0) THEN
     IF BodLoc = 1 THEN
       Player(PNum).LeftArm = PlayerInv(UseSel).Item
      ELSEIF BodLoc = 2 THEN
       Player(PNum).RightArm = PlayerInv(UseSel).Item
      ELSEIF BodLoc = 3 THEN
       Player(PNum).Armor = PlayerInv(UseSel).Item
     END IF
     Player(PNum).Strength = Player(PNum).Strength + PlayerInv(UseSel).Item.StrengthEffect
     Player(PNum).Speed = Player(PNum).Speed + PlayerInv(UseSel).Item.SpeedEffect
     Player(PNum).BaseDefense = Player(PNum).BaseDefense + PlayerInv(UseSel).Item.DefenseEffect
     GOSUB Drop: 'Deletes equiped item from inventory
     GOSUB DispPName:        'Refreshes Screen to display change
     GOSUB UpdateInvList:    '
  
    ELSEIF ((Full = True) AND (PlayerInv(UseSel).Amount = 1)) OR (Full = False) THEN
     IF (LocItem.Num <> PlayerInv(UseSel).Item.Num) THEN
      IF BodLoc = 1 THEN
        TempEquip = Player(PNum).LeftArm
        Player(PNum).LeftArm = PlayerInv(UseSel).Item
       ELSEIF BodLoc = 2 THEN
        TempEquip = Player(PNum).RightArm
        Player(PNum).RightArm = PlayerInv(UseSel).Item
       ELSEIF BodLoc = 3 THEN
        TempEquip = Player(PNum).Armor
        Player(PNum).Armor = PlayerInv(UseSel).Item
      END IF
       Player(PNum).Strength = Player(PNum).Strength + PlayerInv(UseSel).Item.StrengthEffect - TempEquip.StrengthEffect
       Player(PNum).Speed = Player(PNum).Speed + PlayerInv(UseSel).Item.SpeedEffect - TempEquip.SpeedEffect
       Player(PNum).BaseDefense = Player(PNum).BaseDefense + PlayerInv(UseSel).Item.DefenseEffect - TempEquip.DefenseEffect
      GOSUB Drop: 'Deletes equiped item from inventory
       'Finds the new inventory size (may have changed)
       TotalInv = 1
       WHILE (TotalInv < MaxInv - 1) AND (PlayerInv(TotalInv).Amount <> 0)
         TotalInv = TotalInv + 1
       WEND
       IF PlayerInv(TotalInv).Amount = 0 THEN
         Diff = 0
        ELSE
         Diff = 1
       END IF
       PlayerInv(TotalInv + Diff).Item = TempEquip
       PlayerInv(TotalInv + Diff).Amount = 1
      GOSUB DispPName:        'Refreshes Screen to display change
      GOSUB UpdateInvList:    '
     END IF
    ELSEIF (Full = True) AND (PlayerInv(TotalInv).Amount > 1) THEN
      CALL BCPrint("Inventory is full, drop item before equiping", Env.FGC, Env.BGC)
   END IF
 END IF
END IF
   



  
RETURN

Special:
RETURN

Sort:
 'Make as many amounts 99 as possible

 FOR T = 1 TO MaxInv
   IF (PlayerInv(T).Amount < 99) AND (PlayerInv(T).Amount > 0) THEN
     IF T + 1 <= MaxInv THEN
       I = T + 1
       WHILE (I <= MaxInv) AND (PlayerInv(T).Amount < 99)
         IF PlayerInv(I).Item.Num = PlayerInv(T).Item.Num THEN
           ANeeded = 99 - PlayerInv(T).Amount
           Have = PlayerInv(I).Amount
           IF ANeeded >= Have THEN
             PlayerInv(T).Amount = PlayerInv(T).Amount + Have
             PlayerInv(I).Amount = 0
             CALL Items(0, PlayerInv(I).Item)
            ELSEIF Have > ANeeded THEN
              PlayerInv(T).Amount = PlayerInv(T).Amount + ANeeded
              PlayerInv(I).Amount = PlayerInv(I).Amount - ANeeded
           END IF
         END IF
         I = I + 1
       WEND
     END IF
   END IF
 NEXT T


'Next eliminate all spaces
TotalItems = 0  'How many items are in the inventory
FOR T = 1 TO MaxInv
 IF PlayerInv(T).Amount > 0 THEN TotalItems = TotalItems + 1
NEXT T
 FOR T = 1 TO TotalItems
   WHILE PlayerInv(T).Amount = 0
     IF PlayerInv(T).Amount = 0 THEN
       FOR I = T + 1 TO MaxInv
         PlayerInv(I - 1) = PlayerInv(I)
         CALL Items(0, PlayerInv(I).Item)
         PlayerInv(I).Amount = 0
       NEXT I
     END IF
   WEND
 NEXT T

'Then organize them by ItemTypes
'Using Bubble Sort

FOR I = 1 TO TotalItems - 1
  FOR J = I + 1 TO TotalItems
    IF PlayerInv(J).Item.Num < PlayerInv(I).Item.Num THEN
      'Swap two items
      TempItem = PlayerInv(J)
      PlayerInv(J) = PlayerInv(I)
      PlayerInv(I) = TempItem
    END IF
  NEXT J
NEXT I


RETURN

Drop:
IF PlayerInv(UseSel).Amount > 0 THEN
  PlayerInv(UseSel).Amount = PlayerInv(UseSel).Amount - 1
END IF
IF (PlayerInv(UseSel).Amount = 0) AND (UseSel < MaxInv) THEN
  FOR X = UseSel TO MaxInv - 1
    CALL SwapItem(PlayerInv(X).Item, PlayerInv(X + 1).Item)
    PlayerInv(X).Amount = PlayerInv(X + 1).Amount
  NEXT X
  CALL Items(0, PlayerInv(MaxInv).Item)
  PlayerInv(MaxInv).Amount = 0
 ELSEIF (PlayerInv(UseSel).Amount = 0) AND (UseSel = MaxInv) THEN
  CALL Items(0, PlayerInv(UseSel).Item)
  PlayerInv(UseSel).Amount = 0
END IF
GOSUB UpdateInvList:

RETURN

RemoveItem: 'Takes weapons/armors off
TotalItems = 0  'How many items are in the inventory
FOR T = 1 TO MaxInv
 IF PlayerInv(T).Amount > 0 THEN TotalItems = TotalItems + 1
NEXT T
IF (TotalItems < MaxInv) THEN
  CALL Items(0, TempEquip)
  IF BodLoc = 1 THEN
    IF Player(PNum).LeftArm.Num > 0 THEN
      TempEquip = Player(PNum).LeftArm
      PlayerInv(TotalItems + 1).Item = Player(PNum).LeftArm
      PlayerInv(TotalItems + 1).Amount = 1
      CALL Items(0, Player(PNum).LeftArm)
     ELSE
      CALL BCPrint("No Weapon Equiped", Env.FGC, Env.BGC)
    END IF
   ELSEIF BodLoc = 2 THEN
    IF Player(PNum).RightArm.Num > 0 THEN
      TempEquip = Player(PNum).RightArm
      PlayerInv(TotalItems + 1).Item = Player(PNum).RightArm
      PlayerInv(TotalItems + 1).Amount = 1
      CALL Items(0, Player(PNum).RightArm)
     ELSE
      CALL BCPrint("No Weapon Equiped", Env.FGC, Env.BGC)
    END IF
   ELSEIF BodLoc = 3 THEN
    IF Player(PNum).Armor.Num > 0 THEN
      TempEquip = Player(PNum).Armor
      PlayerInv(TotalItems + 1).Item = Player(PNum).Armor
      PlayerInv(TotalItems + 1).Amount = 1
      CALL Items(0, Player(PNum).Armor)
     ELSE
      CALL BCPrint("No Weapon Equiped", Env.FGC, Env.BGC)
    END IF
  END IF
  Player(PNum).Strength = Player(PNum).Strength - TempEquip.StrengthEffect
  Player(PNum).Speed = Player(PNum).Speed - TempEquip.SpeedEffect
  Player(PNum).BaseDefense = Player(PNum).BaseDefense - TempEquip.DefenseEffect
  GOSUB DispPName
  GOSUB UpdateInvList:
  GOSUB DispItemIcons:
 ELSEIF TotalItems = MaxInv THEN
  CALL BCPrint("Inventory Full, must make room", Env.FGC, Env.BGC)
END IF

RETURN

Use: 'Using Items
  CALL UseItem(UseSel, PNum, UsedItem)
  IF UsedItem = True THEN
    GOSUB Drop:
  END IF
RETURN

DispInvStuff:
  UseSel = 1
  'InvMode = 2
  PNum = 1 'Player Number
  Arrow = UseSel
  LOCATE 10 + Arrow, 3: PRINT "¯"
  LOCATE 10 + Arrow, 38: PRINT "®"
  LOCATE 10, 2: PRINT "ÄÄÄÄÄ Item ÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄ Amount Ä"
  LINE (7, 120)-STEP(304, 165), Env.FGC, B
  'LINE (265, 32)-STEP(92 * 4 + 5, 33), Env.FGC, B
  GOSUB DispPName
  GOSUB UpdateInvList:
  GOSUB DispItemIcons:

  DO
    CALL GetKey(Key$)
    SELECT CASE Key$
     
      CASE IS = " "
        IF InvMode = 1 THEN
          GOSUB Equip
         ELSEIF InvMode = 2 THEN
          GOSUB Use
        END IF

      CASE IS = CHR$(0) + CHR$(Down)
        IF Arrow < 10 THEN
          Arrow = Arrow + 1
          UseSel = UseSel + 1
          GOSUB UpdateInvList:
         ELSEIF (Arrow = 10) AND (UseSel < MaxInv) THEN
          UseSel = UseSel + 1
          GOSUB UpdateInvList:
        END IF
         
      CASE IS = CHR$(0) + CHR$(Up)
        IF Arrow > 1 THEN
          Arrow = Arrow - 1
          UseSel = UseSel - 1
          GOSUB UpdateInvList:
         ELSEIF (Arrow = 1) AND (UseSel > 1) THEN
          UseSel = UseSel - 1
          GOSUB UpdateInvList:
        END IF
     
      CASE IS = "D"
        GOSUB Drop:

      CASE IS = CHR$(0) + CHR$(Right)
        IF PNum < NumberMembers THEN
          PNum = PNum + 1
         ELSE
          PNum = 1
        END IF
        GOSUB DispPName:
        GOSUB ShowInvStat:
        GOSUB UpdateInvList:


      CASE IS = CHR$(0) + CHR$(Left)
        IF PNum > 1 THEN
          PNum = PNum - 1
         ELSE
          PNum = NumberMembers
        END IF
        GOSUB DispPName:
        GOSUB ShowInvStat:
        GOSUB UpdateInvList:

      CASE IS = "L", "R", "B"
         IF Key$ = "L" THEN
           BodLoc = 1
          ELSEIF Key$ = "R" THEN
           BodLoc = 2
          ELSEIF Key$ = "B" THEN
           BodLoc = 3
         END IF
         GOSUB DispPName:
         GOSUB UpdateInvList:
         GOSUB ShowInvStat:
     
      CASE IS = "O"
       IF InvMode = 1 THEN
        GOSUB RemoveItem:
       END IF
     
      CASE IS = "S"
        GOSUB Sort
        GOSUB Sort  'Twice to eliminate any problems, and for assurance
        UseSel = 1
        Arrow = 1
        PNum = 1
        GOSUB DispPName:
        GOSUB ShowInvStat:
        GOSUB UpdateInvList:


    END SELECT
  LOOP UNTIL (Key$ = CHR$(ESC))
  LINE (1, 50)-STEP(620, 280), Env.BGC, BF
  LINE (265, 32)-STEP(92 * 4 + 5, 33), Env.BGC, BF
  InvMode = 0
  GOSUB UpdateInvMen:

RETURN

DispPName:
FOR T = 1 TO NumberMembers
  LOCATE 4, (T * 11) + 25
  PRINT Player(T).PName
  LINE (180 + (T * 90), 38)-STEP(90, 20), Env.BGC, B
NEXT T
  LINE (180 + (PNum * 90), 38)-STEP(90, 20), Env.FGC, B
IF InvMode = 1 THEN
  LOCATE 6, 36: PRINT "L-Arm:"
  LOCATE 7, 36: PRINT "R-Arm:"
  LOCATE 8, 36: PRINT "Body :"
  FOR T = 1 TO 3
     LOCATE 5 + T, 34:
     IF T = BodLoc THEN
       PRINT "*"
      ELSE
       PRINT " "
     END IF
  NEXT T
  LOCATE 6, 43: PRINT Player(PNum).LeftArm.Iname
  LOCATE 7, 43: PRINT Player(PNum).RightArm.Iname
  LOCATE 8, 43: PRINT Player(PNum).Armor.Iname
END IF
 
RETURN

UpdateInvList:
        FOR T = 1 TO 10
          LOCATE 10 + T, 3: PRINT " "
          LOCATE 10 + T, 38: PRINT " "
        NEXT T
        LOCATE 10 + Arrow, 3: PRINT "¯"
        LOCATE 10 + Arrow, 38: PRINT "®"
 
  IF Arrow = 1 THEN
    T1 = UseSel
    T2 = T1 + 9
  ELSEIF Arrow = 10 THEN
    T2 = UseSel
    T1 = T2 - 9
  ELSE
    T1 = UseSel - Arrow + 1
    T2 = T1 + 9
  END IF
  Counter = 0
  FOR T = T1 TO T2
    Counter = Counter + 1
    LOCATE 10 + Counter, 9
      PRINT PlayerInv(T).Item.Iname
    LOCATE 10 + Counter, 34
    IF PlayerInv(T).Amount > 0 THEN
       PRINT PlayerInv(T).Amount
      ELSE
       PRINT "   "
    END IF
    IF PlayerInv(T).Item.ItemType = 1 THEN 'Item
      LOCATE 10 + Counter, 7
      PRINT ""
      LOCATE 10 + Counter, 5
      IF InvMode = 2 THEN
        PRINT ""
       ELSE
        PRINT " "
      END IF
     
     ELSEIF PlayerInv(T).Item.ItemType = 2 THEN 'Armor
      LOCATE 10 + Counter, 7
      PRINT "’"
      IF InvMode = 1 THEN
        GOSUB DispEquipStar:
       ELSE
        LOCATE 10 + Counter, 5
        PRINT " "
      END IF

     ELSEIF PlayerInv(T).Item.ItemType = 3 THEN 'Weapons
      LOCATE 10 + Counter, 7
      PRINT "î"
      IF InvMode = 1 THEN
        GOSUB DispEquipStar:
       ELSE
        LOCATE 10 + Counter, 5
        PRINT " "
      END IF

     ELSE
      LOCATE 10 + Counter, 7
      PRINT " "
      LOCATE 10 + Counter, 5
      PRINT " "
    END IF
  NEXT T
  GOSUB ShowInvStat:
  IF PlayerInv(UseSel).Amount > 0 THEN
    CALL BCPrint(PlayerInv(UseSel).Item.Desc, Env.FGC, Env.BGC)
   ELSE
    CALL BCPrint("Empty Inventory Location #" + STR$(UseSel), Env.FGC, Env.BGC)
  END IF
  
RETURN

ShowInvStat:
IT = PlayerInv(UseSel).Item.ItemType
IF ((InvMode = 2) AND (IT = 1)) OR ((InvMode = 1) AND (((IT = 2) AND (BodLoc = 3)) OR ((IT = 3) AND (BodLoc < 3)))) THEN
 
  FOR T = 12 TO 16
    LOCATE T, 60: PRINT "    "
    LOCATE T, 67: PRINT "    "
  NEXT T
 
  LOCATE 10, 50: PRINT "  Player: "; Player(PNum).PName
  LOCATE 12, 50: PRINT "      HP:"; Player(PNum).HP
  LOCATE 13, 50: PRINT "      MP:"; Player(PNum).MP
  LOCATE 14, 50: PRINT " Defense:"; Player(PNum).BaseDefense
  LOCATE 15, 50: PRINT "Strength:"; Player(PNum).Strength
  LOCATE 16, 50: PRINT "   Speed:"; Player(PNum).Speed
  MP = Player(PNum).MP
  HP = Player(PNum).HP
  BD = Player(PNum).BaseDefense
  St = Player(PNum).Strength
  SD = Player(PNum).Speed


IF PlayerInv(UseSel).Item.ItemType = 1 THEN
  St$ = MID$(PlayerInv(UseSel).Item.StatusEffect, 1, 1)
  IF (St$ = "N") OR (St$ = "Z") THEN'If It's not for battle use only
    
   
    IF (PlayerInv(UseSel).Item.HPEffect <> 0) AND (Player(PNum).HP <> 0) AND (Player(PNum).HP <> Player(PNum).MaxHP) THEN
      IF Player(PNum).HP + PlayerInv(UseSel).Item.HPEffect <= Player(PNum).MaxHP THEN
          HP = Player(PNum).HP + PlayerInv(UseSel).Item.HPEffect
        ELSE
          HP = Player(PNum).MaxHP
      END IF
    END IF 'HP effect

    IF (PlayerInv(UseSel).Item.MPEffect <> 0) AND (Player(PNum).HP <> 0) THEN
      IF Player(PNum).MP + PlayerInv(UseSel).Item.MPEffect <= Player(PNum).MaxMP THEN
          MP = Player(PNum).MP + PlayerInv(UseSel).Item.MPEffect
        ELSE
          MP = Player(PNum).MaxMP
      END IF
    END IF 'MP effect
   
    IF (PlayerInv(UseSel).Item.StrengthEffect <> 0) AND (Player(PNum).HP <> 0) THEN
      St = Player(PNum).Strength + PlayerInv(UseSel).Item.StrengthEffect
    END IF 'Strength effect

    IF (PlayerInv(UseSel).Item.SpeedEffect <> 0) AND (Player(PNum).HP <> 0) THEN
      SD = Player(PNum).Speed + PlayerInv(UseSel).Item.SpeedEffect
    END IF 'Speed effect

    IF (PlayerInv(UseSel).Item.DefenseEffect <> 0) AND (Player(PNum).HP <> 0) THEN
      BD = Player(PNum).BaseDefense + PlayerInv(UseSel).Item.DefenseEffect
    END IF 'Defense effect

  END IF
 
  IF (St$ = "D") AND (Player(PNum).HP = 0) THEN  'IF The Player is Dead
      IF Player(PNum).HP + PlayerInv(UseSel).Item.HPEffect <= Player(PNum).MaxHP THEN
          HP = Player(PNum).HP + PlayerInv(UseSel).Item.HPEffect
        ELSE
          HP = Player(PNum).MaxHP
      END IF
  END IF

ELSEIF InvMode = 1 THEN

    IF BodLoc = 1 THEN
      LocItem = Player(PNum).LeftArm
     ELSEIF BodLoc = 2 THEN
      LocItem = Player(PNum).RightArm
     ELSEIF BodLoc = 3 THEN
      LocItem = Player(PNum).Armor
    END IF
     
    St = Player(PNum).Strength + PlayerInv(UseSel).Item.StrengthEffect - LocItem.StrengthEffect
    SD = Player(PNum).Speed + PlayerInv(UseSel).Item.SpeedEffect - LocItem.SpeedEffect
    BD = Player(PNum).BaseDefense + PlayerInv(UseSel).Item.DefenseEffect - LocItem.DefenseEffect

END IF

  LOCATE 12, 65: PRINT ""; HP
  LOCATE 13, 65: PRINT ""; MP
  LOCATE 14, 65: PRINT ""; BD
  LOCATE 15, 65: PRINT ""; St
  LOCATE 16, 65: PRINT ""; SD

  FOR T = 12 TO 16
    LOCATE T, 72: PRINT " "
  NEXT T

  IF MP > Player(PNum).MP THEN
    LOCATE 13, 72: PRINT ""
  ELSEIF MP < Player(PNum).MP THEN
    LOCATE 13, 72: PRINT ""
  END IF
 
  IF HP > Player(PNum).HP THEN
    LOCATE 12, 72: PRINT ""
  ELSEIF HP < Player(PNum).HP THEN
    LOCATE 12, 72: PRINT ""
  END IF
 
  IF BD > Player(PNum).BaseDefense THEN
    LOCATE 14, 72: PRINT ""
  ELSEIF BD < Player(PNum).BaseDefense THEN
    LOCATE 14, 72: PRINT ""
  END IF
 
  IF St > Player(PNum).Strength THEN
    LOCATE 15, 72: PRINT ""
  ELSEIF St < Player(PNum).Strength THEN
    LOCATE 15, 72: PRINT ""
  END IF
 
  IF SD > Player(PNum).Speed THEN
    LOCATE 16, 72: PRINT ""
  ELSEIF SD < Player(PNum).Speed THEN
    LOCATE 16, 72: PRINT ""
  END IF

ELSE 'Inital If Statement
 'Draws Box around stat area
LINE (340, 110)-STEP(260, 120), Env.BGC, BF


END IF 'Inital If Statement
RETURN

DispItemIcons:
LOCATE 18, 60: PRINT " = Item"
LOCATE 19, 60: PRINT "’ = Armor"
LOCATE 20, 60: PRINT "î = Weapon"
LOCATE 22, 4: PRINT " Denotes usable \ equipable items"
RETURN

DispEquipStar:
'
Equipable = False
I = 0
EW$ = PlayerInv(T).Item.EquipWho
IT = PlayerInv(T).Item.ItemType  'Item Type
DO
  I = I + 1
  IF (Player(PNum).PLet = MID$(EW$, I, 1)) THEN
    IF ((IT = 2) AND (BodLoc = 3)) OR ((IT = 3) AND (BodLoc < 3)) THEN
      Equipable = True
    END IF
  END IF
LOOP UNTIL (Equipable = True) OR (I = 4)

LOCATE 10 + Counter, 5
IF Equipable = True THEN
  PRINT ""
 ELSE
  PRINT " "
END IF
RETURN

END SUB

SUB LoadPlayerImages
Ext$ = "FLBR"
Num$ = "2131"

FOR D = 1 TO 4
  FOR T = 1 TO 4
    TName$ = "Hero" + MID$(Ext$, D, 1) + MID$(Num$, T, 1) + ".PIC"
    OPEN PicPath$ + TName$ FOR INPUT AS #1
    FOR X = 1 TO 20
      FOR Y = 1 TO 20
        INPUT #1,Hero(D, T, X, Y)
      NEXT Y
    NEXT X
    CLOSE #1
  NEXT T
NEXT D

Num$ = "21"

FOR D = 1 TO 4
  FOR T = 1 TO 2
    TName$ = "Ship" + MID$(Ext$, D, 1) + MID$(Num$, T, 1) + ".PIC"
    OPEN PicPath$ + TName$ FOR INPUT AS #1
    FOR X = 1 TO 20
      FOR Y = 1 TO 20
        INPUT #1,Boat(D, T, X, Y)
      NEXT Y
    NEXT X
    CLOSE #1
  NEXT T
NEXT D

FOR D = 1 TO 4
    TName$ = "Air" + MID$(Ext$, D, 1) + ".PIC"
    OPEN PicPath$ + TName$ FOR INPUT AS #1
    FOR X = 1 TO 20
      FOR Y = 1 TO 20
        INPUT #1,Air(D, X, Y)
      NEXT Y
    NEXT X
    CLOSE #1
NEXT D
    OPEN PicPath$ + "Air.PIC" FOR INPUT AS #1
    FOR X = 1 TO 20
      FOR Y = 1 TO 20
        INPUT #1,Air(5, X, Y)
      NEXT Y
    NEXT X
    CLOSE #1


SCREEN 13

END SUB

'(ShopName$) is a single character
SUB Shop (ShopName$, TotalItems AS INTEGER)
SCREEN 9: CLS
PAINT (1, 1), Env.BGC  'paints the screen the back ground color
PNum = 1
COLOR Env.FGC, Env.BGC

ShopName$ = UCASE$(ShopName$)
GOSUB ResetShopScreen
DO
  CALL GetKey(Key$)
  SELECT CASE Key$
    CASE IS = "Q", CHR$(ESC)
      Selection = 3

    CASE IS = "B"
      GOSUB Buy:

    CASE IS = "S"
      GOSUB Sell:

    CASE IS = " ", CHR$(ENTER)
      Selection = ChoiceNum
      IF Selection = 1 THEN
        GOSUB Buy:
       ELSEIF Selection = 2 THEN
        GOSUB Sell:
      END IF


    CASE IS = CHR$(0) + CHR$(Up)
      IF ChoiceNum > 1 THEN
        ChoiceNum = ChoiceNum - 1
       ELSE
        ChoiceNum = 3
      END IF
      GOSUB UpdateSel

    CASE IS = CHR$(0) + CHR$(Down)
      IF ChoiceNum < 3 THEN
        ChoiceNum = ChoiceNum + 1
       ELSE
        ChoiceNum = 1
      END IF
      GOSUB UpdateSel

  END SELECT

LOOP UNTIL (Selection = 3)

SCREEN 13: CLS
CALL DispMainScreen
CALL DispScreen
CALL DispHero(Location.Dir, 4, XI + 105, YI + 85, Location.FullSeeSize)

EXIT SUB


UpdateSel:
COLOR Env.FGC, Env.BGC
LOCATE 2, 5: PRINT "Buy"
LOCATE 3, 5: PRINT "Sell"
LOCATE 4, 5: PRINT "Exit"
IF (Env.BGC <> 15) AND (Env.FGC <> 15) THEN
   COLOR 15, Env.BGC
  ELSEIF (Env.FGC + 1 <> Env.BGC) AND (Env.FGC < 15) THEN
   COLOR Env.FGC + 1, Env.BGC
END IF
   
LOCATE (1 + ChoiceNum), 5
SELECT CASE ChoiceNum
  CASE 1
    PRINT "Buy"
  CASE 2
    PRINT "Sell"
  CASE 3
    PRINT "Exit"
END SELECT
COLOR Env.FGC, Env.BGC
RETURN

Buy:
  LOCATE 8, 18: PRINT "Quantity: 1"
  LOCATE 10, 6: PRINT "Item"
  LOCATE 10, 40: PRINT "Price"
  LINE (12, 145)-(358, 300), Env.FGC, B
  FOR T = 1 TO TotalItems
    LOCATE 11 + T, 5: PRINT ShopItems(T).Item.Iname
    LOCATE 11 + T, 38
    IF ShopItems(T).Cost = 0 THEN
      PRINT ShopItems(T).Item.Cost
     ELSE
      PRINT ShopItems(T).Cost
    END IF
  NEXT T
  LOCATE 12, 3: PRINT "¯"
  GOSUB DispShopPName:
  GOSUB ShowShopStats:
  CALL BCPrint(ShopItems(ItSel).Item.Desc, Env.FGC, Env.BGC)
  DO
    CALL GetKey(Key$)
    SELECT CASE Key$
      CASE IS = "+"  ' Add's one to the quantity
        IF Qty < 99 THEN
          Qty = Qty + 1
         ELSE
          Qty = 1
        END IF
        LOCATE 8, 18: PRINT "Quantity:"; Qty; "  "
        GOSUB ShowShopStats

      CASE IS = "-"  'Subtracts 1 from the quantity
        IF Qty > 1 THEN
          Qty = Qty - 1
         ELSE
          Qty = 99
        END IF
        LOCATE 8, 18: PRINT "Quantity:"; Qty; "  "
        GOSUB ShowShopStats

      CASE IS = CHR$(0) + CHR$(Up)
        IF ItSel > 1 THEN
          ItSel = ItSel - 1
         ELSE
          ItSel = TotalItems
        END IF
        COLOR Env.BGC
        FOR T = 1 TO TotalItems
          LOCATE 11 + T, 3: PRINT " "
        NEXT T
        COLOR Env.FGC
        LOCATE ItSel + 11, 3: PRINT "¯"
        GOSUB ShowShopStats

        
      CASE IS = CHR$(0) + CHR$(Down)
        IF ItSel < TotalItems THEN
          ItSel = ItSel + 1
         ELSE
          ItSel = 1
        END IF
       
        COLOR Env.BGC
        FOR T = 1 TO TotalItems
          LOCATE 11 + T, 3: PRINT " "
        NEXT T
        COLOR Env.FGC, Env.BGC
        LOCATE ItSel + 11, 3: PRINT "¯"
        GOSUB ShowShopStats

      CASE IS = CHR$(0) + CHR$(Right)
        IF PNum < NumberMembers THEN
          PNum = PNum + 1
         ELSE
          PNum = 1
        END IF
        GOSUB DispShopPName:
        GOSUB ShowShopStats:


      CASE IS = CHR$(0) + CHR$(Left)
        IF PNum > 1 THEN
          PNum = PNum - 1
         ELSE
          PNum = NumberMembers
        END IF
        GOSUB DispShopPName:
        GOSUB ShowShopStats:
     
      CASE IS = " ", CHR$(ENTER)
        GOSUB PurchaseItem:


    END SELECT

  LOOP UNTIL (Key$ = CHR$(ESC))
  GOSUB ResetShopScreen:
RETURN

Sell:
  LOCATE 10, 2: PRINT "ÄÄÄÄÄ Item ÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄ Amount ÄÄÄÄ Price ÄÄÄÄÄÄ"
  LOCATE 8, 10: PRINT "Quantity: "
  LOCATE 8, 30: PRINT "Total Price: "
  LINE (7, 120)-STEP(432, 165), Env.FGC, B
  Arrow = 1 'Sets arrow locations to top spot
  UseSel = 1 'Sets Inventory location to first spot
  IF PlayerInv(UseSel).Amount > 0 THEN
    Qty = 1
   ELSE
    Qty = 0
  END IF
  LOCATE 6, 50: PRINT "Gold: "; Player(1).Gold
  GOSUB UpdateSellList:
  DO
    CALL GetKey(Key$)  'Gets input from Keyboard
    SELECT CASE Key$
      CASE IS = " ", CHR$(ENTER) 'Sell Item
        GOSUB SellItem:

      CASE IS = CHR$(0) + CHR$(Down) 'Scroll Down an Item
        IF Arrow < 10 THEN
          Arrow = Arrow + 1
          UseSel = UseSel + 1
          GOSUB ChangeQty:     'Makes Qty 1 or 0  for next Inv Location
          GOSUB UpdateSellList:
         ELSEIF (Arrow = 10) AND (UseSel < MaxInv) THEN
          UseSel = UseSel + 1
          GOSUB ChangeQty:     'Makes Qty 1 or 0  for next Inv Location
          GOSUB UpdateSellList:
        END IF
        
      CASE IS = CHR$(0) + CHR$(Up) 'Scroll Up an Item
        IF Arrow > 1 THEN
          Arrow = Arrow - 1
          UseSel = UseSel - 1
          GOSUB ChangeQty:     'Makes Qty 1 or 0  for next Inv Location
          GOSUB UpdateSellList:
         ELSEIF (Arrow = 1) AND (UseSel > 1) THEN
          UseSel = UseSel - 1
          GOSUB ChangeQty: 'Makes Qty 1 or 0  for next Inv Location
          GOSUB UpdateSellList:
        END IF
    
      CASE IS = CHR$(0) + CHR$(Right), "+" 'Quantity Up
        IF Qty < PlayerInv(UseSel).Amount THEN
          Qty = Qty + 1
          GOSUB DispQty:
        END IF

      CASE IS = CHR$(0) + CHR$(Left), "-" 'Quantity Down
        IF Qty > 1 THEN
          Qty = Qty - 1
          GOSUB DispQty:
        END IF

      CASE IS = "A"
        Qty = PlayerInv(UseSel).Amount
        GOSUB DispQty:

    END SELECT

  LOOP UNTIL (Key$ = CHR$(ESC)) OR (Key$ = "Q")
  GOSUB ResetShopScreen:
RETURN

UpdateSellList:

  FOR T = 1 TO 10
   LOCATE 10 + T, 3: PRINT " "
   LOCATE 10 + T, 54: PRINT " "
  NEXT T
   LOCATE 10 + Arrow, 3: PRINT "¯"
   LOCATE 10 + Arrow, 54: PRINT "®"
 
  IF Arrow = 1 THEN
    T1 = UseSel
    T2 = T1 + 9
  ELSEIF Arrow = 10 THEN
    T2 = UseSel
    T1 = T2 - 9
  ELSE
    T1 = UseSel - Arrow + 1
    T2 = T1 + 9
  END IF
  Counter = 0
  FOR T = T1 TO T2
    Counter = Counter + 1
    LOCATE 10 + Counter, 9
      PRINT PlayerInv(T).Item.Iname
    LOCATE 10 + Counter, 34
     PRINT "                    "
    LOCATE 10 + Counter, 34
    IF PlayerInv(T).Amount > 0 THEN
       PRINT PlayerInv(T).Amount
       LOCATE 10 + Counter, 45: PRINT FIX(PlayerInv(T).Item.Cost / 2)
      ELSE
     PRINT "                    "
    END IF

    IF PlayerInv(T).Item.ItemType = 1 THEN 'Item
      LOCATE 10 + Counter, 7
      PRINT ""
    
     ELSEIF PlayerInv(T).Item.ItemType = 2 THEN 'Armor
      LOCATE 10 + Counter, 7
      PRINT "’"

     ELSEIF PlayerInv(T).Item.ItemType = 3 THEN 'Weapons
      LOCATE 10 + Counter, 7
      PRINT "î"
     ELSE
      LOCATE 10 + Counter, 7
      PRINT " "
    END IF
  NEXT T
  IF PlayerInv(UseSel).Amount > 0 THEN
    CALL BCPrint(PlayerInv(UseSel).Item.Desc, Env.FGC, Env.BGC)
   ELSE
    CALL BCPrint("Empty Inventory Location #" + STR$(UseSel), Env.FGC, Env.BGC)
  END IF
  GOSUB DispQty:

RETURN

ChangeQty:
  IF PlayerInv(UseSel).Amount > 0 THEN
    Qty = 1
   ELSE
    Qty = 0
  END IF
RETURN

DispQty:
  LOCATE 8, 20: PRINT Qty
  LOCATE 8, 43: PRINT "        "
  LOCATE 8, 43: PRINT (Qty * FIX(PlayerInv(UseSel).Item.Cost / 2))
RETURN

SellItem: 'Does physical removal of selected item from inventory
  Player(1).Gold = Player(1).Gold + (Qty * FIX(PlayerInv(UseSel).Item.Cost / 2))
  PlayerInv(UseSel).Amount = PlayerInv(UseSel).Amount - Qty
  IF PlayerInv(UseSel).Amount = 0 THEN
    IF (PlayerInv(UseSel).Amount = 0) AND (UseSel < MaxInv) THEN
      FOR X = UseSel TO MaxInv - 1
        CALL SwapItem(PlayerInv(X).Item, PlayerInv(X + 1).Item)
        PlayerInv(X).Amount = PlayerInv(X + 1).Amount
      NEXT X
      CALL Items(0, PlayerInv(MaxInv).Item)
      PlayerInv(MaxInv).Amount = 0
     ELSEIF (PlayerInv(UseSel).Amount = 0) AND (UseSel = MaxInv) THEN
      CALL Items(0, PlayerInv(UseSel).Item)
      PlayerInv(UseSel).Amount = 0
    END IF
  END IF
  GOSUB UpdateSellList:
  LOCATE 6, 50: PRINT "Gold: "; Player(1).Gold
RETURN


ResetShopScreen: 'Clears and displays initial Shop Screen
  CLS
  LOCATE 2, 37:
    IF ShopName$ = "A" THEN
      PRINT "ARMOR SHOP"
     ELSEIF ShopName$ = "W" THEN
      PRINT "WEAPONS SHOP"
     ELSEIF ShopName$ = "I" THEN
      PRINT "ITEM SHOP"
     ELSE
      PRINT "ShopName$"
    END IF

  ChoiceNum = 1 'The choice number on the menu selection
  Selection = 0  'Which menu choice was selected
  Qty = 1  'Amount of items to be purchased
  ItSel = 1   'Item Selection Location
  LINE (0, 0)-(639, 349), Env.FGC, B
  GOSUB UpdateSel
  LINE (15, 7)-(90, 65), Env.FGC, B
RETURN

ShowShopStats: 'This displays the change in player stats for weapons and armor
 
  CALL BCPrint(ShopItems(ItSel).Item.Desc, Env.FGC, Env.BGC)
  FOR T = 14 TO 16
    LOCATE T, 60: PRINT "    "
    LOCATE T, 67: PRINT "    "
  NEXT T
   
  LOCATE 8, 40:
  IF ShopItems(ItSel).Cost = 0 THEN
    PRINT "Total Price: "; (Qty * ShopItems(ItSel).Item.Cost); "     "
   ELSE
    PRINT "Total Price: "; (Qty * ShopItems(ItSel).Cost); "     "
  END IF
 
  
  LOCATE 10, 50: PRINT "  Player: "; Player(PNum).PName; "     "
  LOCATE 12, 50: PRINT "    Gold: "; Player(1).Gold; "     "
  LOCATE 14, 50: PRINT " Defense:"; Player(PNum).BaseDefense; "   "
  LOCATE 15, 50: PRINT "Strength:"; Player(PNum).Strength; "  "
  LOCATE 16, 50: PRINT "   Speed:"; Player(PNum).Speed; "   "
  BD = Player(PNum).BaseDefense
  St = Player(PNum).Strength
  SD = Player(PNum).Speed
 
  St = Player(PNum).Strength + ShopItems(ItSel).Item.StrengthEffect
  SD = Player(PNum).Speed + ShopItems(ItSel).Item.SpeedEffect
  BD = Player(PNum).BaseDefense + ShopItems(ItSel).Item.DefenseEffect
 
  LOCATE 14, 65: PRINT ""; BD; "  "
  LOCATE 15, 65: PRINT ""; St; "  "
  LOCATE 16, 65: PRINT ""; SD; "  "

  FOR T = 14 TO 16
    LOCATE T, 72: PRINT " "
  NEXT T
 
  IF BD > Player(PNum).BaseDefense THEN
    LOCATE 14, 72: PRINT ""
  ELSEIF BD < Player(PNum).BaseDefense THEN
    LOCATE 14, 72: PRINT ""
  END IF

  IF St > Player(PNum).Strength THEN
    LOCATE 15, 72: PRINT ""
  ELSEIF St < Player(PNum).Strength THEN
    LOCATE 15, 72: PRINT ""
  END IF

  IF SD > Player(PNum).Speed THEN
    LOCATE 16, 72: PRINT ""
  ELSEIF SD < Player(PNum).Speed THEN
    LOCATE 16, 72: PRINT ""
  END IF
  FOR T = 1 TO NumberMembers
    Equipable = False
    I = 0
    EW$ = ShopItems(ItSel).Item.EquipWho
    DO
      I = I + 1
      IF (Player(T).PLet = MID$(EW$, I, 1)) THEN
        Equipable = True
      END IF
    LOOP UNTIL (Equipable = True) OR (I = 4)
    LOCATE 6, (T * 11) + 25
    IF Equipable = True THEN
      PRINT "*"
     ELSE
      PRINT " "
    END IF
    LOCATE 6, (T * 11) + 26
    IF (ShopItems(ItSel).Item.Num = Player(T).LeftArm.Num) THEN
      PRINT "L"
     ELSE
      PRINT " "
    END IF
    LOCATE 6, (T * 11) + 28
    IF (ShopItems(ItSel).Item.Num = Player(T).RightArm.Num) THEN
      PRINT "R"
     ELSE
      PRINT " "
    END IF
    LOCATE 6, (T * 11) + 30
    IF (ShopItems(ItSel).Item.Num = Player(T).Armor.Num) THEN
      PRINT "B"
     ELSE
      PRINT " "
    END IF
  NEXT T
  

RETURN

DispShopPName:
FOR T = 1 TO NumberMembers
  LOCATE 4, (T * 11) + 25
  PRINT Player(T).PName
  LINE (180 + (T * 90), 38)-STEP(90, 20), Env.BGC, B
NEXT T
  LINE (180 + (PNum * 90), 38)-STEP(90, 20), Env.FGC, B
RETURN

PurchaseItem:
IF ShopItems(ItSel).Cost = 0 THEN
  TotalPurchasePrice = (Qty * ShopItems(ItSel).Item.Cost)
 ELSE
  TotalPurchasePrice = (Qty * ShopItems(ItSel).Cost)
END IF

IF (Player(1).Gold >= TotalPurchasePrice) THEN
  'Buying Item
 T = 0
 DO
   T = T + 1
 LOOP UNTIL (T = MaxInv) OR (PlayerInv(T).Amount = 0)
 IF (T = MaxInv) AND (PlayerInv(T).Amount <> 0) THEN
    'Inv is full
    SOUND 63, 1
    CALL BCPrint("INVETORY IS FULL!", Env.FGC, Env.BGC)
  ELSE
    'Able to purchase Item
    PlayerInv(T).Item = ShopItems(ItSel).Item
    PlayerInv(T).Item.Cost = INT(TotalPurchasePrice / Qty)
    PlayerInv(T).Amount = Qty
    Player(1).Gold = Player(1).Gold - TotalPurchasePrice
    GOSUB ShowShopStats:
 END IF

 ELSE 'If the player doesn't have neough gold
  SOUND 63, 1
  CALL BCPrint("NOT ENOUGH GOLD!", Env.FGC, Env.BGC)
END IF


RETURN

END SUB

SUB SwapItem (Item1 AS ItemType, Item2 AS ItemType)
      Item1.Num = Item2.Num
      Item1.Iname = Item2.Iname
      Item1.Desc = Item2.Desc
      Item1.ItemType = Item2.ItemType
      Item1.EquipWho = Item2.EquipWho
      Item1.HPEffect = Item2.HPEffect
      Item1.MPEffect = Item2.MPEffect
      Item1.SpeedEffect = Item2.SpeedEffect
      Item1.StrengthEffect = Item2.StrengthEffect
      Item1.DefenseEffect = Item2.DefenseEffect
      Item1.StatusEffect = Item2.StatusEffect
      Item1.Cost = Item2.Cost

END SUB

SUB UseItem (ItemNum, PNum, UsedItem)
'For Inventory Screen, Not for Battle Use

'  ItemType AS INTEGER '(1= Item, 2 = Armor, 3= Weapon, 4 = Specials)
'  StatusEffect AS STRING * 4 '(P = Poison, S = Stone, D = Dead,
            'U = Unconcious, C = Confused, A = asleep, B = Battle Only)
            'N = None

UsedItem = False
IF PlayerInv(ItemNum).Item.ItemType = 1 THEN
  St$ = MID$(PlayerInv(ItemNum).Item.StatusEffect, 1, 1)
  St2$ = MID$(PlayerInv(ItemNum).Item.StatusEffect, 2, 1)
  IF (St$ = "N") OR ((St$ = "Z") AND (St2$ = "N")) THEN
    'If It's not for battle use only
    'or It effects all members
   TempPNum = PNum
   IF (St$ = "Z") THEN      'Looping for entire party
     BPNum = 1
     EPNum = NumberMembers
    ELSE                    'Loops once for selected person
     BPNum = PNum
     EPNum = PNum
   END IF
   FOR PNum = BPNum TO EPNum
    IF (PlayerInv(ItemNum).Item.HPEffect <> 0) AND (Player(PNum).HP <> 0) AND (Player(PNum).HP <> Player(PNum).MaxHP) THEN
      IF Player(PNum).HP + PlayerInv(ItemNum).Item.HPEffect <= Player(PNum).MaxHP THEN
          Player(PNum).HP = Player(PNum).HP + PlayerInv(ItemNum).Item.HPEffect
        ELSE
          Player(PNum).HP = Player(PNum).MaxHP
      END IF
      UsedItem = True
    END IF 'HP effect

    IF (PlayerInv(ItemNum).Item.MPEffect <> 0) AND (Player(PNum).HP <> 0) THEN
      IF Player(PNum).MP + PlayerInv(ItemNum).Item.MPEffect <= Player(PNum).MaxMP THEN
          Player(PNum).MP = Player(PNum).MP + PlayerInv(ItemNum).Item.MPEffect
        ELSE
          Player(PNum).MP = Player(PNum).MaxMP
      END IF
      UsedItem = True
    END IF 'MP effect
    
    IF (PlayerInv(ItemNum).Item.StrengthEffect <> 0) AND (Player(PNum).HP <> 0) THEN
      Player(PNum).Strength = Player(PNum).Strength + PlayerInv(ItemNum).Item.StrengthEffect
      UsedItem = True
    END IF 'Strength effect

    IF (PlayerInv(ItemNum).Item.SpeedEffect <> 0) AND (Player(PNum).HP <> 0) THEN
      Player(PNum).Speed = Player(PNum).Speed + PlayerInv(ItemNum).Item.SpeedEffect
      UsedItem = True
    END IF 'Speed effect

    IF (PlayerInv(ItemNum).Item.DefenseEffect <> 0) AND (Player(PNum).HP <> 0) THEN
      Player(PNum).BaseDefense = Player(PNum).BaseDefense + PlayerInv(ItemNum).Item.DefenseEffect
      UsedItem = True
    END IF 'Speed effect
   NEXT PNum
   PNum = TempPNum
  END IF
 TempPNum = PNum
 IF (St$ = "Z") THEN      'Looping for entire party
   BPNum = 1
   EPNum = NumberMembers
  ELSE                    'Loops once for selected person
   BPNum = PNum
   EPNum = PNum
 END IF
 FOR PNum = BPNum TO EPNum

  IF (St$ = "D") OR ((St$ = "Z") AND (St2$ = "D")) AND (Player(PNum).HP = 0) THEN  'IF The Player is Dead
      IF Player(PNum).HP + PlayerInv(ItemNum).Item.HPEffect <= Player(PNum).MaxHP THEN
          Player(PNum).HP = Player(PNum).HP + PlayerInv(ItemNum).Item.HPEffect
        ELSE
          Player(PNum).HP = Player(PNum).MaxHP
      END IF
      UsedItem = True
  END IF
 NEXT PNum
 PNum = TempPNum
END IF
END SUB