DECLARE SUB DispScreen ()
DECLARE SUB DispMainScreen ()
DECLARE SUB DispAir (D AS INTEGER, SX!, SY!, Amount AS INTEGER)
DECLARE SUB DispBoat (D AS INTEGER, T!, SX!, SY!, Amount AS INTEGER)
DECLARE SUB DispHero (D AS INTEGER, T!, SX!, SY!, Amount AS INTEGER)
DECLARE SUB DispSideMenuDesc (Text$)
DECLARE SUB Inventory ()
DECLARE SUB DispStats ()
DECLARE SUB MainScreenMagic ()
DECLARE SUB FileScreen ()
DECLARE SUB GetKey (Option$)
DECLARE SUB DispSmallStatusBox ()
DECLARE SUB TextFont (S$, X!, Y!, C!, S!)
DECLARE SUB PAUSE ()
OPTION BASE 1
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

COMMON SHARED NumberMembers AS INTEGER, Player   AS PlayerType
COMMON SHARED Enemies AS INTEGER, Map   AS INTEGER
COMMON SHARED Env AS EnvironmentType, Location AS LocationType
COMMON SHARED MapIcons   AS INTEGER, Hero   AS INTEGER, Boat   AS INTEGER
COMMON SHARED Air   AS INTEGER, ShopItems   AS ShopItemType
COMMON SHARED PlayerInv   AS InvType
' * CONST

CONST True = 1
CONST False = 0
CONST MaxInv = 50  'Max number of items in the inventory
                   'Max inv also in GameSubs Module
'CONST DirRight = 1, DirDown = 2, DirLeft = 3, DirUp = 4
CONST ESC = 27, Down = 80, Up = 72, Left = 75, Right = 77, ENTER = 13


DEFSNG A-Z
SUB DispAir (D AS INTEGER, SX, SY, Amount AS INTEGER)
'T = Loop 1-4 T is the # movement to be placed
'D = The Direction (1 = Front, 2 = Left, 3 = Back, 4 = Right)
'SX = Starting X
'SY = Starting Y
'Amount is a # of pixels shown
'

SELECT CASE D  'Used to change position if not able to move
  CASE Down
      D = 1
  CASE Left
      D = 2
  CASE Up
      D = 3
  CASE Right
      D = 4
END SELECT
IF Env.Icon = True THEN
  FOR TY = 1 TO Amount
    FOR TX = 1 TO 20
      IF Air(D, TX, TY) <> -1 THEN
        PSET (SX + TX - 1, SY + TY - 1), Air(D, TX, TY)
      END IF
    NEXT TX
  NEXT TY
ELSE
  LINE (SX + TX, SY + TY)-STEP(19, Amount - 1), 7, B
END IF


END SUB

SUB DispAll
'Does two calls in a single one
CALL DispScreen
CALL DispHero(Location.Dir, 4, XI + 105, YI + 85, Location.FullSeeSize)
END SUB

SUB DispBoat (D AS INTEGER, T, SX, SY, Amount AS INTEGER)
'T = Loop 1-4 T is the # movement to be placed
'D = The Direction (1 = Front, 2 = Left, 3 = Back, 4 = Right)
'SX = Starting X
'SY = Starting Y
'Amount is a # of pixels shown
'
TT = T
SELECT CASE D  'Used to change position if not able to move
  CASE Down
      D = 1
  CASE Left
      D = 2
  CASE Up
      D = 3
  CASE Right
      D = 4
END SELECT

IF Env.Icon = True THEN

IF (TT = 2) OR (TT = 4) THEN  'If T = (2,4) TT=1  IF T = (1,3) TT = 2
   TT = 1                    'Only 2 ship movements
 ELSE
   TT = 2
END IF

  FOR TY = 1 TO Amount
    FOR TX = 1 TO 20
      IF Boat(D, TT, TX, TY) <> -1 THEN
        PSET (SX + TX, SY + TY - 1), Boat(D, TT, TX, TY)
      END IF
    NEXT TX
  NEXT TY
ELSE
  LINE (SX + TX, SY + TY)-STEP(19, Amount - 1), 7, B
END IF

END SUB


SUB DispHero (D AS INTEGER, T, SX, SY, Amount AS INTEGER)
'T = Loop 1-4 T is the # movement to be placed
'D = The Direction (1 = Front, 2 = Left, 3 = Back, 4 = Right)
'SX = Starting X
'SY = Starting Y
'Amount is a # of pixels shown
'
SELECT CASE D  'Used to change position if not able to move
  CASE Down
      D = 1
  CASE Left
      D = 2
  CASE Up
      D = 3
  CASE Right
      D = 4
END SELECT


