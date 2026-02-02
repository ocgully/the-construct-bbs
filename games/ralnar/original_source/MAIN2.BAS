DECLARE SUB Battle ()
DECLARE SUB Items (Num AS INTEGER, Item AS ANY)
DECLARE SUB Shop (ShopName$, TotalItems AS INTEGER)
DECLARE SUB DispAir (D AS INTEGER, SX!, SY!, Amount AS INTEGER)
DECLARE SUB DispBoat (D AS INTEGER, T!, SX!, SY!, Amount AS INTEGER)
DECLARE SUB EPHandler (X AS INTEGER, Y AS INTEGER, MapName$)
DECLARE SUB DispHero (D AS INTEGER, T!, SX!, SY!, Amount AS INTEGER)
DECLARE SUB LoadMap (Name$)
DECLARE SUB LoadPlayerImages ()
DECLARE SUB DispScreenLeft ()
DECLARE SUB DispScreenRight ()
DECLARE SUB DispScreenUp ()
DECLARE SUB DispScreenDown ()
DECLARE SUB DispScreen ()
DECLARE SUB LoadStuff ()
DECLARE SUB GetMapLoad ()
DECLARE SUB CheckDir ()
DECLARE SUB DispLogo ()
DECLARE SUB FileScreen ()
DECLARE SUB Options ()
DECLARE SUB Inventory ()
DECLARE SUB DispStats ()
DECLARE SUB MainScreenMagic ()
DECLARE SUB PAUSE ()
DECLARE SUB DispMainScreen ()
DECLARE SUB TextFont (S$, X!, Y!, C!, S!)
DECLARE SUB TestEventTriggered ()
DECLARE SUB MainSideMenu (Option$, Done%)
DECLARE SUB Movement (Dir$)
DECLARE SUB MainGame ()
DECLARE SUB StringInput (X%, Y%, Length%, StringName$, UsingTextFont%)
DECLARE SUB Default ()
DECLARE SUB MainMenu (Option$, Done%)
DECLARE SUB NewChar ()
DECLARE SUB LoadChar ()
DECLARE SUB GetKey (Option$)
DECLARE SUB PrintL (Text$, X%, Y%, Col%)
OPTION BASE 1
'The Realm of Ralnar
'Version 1.0 By Brian And Christopher Gulliver
'Programed in QuickBASIC v4.5 '1996-1997
'$DYNAMIC

' * TYPES

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

CONST True = 1
CONST False = 0
CONST MaxInv = 50  'Max number of items in the inventory
                   'Max inv also in GameSubs Module
'CONST DirRight = 1, DirDown = 2, DirLeft = 3, DirUp = 4
CONST ESC = 27, Down = 80, Up = 72, Left = 75, Right = 77, ENTER = 13

CONST MMIPATH$ = "C:\BG_RPG\MMI\"
CONST IconList$ = "MMIFILES.TXT"
CONST MapPath$ = "C:\BG_RPG\Maps\"
CONST PICPATH$ = "C:\BG_RPG\PICS\" 'Where Player Images are stored


' * Arrays
DIM SHARED Player(4) AS PlayerType
DIM SHARED PlayerInv(MaxInv) AS InvType
DIM SHARED Map(1, 1, 1) AS INTEGER
DIM SHARED MapIcons(203, 1) AS INTEGER
DIM SHARED Hero(4, 4, 20, 20) AS INTEGER
DIM SHARED Boat(4, 2, 20, 20) AS INTEGER
DIM SHARED Air(5, 20, 20) AS INTEGER
DIM SHARED ShopItems(10) AS ShopItemType
DIM SHARED SpecialItems AS SpecialType
DIM SHARED TChest(50) AS INTEGER 'Treasure Chest (1 = Full, 0 = Empty)
FOR T = 1 TO 50
  TChest(T) = 1
NEXT T
DEFINT A-Z
' * * * * * * * * MAIN PROGRAM * * * * * * * * *
RANDOMIZE TIMER
'CALL LoadPlayerImages
Env.BGC = 1
Env.FGC = 7
NumberMembers = 1
FOR T = 1 TO 50
  PlayerInv(T).Amount = T
  C = INT(RND * 20) + 1

