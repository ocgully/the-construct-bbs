DECLARE SUB DispScreen ()
COMMON SHARED WorldX, WorldY
'$DYNAMIC
CONST MMIPATH$ = "C:\BG_RPG\MMI\"
CONST IconList$ = "MMIFILES.TXT"
CONST MapPath$ = "C:\BG_RPG\Maps\"
CONST True = 1
CONST False = 0
CONST MaxIcons = 410
CONST Ver$ = "Ver 1.1a"
CONST Right = 1
CONST Left = 2
CONST UP = 3
CONST Down = 4

                         'Types

                       
                         'Arrays
DIM SHARED Icons(MaxIcons) AS STRING
DIM SHARED ActiveIcons(203, 5) AS INTEGER
DIM SHARED SelectedIcon(203) AS INTEGER
DIM SHARED BlockAttributes(10, 9) AS INTEGER
DIM SHARED ActiveIconsAttrib(5) AS INTEGER
DIM SHARED PreLoadedIcons(203, MaxIcons) AS INTEGER
DIM SHARED PreLoadedIconsA(MaxIcons) AS INTEGER
DIM SHARED ICON(20, 20) AS INTEGER  'Pixelated
DIM SHARED Map(1, 1, 1)AS INTEGER

                       
                         'Initial Assignments

'CALL OpeningScreen
TotalIcons = 0
OPEN MMIPATH$ + IconList$ FOR INPUT AS #1
  WHILE NOT EOF(1)
    TotalIcons = TotalIcons + 1
    INPUT #1,Icons(TotalIcons)   'Loads a List of the MMI filenames
  WEND
CLOSE #1
LOCATE 1, 1: PRINT "Loading %0"
'Loads the Selected Icons into a File for Quick Retrieval
FOR X = 1 TO TotalIcons
 OPEN MMIPATH$ + Icons(X) FOR INPUT AS #1
  FOR Y = 1 TO 203
    INPUT #1,PreLoadedIcons(Y, X)
  NEXT Y
    INPUT #1,PreLoadedIconsA(X)  'Icon Attribute
 CLOSE #1
 LOCATE 1, 10: PRINT INT((X / TotalIcons) * 100)
NEXT X

FOR X = 1 TO 203
SelectedIcon(X) = PreLoadedIcons(X, 1)
NEXT X
SelectedIconAttrib = PreLoadedIconsA(1)
'***
'***
'*** Begining of program
'***
'***
SCREEN 13

OPEN "C:\BG_RPG\MAPS\CAVE2.MMM" FOR INPUT AS #1
      INPUT #1,MapName$   'Loads MapName From First Line
      INPUT #1,MapDimX    'Loads MapDimX From second Line
      INPUT #1,MapDimY    'Loads MapDimY From 3rd Line
      INPUT #1,Enemies    'Loads if enemies will be on current map
      REDIM Map(MapDimX, MapDimY, 2) AS INTEGER
      FOR X = 1 TO MapDimX
        FOR Y = 1 TO MapDimY
          INPUT #1,Map(X, Y, 1)  'Alternates Between the Icon # and the
          INPUT #1,Map(X, Y, 2)  'Attrib for each coordinate of the map
        NEXT Y
      NEXT X
    CLOSE #1
    WorldX = INT(MapDimX / 2)
    WorldY = INT(MapDimY / 2)

CLOSE #1
CALL DispScreen
PRINT FRE(-1)

DEFSNG A-Z
SUB DispScreen

XX = Grid
YY = Grid
'  This can be used to place an icon in a location when the coordinates are
'  (XX,YY) on Screen Grid.

    FOR X = WorldX - 5 TO WorldX + 5
        IF X <= 0 THEN
          MapX = MapDimX + X
        ELSEIF X >= (MapDimX + 1) THEN
            MapX = X - MapDimX
          ELSE
            MapX = X
        END IF
  
      FOR Y = WorldY - 4 TO WorldY + 4
          IF Y <= 0 THEN
            MapY = MapDimY + Y
          ELSEIF Y >= (MapDimY + 1) THEN
              MapY = Y - MapDimY
            ELSE
              MapY = Y
          END IF
    
    
        IF Map(MapX, MapY, 1) <> 0 THEN
           PUT (XX + 1, YY + 1), PreLoadedIcons(1, Map(MapX, MapY, 1)), PSET
         ELSE
           LINE (XX + 1, YY + 1)-STEP(19, 19), 0, BF
        END IF

        YY = YY + 20 + Grid
      NEXT Y
        XX = XX + 20 + Grid
        YY = Grid
    NEXT X


IF Grid = False THEN

  FOR Y = 1 TO 178 STEP 20
    IF (5 - WorldY = INT(Y / 20)) OR (MapDimY - WorldY + 5 = INT(Y / 20)) THEN
     IF WorldY = 1 THEN
       LINE (1, Y - 1)-(219, Y - 1), 9
      ELSE
       LINE (1, Y)-(219, Y), 9
     END IF
    END IF
  NEXT Y

  FOR X = 1 TO 220 STEP 20  'Redraws Blue line after it gray Y lines go trough it
   IF (6 - WorldX = INT(X / 20)) OR (MapDimX - WorldX + 6 = INT(X / 20)) THEN
     IF WorldX = 1 THEN
       LINE (X - 1, 1)-(X - 1, 180), 9
     ELSE
       LINE (X, 1)-(X, 180), 9
     END IF
   END IF
  NEXT X

END IF


END SUB

SUB ScrollScreen
END SUB