IF Env.Icon = True THEN
  FOR TY = 1 TO Amount
    FOR TX = 1 TO 20
      IF Hero(D, T, TX, TY) <> -1 THEN
        PSET (SX + TX, SY + TY - 1), Hero(D, T, TX, TY)
      END IF
    NEXT TX
  NEXT TY
ELSE
  LINE (SX + TX, SY + TY)-STEP(19, Amount - 1), 7, B
END IF

END SUB

SUB DispMainScreen
SCREEN 13
FOR T = 4 TO 0 STEP -1
LINE (0 + T, 0 + T)-STEP(229 - 2 * T, 189 - 2 * T), 20 + 2 * T, B
NEXT T
PAINT (300, 100), 19, 20

CALL TextFont(" ile", 251, 12, 24, 0)
CALL TextFont("F", 250, 12, 21, 0)
CALL TextFont(" tats", 251, 22, 24, 0)
CALL TextFont("S", 250, 22, 21, 0)
CALL TextFont(" nventory", 251, 32, 24, 0)
CALL TextFont("I", 250, 32, 21, 0)
CALL TextFont(" agic", 253, 42, 24, 0)
CALL TextFont("M", 250, 42, 21, 0)
CALL TextFont(" uit", 252, 52, 24, 0)
CALL TextFont("Q", 250, 52, 21, 0)

CALL DispSmallStatusBox
END SUB

SUB DispScreen
IY = 4
XX = 4
YY = IY
'  This can be used to place an icon in a location when the coordinates are
'  (XX,YY) on Screen Grid.

    FOR X = Location.X - 5 TO Location.X + 5
        IF X <= 0 THEN
          MapX = Location.MapDimX + X
        ELSEIF X >= (Location.MapDimX + 1) THEN
            MapX = X - Location.MapDimX
          ELSE
            MapX = X
        END IF
   
      FOR Y = Location.Y - 4 TO Location.Y + 4
          IF Y <= 0 THEN
            MapY = Location.MapDimY + Y
          ELSEIF Y >= (Location.MapDimY + 1) THEN
              MapY = Y - Location.MapDimY
            ELSE
              MapY = Y
          END IF
     
     
        IF Map(MapX, MapY, 1) <> 0 THEN
           PUT (XX + 1, YY + 1), MapIcons(1, Map(MapX, MapY, 1)), PSET
         ELSE
           LINE (XX + 1, YY + 1)-STEP(19, 19), 0, BF
        END IF
        IF Location.MapName = "WORLD     " OR Location.MapName = "SAMPLE    " THEN
          IF (Location.ShipExist = True) AND (MapX = Location.ShipX) AND (MapY = Location.ShipY) THEN
            CALL DispBoat(1, 1, XX, YY, 20)
          END IF
          IF (Location.AirExist = True) AND (MapX = Location.AirX) AND (MapY = Location.AirY) THEN
            CALL DispAir(5, XX + 1, YY + 1, 20)
          END IF
        END IF

        YY = YY + 20
      NEXT Y
        XX = XX + 20
        YY = IY
    NEXT X


END SUB

SUB DispScreenDown
DIM TempIcon(203) AS INTEGER
XI = 5
YI = 5
FOR T = 1 TO 4
XX = XI
YY = YI
XL = 0   'X Loop Counter
YL = 0   'X Loop Counter
  FOR Y = Location.Y + 4 TO Location.Y - 4 STEP -1
    IF Y <= 0 THEN
      MapY = Location.MapDimY + Y
     ELSEIF Y >= (Location.MapDimY + 1) THEN
      MapY = Y - Location.MapDimY
     ELSE
      MapY = Y
    END IF
    YL = YL + 1

    FOR X = Location.X - 5 TO Location.X + 5
      IF X <= 0 THEN
        MapX = Location.MapDimX + X
       ELSEIF X >= (Location.MapDimX + 1) THEN
        MapX = X - Location.MapDimX
       ELSE
        MapX = X
      END IF
      XL = XL + 1

      IF YL = 1 THEN
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (299, 0), MapIcons(1, Map(MapX, MapY, 1)), PSET
          GET (299, 0)-STEP(19, (T * 5) - 1), TempIcon
          PUT (XX, 180 + YI - (T * 5)), TempIcon(1), PSET
         ELSE
          LINE (XX, 180 + YI - (T * 5))-STEP(19, (T * 5) - 1), 0, BF
        END IF
          'FOR M = 1 TO 50000: NEXT M
      END IF
                  
      
      IF (YL > 1) AND (YL < 9) THEN
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (XX, YY), MapIcons(1, Map(MapX, MapY, 1)), PSET
         ELSE
          LINE (XX, YY)-STEP(19, 19), 0, BF
        END IF
        'The following IF Waits to put the new character on the screen
        'When the X Loop and Y loop are at a certain point
          IF (XL = 7) AND (YL = 6) THEN
            IF Location.Mode = 1 THEN
              IF Map(Location.X, Location.Y, 2) = 7 THEN
                CALL DispHero(1, T, XI + 100, YI + 80, Location.HalfSeeSize)
               ELSEIF Map(Location.X, Location.Y, 2) <> 8 THEN
                CALL DispHero(1, T, XI + 100, YI + 80, Location.FullSeeSize)
             
              END IF
             ELSE 'IF The mode is By Ship
              CALL DispBoat(1, T, XI + 100, YI + 80, Location.FullSeeSize)
            END IF
          END IF
        IF Location.MapName = "WORLD     " OR Location.MapName = "SAMPLE    " THEN
          IF (Location.ShipExist = True) AND (MapX = Location.ShipX) AND (MapY = Location.ShipY) THEN
            CALL DispBoat(1, 1, XX, YY, 20)
          END IF
          IF (Location.AirExist = True) AND (MapX = Location.AirX) AND (MapY = Location.AirY) THEN
            CALL DispAir(5, XX, YY, 20)
          END IF

        END IF

      END IF

      IF YL = 9 THEN
        GET (XX, YI + 5)-STEP(19, (4 - T) * 5 - 1), TempIcon
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (XX, YY), MapIcons(1, Map(MapX, MapY, 1)), PSET
         ELSE
          LINE (XX, YY)-STEP(19, 19), 0, BF
        END IF
        IF T < 3 THEN
          PUT (XX, YI), TempIcon(1), PSET
        END IF
          'FOR M = 1 TO 50000: NEXT M
      END IF

        XX = XX + 20
  
    NEXT X
      IF YL = 1 THEN
        YY = YI + 160 - (T * 5)
       ELSE
        YY = YY - 20
      END IF
    XX = XI
    XL = 0
   
  NEXT Y