'  SELECT CASE C
'    CASE 1
'      C = 1
'    CASE 2
'      C = 7
'    CASE 3
'      C = 201
'    CASE 4
'      C = 100
'    CASE 5
'      C = 200
'  END SELECT
  CALL Items(C, PlayerInv(T).Item)
NEXT T
CALL Inventory
'END
'Do Opening Title Screen
CLS
Done% = False
SCREEN 13: CLS
CALL DispLogo
CALL MainMenu(Option$, Done%)
DO
IF Option$ = "S" THEN
    CALL NewChar
  ELSEIF Option$ = "D" THEN
    CALL Default
  ELSE
    CALL LoadChar
END IF
IF Done% = False THEN CALL MainMenu(Option$, Done%)
LOOP UNTIL Done% = True
COLOR 7: PALETTE
END 'END OF GAME


DEFSNG A-Z
SUB CheckDir
DIM Sz AS INTEGER
XI = 5
YI = 5
EventTriggered = False
BoatNext = False
ValidDir = False
SELECT CASE Location.Mode
         CASE IS = 1  'Land
             SELECT CASE Map(Location.NX, Location.NY, 2)
             
                CASE IS = 0
                  BEEP
             
                CASE IS = 1, 7, 8, 9
                  ValidDir = True

                CASE IS = 4, 5
                  ValidDir = False

                CASE IS = 2  'IF the next one is water
                  IF Location.MapName = "WORLD     " OR Location.MapName = "SAMPLE    " THEN
                    IF (Location.ShipX = Location.NX) AND (Location.ShipY = Location.NY) THEN
                      ValidDir = True
                      BoatNext = True
                      Location.ShipX = 0
                      Location.ShipY = 0
                    END IF
                   ELSE
                    ValidDir = False
                  END IF


                CASE IS = 3
                  ValidDir = True
                  EventTriggered = True

             END SELECT

         CASE IS = 2  'Water
             SELECT CASE Map(Location.NX, Location.NY, 2)
             
                CASE IS = 0
                  BEEP

                CASE IS = 4, 5, 7, 8
                  ValidDir = False

                CASE IS = 1
                  ValidDir = True
                  Location.ShipX = Location.X
                  Location.ShipY = Location.Y
                  Location.Mode = 1

                CASE IS = 2, 9
                  ValidDir = True

                CASE IS = 3
                  ValidDir = True
                  EventTriggered = True

             END SELECT

         CASE IS = 3  'Air
             SELECT CASE Map(Location.NX, Location.NY, 2)
                CASE IS = 0
                  BEEP
             
                CASE IS = 1, 2, 4, 5, 7, 8, 9
                  ValidDir = True

                CASE IS = 3
                  ValidDir = True
                  IF (Location.X = 64) AND (Location.Y = 46) THEN
                    EventTriggered = True
                  END IF
                  'Only Event Passable place for the AirShip is the
                  'Sky Palace.

             END SELECT


END SELECT

IF (ValidDir = True) THEN
  Location.X = Location.NX
  Location.Y = Location.NY
  IF Location.Mode < 3 THEN
    SELECT CASE Location.Dir
      CASE Right
        CALL DispScreenRight
      CASE Left
        CALL DispScreenLeft
      CASE Up
        CALL DispScreenUp
      CASE Down
        CALL DispScreenDown
    END SELECT
    LINE (299, 0)-STEP(19, 19), 19, BF
   ELSE 'If Mode is 3 (AirShip)
    DispScreen
    SELECT CASE Location.Dir
      CASE Right
        CALL DispAir(4, XI + 100, YI + 80, 20)
      CASE Left
        CALL DispAir(2, XI + 100, YI + 80, 20)
      CASE Up
        CALL DispAir(3, XI + 100, YI + 80, 20)
      CASE Down
        CALL DispAir(1, XI + 100, YI + 80, 20)
    END SELECT

  END IF

 ELSEIF (Map(Location.X, Location.Y, 2) <> 8) THEN
  IF Map(Location.X, Location.Y, 2) = 7 THEN
    Sz = Location.HalfSeeSize
   ELSE
    Sz = Location.FullSeeSize
  END IF