'FOR M = 1 TO 30000: NEXT M
NEXT T
ERASE TempIcon


END SUB

SUB DispScreenLeft
DIM TempIcon(203) AS INTEGER
XI = 5
YI = 5
FOR T = 1 TO 4

XX = XI
YY = YI
XL = 0   'X Loop Counter
YL = 0   'Y Loop Counter

  FOR X = Location.X - 5 TO Location.X + 5
    IF X <= 0 THEN
      MapX = Location.MapDimX + X
     ELSEIF X >= (Location.MapDimX + 1) THEN
      MapX = X - Location.MapDimX
     ELSE
      MapX = X
    END IF
    XL = XL + 1
 
    FOR Y = Location.Y - 4 TO Location.Y + 4
      IF Y <= 0 THEN
        MapY = Location.MapDimY + Y
       ELSEIF Y >= (Location.MapDimY + 1) THEN
        MapY = Y - Location.MapDimY
       ELSE
        MapY = Y
      END IF
      YL = YL + 1
     
      IF XL = 1 THEN
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (299, 0), MapIcons(1, Map(MapX, MapY, 1)), PSET
          GET (299 + (4 - T) * 5, 0)-STEP((T * 5) - 1, 19), TempIcon
          PUT (XI, YY), TempIcon(1), PSET
         ELSE
          LINE (XI, YY)-STEP((T * 5) - 1, 19), 0, BF
        END IF
          'FOR M = 1 TO 50000: NEXT M
      END IF
                    
      IF (XL > 1) AND (XL < 11) THEN
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (XX, YY), MapIcons(1, Map(MapX, MapY, 1)), PSET
         ELSE
          LINE (XX, YY)-STEP(19, 19), 0, BF
        END IF
          IF (XL = 7) AND (YL = 6) THEN
            IF Location.Mode = 1 THEN
              IF Map(Location.X, Location.Y, 2) = 7 THEN
                CALL DispHero(2, T, XI + 100, YI + 80, Location.HalfSeeSize)
               ELSEIF Map(Location.X, Location.Y, 2) <> 8 THEN
                CALL DispHero(2, T, XI + 100, YI + 80, Location.FullSeeSize)
            
              END IF
             ELSE 'IF The mode is By Ship
              CALL DispBoat(2, T, XI + 100, YI + 80, Location.FullSeeSize)
            END IF
          END IF
        IF Location.MapName = "WORLD     " OR Location.MapName = "SAMPLE    " THEN
          IF (Location.ShipExist = True) AND (MapX = Location.ShipX) AND (MapY = Location.ShipY) THEN
            CALL DispBoat(1, 1, XX, YY, 20)
          END IF
          IF (Location.AirExist = True) AND (MapX = Location.AirX) AND (MapY = Location.AirY) THEN
            CALL DispAir(5, XX, YY, 20)
          END IF
        END IF

      END IF

      IF XL = 11 THEN
        GET (195 + (T * 5) + XI, YY)-STEP((4 - T) * 5 - 1, 19), TempIcon
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (XX, YY), MapIcons(1, Map(MapX, MapY, 1)), PSET
         ELSE
          LINE (XX, YY)-STEP(19, 19), 0, BF
        END IF
        IF T < 3 THEN
          PUT (200 + (T * 5) + XI, YY), TempIcon(1), PSET
        END IF
          'FOR M = 1 TO 50000: NEXT M
      END IF

      YY = YY + 20
      'XX = XI
      'XL = 0
     
    NEXT Y
    IF XL = 1 THEN
      XX = XI + (T * 5)
     ELSE
      XX = XX + 20
    END IF
    YL = 0
    YY = YI

  NEXT X