'If the attrib is 7 (Half See) then the the size(Sz) the map's halfsize
PUT (100 + XI, 80 + YI), MapIcons(1, Map(Location.X, Location.Y, 1)), PSET
SELECT CASE Location.Dir  'Used to change position if not able to move
  CASE Down
    IF Location.Mode = 1 THEN
      CALL DispHero(1, 4, XI + 100, YI + 80, Sz)
     ELSE
      CALL DispBoat(1, 2, XI + 100, YI + 80, Location.FullSeeSize)
   END IF
  CASE Left
    IF Location.Mode = 1 THEN
      CALL DispHero(2, 4, XI + 100, YI + 80, Sz)
     ELSE
      CALL DispBoat(2, 2, XI + 100, YI + 80, Location.FullSeeSize)
   END IF
  CASE Up
    IF Location.Mode = 1 THEN
      CALL DispHero(3, 4, XI + 100, YI + 80, Sz)
     ELSE
      CALL DispBoat(3, 2, XI + 100, YI + 80, Location.FullSeeSize)
   END IF
  CASE Right
    IF Location.Mode = 1 THEN
      CALL DispHero(4, 4, XI + 100, YI + 80, Sz)
     ELSE
      CALL DispBoat(4, 2, XI + 100, YI + 80, Location.FullSeeSize)
   END IF
END SELECT
SOUND 63, .5
END IF

IF EventTriggered = True THEN
  CALL EPHandler(Location.X, Location.Y, Location.MapName)
 ELSEIF BoatNext = True THEN
  Location.Mode = 2
  CALL DispScreen
  CALL DispBoat(1, T, XI + 100, YI + 80, Location.FullSeeSize)
  Location.Dir = Down
ELSE
  'Handles the randomization of battles
  IF (Location.Enemies = True) AND (ValidDir = True) AND Location.Mode <> 3 THEN
    Env.LastB = Env.LastB + 1
    X = INT(RND * 100)   'To increase rate decrease number, vice versa
    IF X > 100 - Env.LastB THEN
      CALL Battle
      Env.LastB = 0
    END IF
   ELSE
    Env.LastB = 0
  END IF
END IF
END SUB

SUB Credits
END SUB

SUB Default
PALETTE: CLS
COLOR 7
'Sets a default player
Player(1).PName = "Default"
Player(1).PLet = "A"
Player(1).Level = 1
Player(1).HP = 25
Player(1).MaxHP = 30
Player(1).MP = 10
Player(1).MaxMP = 10
Player(1).Status = 1  '1 = Okay, status is for battle use only
Player(1).Strength = 10
Player(1).BaseDefense = 1
Player(1).MAttack = 0
Player(1).MDefense = 2
Player(1).PhysicalLearnRate = 5  '%?
Player(1).MagicLearnRate = 5 '%?
Player(1).PLimit = 0
Player(1).MLimit = 0
Player(1).XP = 34
Player(1).XPNext = 120
  CALL Items(0, Player(1).LeftArm)
  CALL Items(200, Player(1).RightArm)
  CALL Items(100, Player(1).Armor)
Player(1).Speed = 0
Player(1).Gold = 100
NumberMembers = 1
INPUT "Enter name of screen to load (no ext): ", ScName$
CALL LoadMap(ScName$)
Location.X = 10
Location.Y = 10
Location.Mode = 1
Location.Dir = Down
Location.ShipExist = True  'Have Ship
Location.AirExist = True 'Have Airship
Player(2).PLet = "B"
Player(2).PName = "Bobbityurn"
Player(2).MaxHP = 100
Player(3).PName = "Jim"
Player(3).PLet = "C"
Player(3).MaxHP = 30
Player(4).PName = "Humani"
Player(4).PLet = "D"
Player(4).MaxHP = 9999
NumberMembers = 4



CALL MainGame

END SUB

DEFINT A-Z
SUB Initialize

FOR T = 1 TO 50
  TChest(T) = 1
NEXT T

END SUB

SUB LoadChar
END SUB

DEFSNG A-Z
SUB LoadMap (Name$)
DIM TMI AS INTEGER
Location.MapName = UCASE$(Name$)
Name$ = Name$ + ".NMF"
OPEN MapPath$ + Name$ FOR RANDOM AS #1 LEN = 2
  GET #1, 1, Location.MapDimX
  GET #1, 2, Location.MapDimY
  GET #1, 3, Location.Enemies
  GET #1, 4, Location.HalfSeeSize
  GET #1, 5, Location.FullSeeSize
  GET #1, 6, TMI
  REDIM Map(Location.MapDimX, Location.MapDimY, 2) AS INTEGER
  FOR X = 1 TO Location.MapDimX
    GOSUB DispLoadPercent
    FOR Y = 1 TO Location.MapDimY
      GET #1, , Map(X, Y, 1)
      GET #1, , Map(X, Y, 2)
    NEXT Y
  NEXT X
  REDIM MapIcons(203, TMI)AS INTEGER
  FOR Y = 1 TO TMI
    FOR X = 1 TO 203
      GET #1, , MapIcons(X, Y)
    NEXT X
  NEXT Y
CLOSE #1

EXIT SUB 'End's Sub

DispLoadPercent:
  PDone = CINT((X / Location.MapDimX) * 100)
  LOCATE 2, 1
  PRINT "Loading: "; PDone; "%"
RETURN


END SUB

'Main Program that gets input and calls the functions required to
'make the game flow in an orderly fashion.
SUB MainGame
XI = 5: YI = 5
SCREEN 13: CLS
PALETTE  'Resets Defaults
Done% = False
CALL DispMainScreen
CALL DispScreen
CALL DispHero(Location.Dir, 4, XI + 100, YI + 80, Location.FullSeeSize)
DO
CALL GetKey(Option$)
SELECT CASE Option$
   
  CASE IS = CHR$(0) + CHR$(Up), CHR$(0) + CHR$(Down), CHR$(0) + CHR$(Right), CHR$(0) + CHR$(Left)
    CALL Movement(Option$)
  
  CASE IS = CHR$(0) + CHR$(33), CHR$(0) + CHR$(31), CHR$(0) + CHR$(23), CHR$(0) + CHR$(50)
     'Alt F / Alt S / Alt I / Alt M
    CALL MainSideMenu(Option$, Done%)
    CALL DispMainScreen
 
  CASE IS = "F", "S", "I", "M"
    CALL MainSideMenu(Option$, Done%)
    CALL DispMainScreen
 
  CASE IS = " "
    CALL TestEventTriggered
 
  CASE IS = CHR$(0) + ";"  'F1 = Help
    'Takes you to the help Screen

  CASE IS = CHR$(0) + "<"    'F2 = QuickSave
    'Takes you to the save screen
 
  CASE IS = CHR$(0) + CHR$(16)  'Alt Q
    CLS
    Done% = True
   
  CASE IS = "C"
    IF Location.Mode = 3 THEN
      Location.Mode = 1
     ELSE
      Location.Mode = Location.Mode + 1
    END IF

  CASE IS = "~"
    PRINT FRE(-1)

  CASE IS = "!"
    DIM TempIcon(203) AS INTEGER
    DIM TempIcon2(203) AS INTEGER
    FOR T = 1 TO 1000
      X1 = ((INT(RND * 41)) * 5) + 5
      Y1 = ((INT(RND * 33)) * 5) + 5
      GET (X1, Y1)-STEP(19, 19), TempIcon
      X2 = ((INT(RND * 41)) * 5) + 5
      Y2 = ((INT(RND * 33)) * 5) + 5
      GET (X2, Y2)-STEP(19, 19), TempIcon2
      PUT (X2, Y2), TempIcon, PSET
      PUT (X1, Y1), TempIcon2, PSET
    NEXT T
    FOR T = 1 TO 1000
      X1 = ((INT(RND * 41)) * 5) + 5
      Y1 = ((INT(RND * 33)) * 5) + 5
      GET (X1, Y1)-STEP(19, 19), TempIcon
      X2 = ((INT(RND * 41)) * 5) + 5
      Y2 = ((INT(RND * 33)) * 5) + 5
      GET (X2, Y2)-STEP(19, 19), TempIcon2
      PUT (X2, Y2), TempIcon, AND
      PUT (X1, Y1), TempIcon2, AND
    NEXT T

    ERASE TempIcon
    ERASE TempIcon2
    CALL DispScreen

END SELECT

LOOP UNTIL Done%
CALL DispLogo
END SUB