NEXT T
ERASE TempIcon

END SUB

SUB DispScreenRight
DIM TempIcon(203) AS INTEGER
XI = 5
YI = 5
FOR T = 1 TO 4
XX = XI
YY = YI
XL = 0   'X Loop Counter
YL = 0

  FOR X = Location.X + 5 TO Location.X - 5 STEP -1
    IF X <= 0 THEN
      MapX = Location.MapDimX + X
     ELSEIF X >= (Location.MapDimX + 1) THEN
      MapX = X - Location.MapDimX
     ELSE
      MapX = X
    END IF
    XL = XL + 1
   
    FOR Y = Location.Y - 4 TO Location.Y + 4
      IF Y <= 0 THEN
        MapY = Location.MapDimY + Y
       ELSEIF Y >= (Location.MapDimY + 1) THEN
        MapY = Y - Location.MapDimY
       ELSE
        MapY = Y
      END IF
      YL = YL + 1
     
      IF XL = 1 THEN
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (299, 0), MapIcons(1, Map(MapX, MapY, 1)), PSET
          GET (299, 0)-STEP((T * 5) - 1, 19), TempIcon
          PUT (220 + XI - (T * 5), YY), TempIcon(1), PSET
         ELSE
          LINE (220 + XI - (T * 5), YY)-STEP((T * 5) - 1, 19), 0, BF
        END IF
          'FOR M = 1 TO 50000: NEXT M
      END IF
                   
      IF (XL > 1) AND (XL < 11) THEN
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (XX, YY), MapIcons(1, Map(MapX, MapY, 1)), PSET
         ELSE
          LINE (XX, YY)-STEP(19, 19), 0, BF
        END IF
          IF (XL = 7) AND (YL = 6) THEN
            IF Location.Mode = 1 THEN
              IF Map(Location.X, Location.Y, 2) = 7 THEN
                CALL DispHero(4, T, XI + 100, YI + 80, Location.HalfSeeSize)
               ELSEIF Map(Location.X, Location.Y, 2) <> 8 THEN
                CALL DispHero(4, T, XI + 100, YI + 80, Location.FullSeeSize)
            
              END IF
             ELSE 'IF The mode is By Ship
              CALL DispBoat(4, T, XI + 100, YI + 80, Location.FullSeeSize)
            END IF
          END IF
        IF Location.MapName = "WORLD     " OR Location.MapName = "SAMPLE    " THEN
          IF (Location.ShipExist = True) AND (MapX = Location.ShipX) AND (MapY = Location.ShipY) THEN
            CALL DispBoat(1, 1, XX, YY, 20)
          END IF
          IF (Location.AirExist = True) AND (MapX = Location.AirX) AND (MapY = Location.AirY) THEN
            CALL DispAir(5, XX, YY, 20)
          END IF
       END IF

      END IF

      IF XL = 11 THEN
        GET (XI + 5, YY)-STEP((4 - T) * 5 - 1, 19), TempIcon
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (XX, YY), MapIcons(1, Map(MapX, MapY, 1)), PSET
         ELSE
          LINE (XX, YY)-STEP(19, 19), 0, BF
        END IF
        IF T < 3 THEN
          PUT (XI, YY), TempIcon(1), PSET
        END IF
          'FOR M = 1 TO 50000: NEXT M
      END IF
     
      YY = YY + 20
    NEXT Y
    IF XL = 1 THEN
      XX = XI + 200 - (T * 5)
     ELSE
      XX = XX - 20
    END IF
    YY = YI
    YL = 0
  NEXT X
'FOR M = 1 TO 30000: NEXT M
NEXT T
ERASE TempIcon


END SUB

SUB DispScreenUp
DIM TempIcon(203) AS INTEGER
XI = 5
YI = 5
FOR T = 1 TO 4

XX = XI
YY = YI
YL = 0   'Y Loop Counter
XL = 0
  FOR Y = Location.Y - 4 TO Location.Y + 4
    IF Y <= 0 THEN
      MapY = Location.MapDimY + Y
     ELSEIF Y >= (Location.MapDimY + 1) THEN
      MapY = Y - Location.MapDimY
     ELSE
      MapY = Y
    END IF
    YL = YL + 1

    FOR X = Location.X - 5 TO Location.X + 5
      IF X <= 0 THEN
        MapX = Location.MapDimX + X
       ELSEIF X >= (Location.MapDimX + 1) THEN
        MapX = X - Location.MapDimX
       ELSE
        MapX = X
      END IF
      XL = XL + 1

      IF YL = 1 THEN
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (299, 0), MapIcons(1, Map(MapX, MapY, 1)), PSET
          GET (299, 0 + (4 - T) * 5)-STEP(19, (T * 5) - 1), TempIcon
          PUT (XX, YI), TempIcon(1), PSET
         ELSE
          LINE (XX, YI)-STEP(19, (T * 5) - 1), 0, BF
        END IF
          'FOR M = 1 TO 50000: NEXT M
      END IF
                   
      IF (YL > 1) AND (YL < 9) THEN
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (XX, YY), MapIcons(1, Map(MapX, MapY, 1)), PSET
         ELSE
          LINE (XX, YY)-STEP(19, 19), 0, BF
        END IF
          IF (XL = 7) AND (YL = 6) THEN
            IF Location.Mode = 1 THEN
              IF Map(Location.X, Location.Y, 2) = 7 THEN
                CALL DispHero(3, T, XI + 100, YI + 80, Location.HalfSeeSize)
               ELSEIF Map(Location.X, Location.Y, 2) <> 8 THEN
                CALL DispHero(3, T, XI + 100, YI + 80, Location.FullSeeSize)
            
              END IF
             ELSE 'IF The mode is By Ship
              CALL DispBoat(3, T, XI + 100, YI + 80, Location.FullSeeSize)
            END IF
          END IF
        IF Location.MapName = "WORLD     " OR Location.MapName = "SAMPLE    " THEN
          IF (Location.ShipExist = True) AND (MapX = Location.ShipX) AND (MapY = Location.ShipY) THEN
            CALL DispBoat(1, 1, XX, YY, 20)
          END IF
          IF (Location.AirExist = True) AND (MapX = Location.AirX) AND (MapY = Location.AirY) THEN
            CALL DispAir(5, XX, YY, 20)
          END IF
        END IF

      END IF

      IF YL = 9 THEN
        GET (XX, 155 + (T * 5) + YI)-STEP(19, (4 - T) * 5 - 1), TempIcon
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (XX, YY), MapIcons(1, Map(MapX, MapY, 1)), PSET
         ELSE
          LINE (XX, YY)-STEP(19, 19), 0, BF
        END IF
        IF T < 3 THEN
          PUT (XX, 160 + (T * 5) + YI), TempIcon(1), PSET
        END IF
          'FOR M = 1 TO 50000: NEXT M
      END IF

      XX = XX + 20
    
    NEXT X
      IF YL = 1 THEN
        YY = YI + (T * 5)
       ELSE
        YY = YY + 20
      END IF
    XX = XI
    XL = 0
   
  NEXT Y  'Next YValue
'FOR M = 1 TO 30000: NEXT M
NEXT T
ERASE TempIcon
END SUB

SUB DispSideMenuDesc (Text$)
X = 160 - ((LEN(Text$) * 5) / 2)
LINE (0, 190)-STEP(320, 20), 20, BF
CALL TextFont(Text$, X, 193, 24, 0)
END SUB

SUB DispSmallStatusBox
LINE (250, 118)-STEP(60, 18 * NumberMembers), 24, B
LINE (251, 119)-STEP(58, 18 * NumberMembers - 2), 20, BF
IF NumberMembers >= 1 THEN
  CALL TextFont(Player(1).PName, 254, 123, 26, 0)
  CALL TextFont(" " + STR$(Player(1).HP) + "/" + MID$(STR$(Player(1).MaxHP), 2, LEN(STR$(Player(1).MaxHP))), 254, 130, 26, 0)
END IF

IF NumberMembers >= 2 THEN
  CALL TextFont(Player(2).PName, 254, 140, 26, 0)
  CALL TextFont(" " + STR$(Player(2).HP) + "/" + MID$(STR$(Player(2).MaxHP), 2, LEN(STR$(Player(2).MaxHP))), 254, 147, 26, 0)
END IF

IF NumberMembers >= 3 THEN
  CALL TextFont(Player(3).PName, 254, 157, 26, 0)
  CALL TextFont(" " + STR$(Player(3).HP) + "/" + MID$(STR$(Player(3).MaxHP), 2, LEN(STR$(Player(3).MaxHP))), 254, 164, 26, 0)
END IF

IF NumberMembers = 4 THEN
  CALL TextFont(Player(4).PName, 254, 174, 26, 0)
  CALL TextFont(" " + STR$(Player(4).HP) + "/" + MID$(STR$(Player(4).MaxHP), 2, LEN(STR$(Player(4).MaxHP))), 254, 181, 26, 0)