SUB MainMenu (Option$, Done%)
CALL TextFont(" )tart a New Game", 116, 150, 7, 0)
CALL TextFont(" )oad an Old Game", 116, 160, 7, 0)
CALL TextFont(" )uit", 117, 170, 7, 0)
CALL TextFont("S", 115, 150, 5, 0)
CALL TextFont("L", 115, 160, 5, 0)
CALL TextFont("Q", 115, 170, 5, 0)
CALL GetKey(Option$)
IF Option$ = "Q" THEN Done% = True



END SUB

SUB MainScreenMagic
PRINT "MainScreen Magic"
END SUB

SUB Movement (Dir$)

Location.NX = Location.X
Location.NY = Location.Y


SELECT CASE Dir$
      
       CASE IS = CHR$(0) + "H"    'UP
         IF Location.Dir <> Up THEN Location.Dir = Up
         IF Location.Y = 1 THEN
            Location.NY = Location.MapDimY
            Location.NX = Location.X
           ELSE
            Location.NY = Location.Y - 1
             Location.NX = Location.X
          END IF
         CALL CheckDir
    
      CASE IS = CHR$(0) + "P"    'DOWN
        IF Location.Dir <> Down THEN Location.Dir = Down
         IF Location.Y = Location.MapDimY THEN
            Location.NY = 1
            Location.NX = Location.X
           ELSE
             Location.NY = Location.Y + 1
             Location.NX = Location.X
         END IF
         CALL CheckDir
    
      CASE IS = CHR$(0) + "K"    'LEFT
      IF Location.Dir <> Left THEN Location.Dir = Left
        IF Location.X = 1 THEN
            Location.NX = Location.MapDimX
            Location.NY = Location.Y
           ELSE
             Location.NX = Location.X - 1
             Location.NY = Location.Y
          END IF
        CALL CheckDir
       
  
      CASE IS = CHR$(0) + "M"    'RIGHT
      IF Location.Dir <> Right THEN Location.Dir = Right
        IF Location.X = Location.MapDimX THEN
            Location.NX = 1
            Location.NY = Location.Y
           ELSE
             Location.NX = Location.X + 1
             Location.NY = Location.Y
          END IF
        CALL CheckDir
  
END SELECT
COLOR 15
'LOCATE 1, 31: PRINT "Name:"; Location.MapName$
'LOCATE 1, 31: PRINT "X:"; Location.X; " "
'LOCATE 2, 31: PRINT "Y:"; Location.Y; " "
'LOCATE 3, 31: PRINT "NX:"; Location.NX; " "
'LOCATE 4, 31: PRINT "NY:"; Location.NY; " "
'LOCATE 5, 31: PRINT "NA:"; Map(Location.NX, Location.NY, 2); " "
'LOCATE 6, 31: PRINT "SX:"; Location.ShipX; " "
'LOCATE 7, 31: PRINT "SY:"; Location.ShipY; " "
'LOCATE 8, 31: PRINT "AX:"; Location.AirX; " "
'LOCATE 9, 31: PRINT "AY:"; Location.AirY

'LINE (5 * 20 + 4, 4 * 20 + 4)-STEP(21, 21), 15, B
'GOSUB PrintMode
'GOSUB ShowArrow

 
EXIT SUB 'Leaves Sub


'PrintMode: 'Gosub Label
'LINE (238, 155)-(319, 168), 0, BF
'SELECT CASE Location.Mode
'     CASE IS = 1
'        CALL TextFont("Land Mode", 240, 160, 2, 0)
'
'     CASE IS = 2
'        CALL TextFont("Boat Mode", 240, 160, 3, 0)
'
'     CASE IS = 3
'        CALL TextFont("Air Mode", 240, 160, 9, 0)
'
'END SELECT
'RETURN
'
'ShowArrow:
'
'SELECT CASE Location.Dir
'      CASE IS = Up
'          PSET (115, 97), 15
'          DRAW "C15;u5;g2e2f2"
'
'      CASE IS = Down
'          PSET (115, 92), 15
'          DRAW "C15;d5;h2f2e2"
'
'      CASE IS = Right
'          PSET (113, 94), 15
'          DRAW "C15;r5;h2f2g2"
'
'      CASE IS = Left
'          PSET (117, 94), 15
'          DRAW "C15;l5;e2g2f2"
'
'END SELECT
'RETURN

END SUB