END IF

END SUB

SUB DispStats
SCREEN 13: CLS
PAINT (1, 1), 1
PlayLoc = 1
SizeX = 55
SizeY = 60
FOR T = 1 TO NumberMembers
  LINE ((T - 1) * SizeX, 0)-STEP(SizeX, SizeY), 20, B
NEXT T

GOSUB UpdateSelectorBox
DO
CALL GetKey(A$)
SELECT CASE A$
      CASE IS = CHR$(0) + CHR$(77) 'Right
         IF PlayLoc + 1 > NumberMembers THEN
             PlayLoc = 1
           ELSE
             PlayLoc = PlayLoc + 1
         END IF
        GOSUB UpdateSelectorBox

      CASE IS = CHR$(0) + CHR$(75) 'Left
         IF PlayLoc - 1 < 1 THEN
            PlayLoc = NumberMembers
          ELSE
            PlayLoc = PlayLoc - 1
         END IF
        GOSUB UpdateSelectorBox

END SELECT

LOOP UNTIL A$ = CHR$(27)
CLS
EXIT SUB

UpdateSelectorBox:
FOR T = 1 TO NumberMembers
  IF T = PlayLoc THEN
    LINE ((T - 1) * SizeX + 1, 1)-STEP(SizeX - 2, SizeY - 2), 28, B
   ELSE
    LINE ((T - 1) * SizeX + 1, 1)-STEP(SizeX - 2, SizeY - 2), 21, B
  END IF
NEXT T
LINE (0, 70)-STEP(320, 120), 1, BF
'Displays Stats
CALL TextFont("Name: ", 10, 70, 23, 0)
CALL TextFont(Player(PlayLoc).PName, 40, 70, 32, 0)
CALL TextFont("Gold: ", 170, 70, 23, 0)
CALL TextFont(STR$(Player(1).Gold), 190, 70, 23, 0)
CALL TextFont("Level: " + MID$(STR$(Player(PlayLoc).Level), 2, (LEN(STR$(Player(PlayLoc).Level)) - 1)), 20, 90, 23, 0)
IF Player(PlayLoc).HP > 0 THEN
   CALL TextFont("HP: " + MID$(STR$(Player(PlayLoc).HP), 2, (LEN(STR$(Player(PlayLoc).HP)) - 1)) + "/" + MID$(STR$(Player(PlayLoc).MaxHP), 2, (LEN(STR$(Player(PlayLoc).MaxHP)) - 1)), 100, 90, 23, 0)
 ELSE
   CALL TextFont("HP: " + MID$(STR$(Player(PlayLoc).HP), 2, (LEN(STR$(Player(PlayLoc).HP)) - 1)) + "/" + MID$(STR$(Player(PlayLoc).MaxHP), 2, (LEN(STR$(Player(PlayLoc).MaxHP)) - 1)), 100, 90, 4, 0)
END IF
CALL TextFont("MP: " + MID$(STR$(Player(PlayLoc).MP), 2, (LEN(STR$(Player(PlayLoc).MP)) - 1)) + "/" + MID$(STR$(Player(PlayLoc).MaxMP), 2, (LEN(STR$(Player(PlayLoc).MaxMP)) - 1)), 180, 90, 23, 0)
CALL TextFont("Defense: " + MID$(STR$(Player(PlayLoc).BaseDefense), 2, (LEN(STR$(Player(PlayLoc).BaseDefense)) - 1)), 20, 110, 23, 0)
CALL TextFont("Strength: " + MID$(STR$(Player(PlayLoc).Strength), 2, (LEN(STR$(Player(PlayLoc).Strength)) - 1)), 20, 120, 23, 0)
CALL TextFont("Speed: " + MID$(STR$(Player(PlayLoc).Speed), 2, (LEN(STR$(Player(PlayLoc).Speed)) - 1)), 20, 130, 23, 0)

CALL TextFont("Total Exp: " + MID$(STR$(Player(PlayLoc).XP), 2, (LEN(STR$(Player(PlayLoc).XP)) - 1)), 20, 150, 24, 0)
ExpNeeded = Player(PlayLoc).XPNext - Player(PlayLoc).XP
CALL TextFont("Exp Needed: " + MID$(STR$(ExpNeeded), 2, (LEN(STR$(ExpNeeded)) - 1)), 20, 160, 24, 0)
CALL TextFont("Next Level at: " + MID$(STR$(Player(PlayLoc).XPNext), 2, (LEN(STR$(Player(PlayLoc).XPNext)) - 1)), 20, 170, 24, 0)


RETURN
END SUB

SUB MainSideMenu (Option$, Done%)
Triggered = False
Key$ = Option$
SELECT CASE Option$
   CASE IS = CHR$(0) + CHR$(33) 'Alt F
      SelLoc = 1
      Triggered = True
   
   CASE IS = "F"
      SelLoc = 1
  
   CASE IS = CHR$(0) + CHR$(31) 'Alt S
      SelLoc = 2
      Triggered = True
  
   CASE IS = "S"
      SelLoc = 2

 
   CASE IS = CHR$(0) + CHR$(23) 'Alt I
      SelLoc = 3
      Triggered = True
  
   CASE IS = "I"
      SelLoc = 3
 
   CASE IS = CHR$(0) + CHR$(50) 'Alt M
      SelLoc = 4
      Triggered = True
  
   CASE IS = "M"
      SelLoc = 1

END SELECT
GOSUB UpdateScrBar
DoneMenu = 0
DO
  IF (Triggered = False) THEN CALL GetKey(Key$)
  IF (Key$ = CHR$(13)) OR (Key$ = CHR$(0) + CHR$(77)) THEN Key$ = MID$(STR$(SelLoc), 2, 1)
  SELECT CASE Key$
    CASE IS = "Q", "5", CHR$(0) + CHR$(16) 'Alt Q
      Done% = True
      Key$ = CHR$(27)
   
    CASE IS = "I", "3", CHR$(0) + CHR$(23)'Alt I
      CALL Inventory
      DoneMenu = 1

    CASE IS = "S", "2", CHR$(0) + CHR$(31)  'Alt S
      CALL DispStats
      DoneMenu = 1

    CASE IS = "M", "4", CHR$(0) + CHR$(50)'Alt M
      CALL MainScreenMagic
      DoneMenu = 1

    CASE IS = "F", "1", CHR$(0) + CHR$(33)'Alt F
      CALL FileScreen
      DoneMenu = 1

    CASE IS = CHR$(0) + CHR$(80) 'Down
      IF SelLoc = 5 THEN
          SelLoc = 1
        ELSE
          SelLoc = SelLoc + 1
      END IF
      GOSUB UpdateScrBar
      
    CASE IS = CHR$(0) + CHR$(72) 'Up
      IF SelLoc = 1 THEN
          SelLoc = 5
        ELSE
          SelLoc = SelLoc - 1
      END IF
      GOSUB UpdateScrBar


  END SELECT


LOOP UNTIL (DoneMenu = 1) OR (Key$ = CHR$(27)) OR (Key$ = CHR$(0) + CHR$(75))'ESC or LEFT
'LINE (240, 10)-(319, 80), 19, BF

SCREEN 13: CLS
CALL DispMainScreen
CALL DispScreen
CALL DispHero(Location.Dir, 4, XI + 105, YI + 85, Location.FullSeeSize)

EXIT SUB  'Leaves Sub

UpdateScrBar: 'Gosub Label
FOR T = 1 TO 5
  LINE (245, T * 10)-(300, T * 10 + 9), 19, B
NEXT T
LINE (245, SelLoc * 10)-(300, SelLoc * 10 + 9), 24, B
SELECT CASE SelLoc
  CASE 1
    Message$ = "Save/Load/Options/Help"
  CASE 2
    Message$ = "Displays Player Stats"
  CASE 3
    Message$ = "Shows Players inventory"
  CASE 4
    Message$ = "Use/View Magic"
  CASE 5
    Message$ = "Exits Game"


END SELECT
CALL DispSideMenuDesc(Message$)
RETURN


END SUB

SUB MessBox (Text$)
MS! = Env.MS   'Convert Integer to Single
FGC! = Env.FGC  'Converts integer to Single
CALL TextFont(Text$, 10, (Env.ML * 10) + 10, FGC, MS)
IF Env.ML = 5 THEN     'Sets the next message line
  PSET (217, 63), Env.FGC: DRAW "d2r3gehf"
  CALL PAUSE
 ELSE
  Env.ML = Env.ML + 1
END IF
END SUB

SUB ResetMessBox (Heading$)
'Displays the message box, as well as the heading
LINE (8, 8)-STEP(212, 58), Env.BGC, BF
LINE (7, 17)-STEP(214, 0), 23
LINE (7, 67)-STEP(214, 0), 22
LINE (221, 7)-STEP(0, 60), 22
LINE (7, 7)-STEP(214, 0), 25
LINE (7, 7)-STEP(0, 60), 25
FGC! = Env.FGC 'change from Integer to SINGLE
CALL TextFont(Heading$, 10, 10, FGC!, 0)
Env.ML = 1

END SUB

SUB TextFont (S$, X, Y, C, S)
'SYNTAX: CALL TEXTFONT ("MESSAGE", X, Y, COLOR,Speed(0-9))
'X AND Y are coordinate locations
'
Y = Y + 4
DRAW "BM=" + VARPTR$(X) + ",=" + VARPTR$(Y) + "C=" + VARPTR$(C)