SUB NewChar
PALETTE
CLS
COLOR 7
LOCATE 1, 1:
PRINT "Enter your Name: "
CALL StringInput(18, 1, 10, Player(1).PName, False)
Player(1).PLet = "A"
Player(1).Level = 1
Player(1).HP = 30
Player(1).MaxHP = 30
Player(1).MP = 10
Player(1).MaxMP = 10
Player(1).Status = 1  '1 = Okay, status is for battle use only
Player(1).Strength = 10
Player(1).BaseDefense = 1
Player(1).MAttack = 0
Player(1).MDefense = 2
Player(1).PhysicalLearnRate = 5  '%?
Player(1).MagicLearnRate = 5 '%?
Player(1).PLimit = 0
Player(1).MLimit = 0
Player(1).XP = 0
Player(1).XPNext = 120
  CALL Items(0, Player(1).LeftArm)
  CALL Items(200, Player(1).RightArm)
  CALL Items(0, Player(1).Armor)
Player(1).Speed = 1
NumberMembers = 1


CALL LoadMap("Town1")

'Runs a Small Opening Sequence And sets the location to coordinates in the
'first town

Player(1).PName = UCASE$(Player(1).PName)
Location.X = 10
Location.Y = 10
Location.NX = 10
Location.NY = 11
Location.Mode = 1
Location.Dir = Down
Env.Icon = True

CALL MainGame

END SUB

SUB Options
NumItems = 5
LINE (320, 50)-(540, 250), Env.FGC, B
SelLoc = 1   'Selection Location
GOSUB UpdateOptions

DO
CALL GetKey(Key$)
  SELECT CASE Key$
    CASE IS = "Q", "B", CHR$(ESC), " ", CHR$(ENTER)
      Key$ = CHR$(27)
 
    CASE IS = "H", "1", CHR$(0) + ";" 'Help/F1
      'CALL Help
   
    CASE IS = CHR$(0) + CHR$(Down) 'Down
      IF SelLoc = NumItems THEN
          SelLoc = 1
        ELSE
          SelLoc = SelLoc + 1
      END IF
      GOSUB OpCheckCol
      GOSUB UpdateOptions

    CASE IS = CHR$(0) + CHR$(Up) 'Up
      IF SelLoc = 1 THEN
          SelLoc = NumItems
        ELSE
          SelLoc = SelLoc - 1
      END IF
      GOSUB OpCheckCol
      GOSUB UpdateOptions

    CASE IS = CHR$(0) + CHR$(Right)
       SELECT CASE SelLoc
          CASE 1
             IF Env.Music = True THEN
                Env.Music = False
              ELSE
                Env.Music = True
             END IF
             GOSUB UpdateOptions
         
          CASE 2
            IF (Env.BGC + 1 = Env.FGC) AND (Env.BGC + 1 < 5) THEN
               Env.BGC = Env.BGC + 2
             ELSEIF (Env.BGC + 1 <> Env.FGC) AND (Env.BGC + 1 <= 5) THEN
               Env.BGC = Env.BGC + 1
             ELSE
               Env.BGC = 0
            END IF
            GOSUB UpdateOptions
        
         CASE 3
            IF (Env.FGC + 1 = Env.BGC) AND (Env.FGC + 1 < 15) THEN
               Env.FGC = Env.FGC + 2
             ELSEIF (Env.FGC + 1 <> Env.BGC) AND (Env.FGC + 1 <= 15) THEN
               Env.FGC = Env.FGC + 1
             ELSEIF (Env.FGC + 1 > 15) AND (Env.BGC = 1) THEN
               Env.FGC = 2
             ELSE
               Env.FGC = 1
            END IF
            GOSUB UpdateOptions
         
          CASE 4
            IF Env.MS = 9 THEN Env.MS = 1 ELSE Env.MS = Env.MS + 1
            GOSUB UpdateOptions
         
          CASE 5
             IF Env.Icon = True THEN
                Env.Icon = False
              ELSE
                Env.Icon = True
             END IF
             GOSUB UpdateOptions
       

       END SELECT
   
    CASE IS = CHR$(0) + CHR$(Left)
       SELECT CASE SelLoc
          CASE 1
             IF Env.Music = True THEN
                Env.Music = False
              ELSE
                Env.Music = True
             END IF
             GOSUB UpdateOptions
        
          CASE 2
            IF (Env.BGC - 1 = Env.FGC) AND (Env.BGC - 1 > 0) THEN
               Env.BGC = Env.BGC - 2
             ELSEIF (Env.BGC - 1 <> Env.FGC) AND (Env.BGC - 1 >= 0) THEN
               Env.BGC = Env.BGC - 1
             ELSEIF (Env.BGC - 1 < 0) AND (Env.FGC = 5) THEN
               Env.BGC = 4
             ELSE
               Env.BGC = 5
            END IF
            GOSUB UpdateOptions
       
         CASE 3
            IF (Env.FGC - 1 = Env.BGC) AND (Env.FGC - 1 > 1) THEN
               Env.FGC = Env.FGC - 2
             ELSEIF (Env.FGC - 1 <> Env.BGC) AND (Env.FGC - 1 >= 1) THEN
               Env.FGC = Env.FGC - 1
             ELSEIF (Env.FGC - 1 < 1) AND (Env.BGC = 15) THEN
               Env.FGC = 14
             ELSE
               Env.FGC = 15
            END IF
            GOSUB UpdateOptions
        
          CASE 4
            IF Env.MS <= 1 THEN Env.MS = 9 ELSE Env.MS = Env.MS - 1
            GOSUB UpdateOptions
         
          CASE 5
             IF Env.Icon = True THEN
                Env.Icon = False
              ELSE
                Env.Icon = True
             END IF
             GOSUB UpdateOptions

       END SELECT
       
 
 
  END SELECT