FOR T = 1 TO LEN(S$)
SELECT CASE UCASE$(MID$(S$, T, 1))

     CASE IS = "A"
          DRAW "u4r1f1d2l1r1d1br3"
     CASE IS = "B"
          DRAW "u4r1f1g1f1d1l2br5"
     CASE IS = "C"
          DRAW "u4r2l2d4r2br3"
     CASE IS = "D"
          DRAW "u4r1f1d2g1l1br5"
     CASE IS = "E"
          DRAW "u4r2l2d2r2l2d2r2br3"
     CASE IS = "F"
          DRAW "u4r2l2d2r1l1d2br5"
     CASE IS = "G"
          DRAW "u4r3l3d4r3u2l1r1d2br3"
     CASE IS = "H"
          DRAW "u4d2r2u2d4br3"
     CASE IS = "I"
          DRAW "r2l1u4r1l2r1d4r1br3"
     CASE IS = "J"
          DRAW "u1d1r2u4d4br3"
     CASE IS = "K"
          DRAW "u4d2r1e2g2f2br3"
     CASE IS = "L"
          DRAW "u4d4r2br3"
     CASE IS = "M"
          DRAW "u4r1f1d1u1e1r1d4br3"
     CASE IS = "N"
          DRAW "u4f2d1f1r1u4d4br3"
     CASE IS = "O", "0"
          DRAW "u4r2d4l2r2br3"
     CASE IS = "P"
          DRAW "u4r2d2l1bd1bf1br3"
     CASE IS = "Q"
          DRAW "u4r2d4l2r3br2"
     CASE IS = "R"
          DRAW "u4r2d2l1d1f1br3"
     CASE IS = "S", "5"
          DRAW "r2u2l2u2r2bd4br3"
     CASE IS = "T"
          DRAW "br1u4l1r2bd4br3"
     CASE IS = "U"
          DRAW "u4d4r2u4d4br3"
     CASE IS = "V"
          DRAW "br1u1l1u3d3r2u3bd4br3"
     CASE IS = "W"
          DRAW "u4d4r1e1f1r1u4d4br3"
     CASE IS = "X"
          DRAW "e1u2h1f1r1e1g1d2f1br3 "
     CASE IS = "Y"
          DRAW "br1u3r1u1d1l2u1d1r1d3br4"
     CASE IS = "Z"
          DRAW "r2l2u1e2u1l2bd4br5"
     CASE IS = " "
          DRAW "br4"
     CASE IS = "!"
          DRAW "S1br1ubu10u10d10bd10d1S4br3 "
     CASE IS = "."
          DRAW "S1br1rS4br2"
     CASE IS = "("
          DRAW "br2h1u2e1bd4br3"
     CASE IS = ")"
          DRAW "e1u2h1bd4br5"
     CASE IS = ":"
          DRAW "s1bu5ubu10udbd10dbd5s4br4"
     CASE IS = "["
          DRAW "u4rld4rbr3"
     CASE IS = "]"
          DRAW "ru4lrd4br3"
     CASE IS = "/"
          DRAW "e4bd4br3"
     CASE IS = ","
          DRAW "br1gebr4"
     CASE IS = "'"
          DRAW "bu3ubr2dbd3br3"
     CASE IS = "|"
          DRAW "ubu2ubd4Br3"
     CASE IS = "_"
          DRAW "r2br2"
     CASE IS = "?"
          DRAW "br2s1uS2bu5s4ehlbd4br5 "
     CASE IS = "#"
          DRAW "bru4br2d4bel4bu2r4bd3br3"
     CASE IS = "1"
          DRAW "r1u4gr1d3r1br3"
     CASE IS = "2"
          DRAW "br2l2ue2hlbd4br5"
     CASE IS = "3"
          DRAW "rehehlbd4br5"
     CASE IS = "4"
          DRAW "br2u2rl3u2d2r2u2bd4br4"
     CASE IS = "6"
          DRAW "r2u2l2u2r2l2d4br5"
     CASE IS = "7"
          DRAW "br2u4l2r2d4br3"
     CASE IS = "8"
          DRAW "r2u2l2u2r2d2l2d2br5"
     CASE IS = "9"
          DRAW "br2u2l2u2r2d4br3"
     CASE IS = "*"
          DRAW "be1e2gfh2bf3br2"
     CASE IS = "~"  'BACKSPACE
          DRAW "u4rd4ru4rd4ru4"
END SELECT
IF S >= 1 AND UCASE$(MID$(S$, T, 1)) <> " " THEN
  FOR M = 1 TO (1000 * (9 - Env.MS)): NEXT M
END IF
NEXT T

END SUB