LOOP UNTIL Key$ = CHR$(27)
GOSUB OpCheckCol


EXIT SUB

UpdateOptions: 'Gosub Label
COLOR Env.FGC, Env.BGC
LINE (320, 50)-(540, 250), Env.FGC, B
FOR T = 1 TO NumItems
CALL PrintL(" ", 42, T + 4, 1)
NEXT T
FOR T = 1 TO 5
LINE (45, 27 + (T * 28))-STEP(145, 15), Env.BGC, B
NEXT T
CALL PrintL("Help", 10, 5, Env.FGC)
CALL PrintL("Save", 10, 7, Env.FGC)
CALL PrintL("Load", 10, 9, Env.FGC)
CALL PrintL("Options", 10, 11, Env.FGC)
CALL PrintL("Back to Game", 10, 13, Env.FGC)

CALL PrintL("¯", 42, SelLoc + 4, Env.FGC)
IF Env.Music = True THEN
   CALL PrintL("Music ON ", 45, 5, Env.FGC)
  ELSE
   CALL PrintL("Music OFF", 45, 5, Env.FGC)
END IF
CALL PrintL("BGC" + STR$(Env.BGC) + " ", 45, 6, Env.FGC)
CALL PrintL("FGC" + STR$(Env.FGC) + " ", 45, 7, Env.FGC)
CALL PrintL("Û", 55, 7, Env.FGC)
CALL PrintL("Message Speed" + STR$(Env.MS) + " ", 45, 8, Env.FGC)
IF Env.Icon = True THEN
   CALL PrintL("Icon ON ", 45, 9, Env.FGC)
  ELSE
   CALL PrintL("Icon OFF", 45, 9, Env.FGC)
END IF

   

GOSUB UpdateOptionMessage
RETURN

UpdateOptionMessage:
SELECT CASE SelLoc
  CASE 1
     Message$ = "Toggles sound On/Off"
  CASE 2
     Message$ = "Menu screens background color"
  CASE 3
     Message$ = "Menu screens foreground color"
  CASE 4
     Message$ = "Goes to the options screen"
  CASE 5
     Message$ = "Toggles player icons (On/Off)"
  CASE 6
     Message$ = "Exits back to game"
END SELECT
LOCATE 23, 1: PRINT STRING$(79, " ")
LOCATE 23, (40 - CINT(LEN(Message$) / 2)): PRINT Message$
RETURN

OpCheckCol: 'Gosub Label
IF Env.BGC = Env.FGC THEN
  SELECT CASE Env.BGC
      CASE 1, 2, 3, 4, 5, 7
         Env.FGC = Env.FGC + 1
  END SELECT
END IF
COLOR Env.FGC, Env.BGC
RETURN

END SUB