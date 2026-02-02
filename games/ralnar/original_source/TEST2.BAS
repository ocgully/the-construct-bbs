DECLARE SUB DispScreenDown ()
DECLARE SUB DispScreenUp ()
DECLARE SUB DispScreenRight ()
DECLARE SUB DispScreenLeft ()
OPTION BASE 1   'Makes starting points for Arrays 1 instead of 0
DECLARE SUB DispText ()
DECLARE SUB Help (HelpMode AS INTEGER)
DECLARE SUB WalkThrough ()
DECLARE SUB UpdateIcon ()
DECLARE SUB ClearIcon (Initial!)
DECLARE SUB LoadIcon ()
DECLARE SUB SaveIcon ()
DECLARE SUB SelectCol ()
DECLARE SUB MAKEPICBOX ()
DECLARE SUB MAKEGRID ()
DECLARE SUB DRAWIN (X!, Y!)
DECLARE SUB IconMaker (S$)


'From HERE UP are ICONMAKER Subs

DECLARE SUB DispPic ()
DECLARE SUB OpeningScreen ()
DECLARE SUB ChangeScreenDim (Apply!, TMapDimX!, TMapDimY!)
DECLARE SUB FlipH (IM AS INTEGER)
DECLARE SUB IconToGet ()
DECLARE SUB FlipV (IM AS INTEGER)
DECLARE SUB Rotate (IM AS INTEGER)
DECLARE SUB GetToIcon ()
DECLARE SUB LoadMap ()
DECLARE SUB PrintAttrib (IM AS INTEGER)
DECLARE SUB ClearPic ()
DECLARE SUB Textfont (S$, X!, Y!, C!, S!)
DECLARE SUB RefreshIcons ()
DECLARE SUB IconSelection ()
DECLARE SUB DispScreen ()
DECLARE SUB SaveMap ()
'$DYNAMIC
'This program is designed to create scrolling screen maps to be used
'for an RPG game.

'*** Declarations

COMMON SHARED OXCorr, XCorr, OYCorr, YCorr, CHANGE.CORR, Attrib$, IconAttribute AS INTEGER, Col AS INTEGER
'^^^ Icon Maker?
COMMON SHARED IconLoc, TotalIcons, ArrowLoc, Initial, Done, GrlocX, GrLocY, SelectedIconAttrib, Grid, Enemies
COMMON SHARED MapDimX, MapDimY, MapName$, WorldX, WorldY, IconAltered, MKLock
'^^^ Map Maker
                        
                         'Const
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

                        
                         'Initial Assignments

CALL OpeningScreen
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

IF TotalIcons + 25 >= MaxIcons THEN
     SCREEN 9: CLS
     'BEEP: BEEP: COLOR 4
     PRINT "Warning!! TotalIcons nearing MaxIcons!": PRINT
     PRINT "Change CONST MaxIcons to a higher number to prevent"
     PRINT "program crashing when TotalIcons passes MaxIcons!"
     COLOR 2
     PRINT "TotalIcons: "; TotalIcons, "MaxIcons: "; MaxIcons
     COLOR 7
     PRINT: INPUT "Press ENTER to Continue...", Con$
END IF

FOR X = 1 TO 203
SelectedIcon(X) = PreLoadedIcons(X, 1)
NEXT X
SelectedIconAttrib = PreLoadedIconsA(1)
'***
'***
'*** Begining of program
'***
'***
DO
ArrowLoc = 1
Enemies = False
IconLoc = 1
Grid = False  'Screen Grid On
GrlocX = 5
GrLocY = 4
MKLock = False 'Map Maker Key Lock Off

MenuDone = False
SCREEN 9: CLS
COLOR 3
PRINT "The input entered here is assumed to be correct, any input that"
PRINT "is incorrect can hault the program or cause the program to not"
PRINT "function properly."
COLOR 7
DO
LOCATE 5, 1: PRINT " )ew Map"
LOCATE 6, 1: PRINT " )oad Old Map"
LOCATE 7, 1: PRINT " )uit"
COLOR 5
LOCATE 5, 1: PRINT "N"
LOCATE 6, 1: PRINT "L"
LOCATE 7, 1: PRINT "Q"
COLOR 7
LOCATE 9, 1: PRINT "Enter Option:    "

  DO      'Repeats until a key is pressed
    Option$ = INKEY$
    IF Option$ <> "" THEN
      Option$ = UCASE$(Option$)
    END IF
  LOOP UNTIL Option$ <> ""

LOOP UNTIL Option$ = "L" OR Option$ = "N" OR Option$ = "Q" OR Option$ = "D"

SELECT CASE Option$  'Checks for quiting, loading, or creating
  
     CASE IS = "Q"
          Done = True

     CASE IS = "L"
          GOSUB LoadOldMap
         
     CASE IS = "N"
          GOSUB SetNewMap

     CASE IS = "D" 'Debug/Default Bypass
          MapDimX = 50
          MapDimY = 50
          Enemies = True
          DIM SHARED Map(MapDimX, MapDimY, 2) AS INTEGER
          WorldX = INT(MapDimX / 2)
          WorldY = INT(MapDimY / 2)
          '^^ ^^^^^^ ^^^ Makes Array for screen by the new dimensions
          FOR X = 1 TO MapDimX
               FOR Y = 1 TO MapDimY
                    Map(X, Y, 1) = 0
                    Map(X, Y, 2) = 1
               NEXT Y
          NEXT X
          MapName$ = "Default"
          GOSUB MainMapMaker

END SELECT 'Option$
LOOP UNTIL Option$ = "Q"

END 'End of Program

MainMapMaker:    'Gosub Label
SCREEN 13: CLS
Done = False
CALL DispScreen
CALL IconSelection
LINE (239, 165)-(280, 174), 0, BF
LINE (239, 20)-(319, 30), 0, BF
CALL DispText

DO
Changed = False
ChangedGr = False '??
'LINE (5 * (20 + Grid) + Grid, 4 * (20 + Grid) + Grid)-STEP(21, 21), 15, B
A$ = INKEY$
SELECT CASE A$
       CASE IS = CHR$(0) + "H"    'UP
         IF WorldY = 1 THEN
            WorldY = MapDimY
           ELSE
             WorldY = WorldY - 1
          END IF
          'IF MKLock = True THEN GOSUB LockOn:
          CALL DispScreenUp

      CASE IS = CHR$(0) + "P"    'DOWN
          IF WorldY = MapDimY THEN
             WorldY = 1
            ELSE WorldY = WorldY + 1
          END IF
          'IF MKLock = True THEN GOSUB LockOn:
          CALL DispScreenDown
    
      CASE IS = CHR$(0) + "K"    'LEFT
         IF WorldX = 1 THEN
            WorldX = MapDimX
           ELSE
             WorldX = WorldX - 1
          END IF
          'IF MKLock = True THEN GOSUB LockOn
          CALL DispScreenLeft
    
      CASE IS = CHR$(0) + "M"    'RIGHT
          IF WorldX = MapDimX THEN
             WorldX = 1
            ELSE WorldX = WorldX + 1
          END IF
           'IF MKLock = True THEN GOSUB LockOn:
          CALL DispScreenRight
           

    CASE IS = "R", "r"
      CALL GetToIcon
      CALL Rotate(False)
      CALL IconToGet
      IconAltered = True
    
    CASE IS = "h", "H" 'Horizontal flip
      CALL GetToIcon
      CALL FlipH(False)
      CALL IconToGet
      IconAltered = True
   
    CASE IS = "V", "v"
      CALL GetToIcon
      CALL FlipV(False)
      CALL IconToGet
      IconAltered = True

    CASE IS = CHR$(0) + ";"  'F1
       CALL Help(1)

    CASE IS = CHR$(0) + "<"   'F2
      LINE (239, 20)-(319, 30), 0, BF
      IF Enemies = True THEN
          Enemies = False
          CALL Textfont("No Enemies", 240, 25, 25, 0)
        ELSE
          Enemies = True
          CALL Textfont("Enemies On", 240, 25, 25, 0)
      END IF

   
    CASE IS = CHR$(0) + "="    'F3
       'LINE (1, 1)-(232, 190), 0, BF
       'IF Grid = 1 THEN Grid = 0 ELSE Grid = 1
       'CALL DispScreen

    CASE IS = "I", "i"
       CALL IconSelection

    CASE IS = " "
           PUT ((GrlocX * (20 + Grid) + 1 + Grid), (GrLocY * (20 + Grid) + 1 + Grid)), SelectedIcon, PSET
        ' For Transparent:
           IF Map(WorldX, WorldY, 1) <> 0 THEN
           DIM TIcon(203) AS INTEGER  'Temporary allocation
               FOR TX = 1 TO 203
                 TIcon(TX) = PreLoadedIcons(TX, Map(WorldX, WorldY, 1))
               NEXT TX
           PUT (240, 60), TIcon(1), PSET
           FOR X = 1 TO 20  ' ^^^ Haven't yet stored new icon in Pic.
             FOR Y = 1 TO 20
               IF POINT((GrlocX * (20 + Grid) + Grid) + X, (GrLocY * (20 + Grid) + Grid) + Y) = 0 THEN
                  C! = POINT(239 + X, 59 + Y)
                  PSET ((GrlocX * (20 + Grid) + Grid) + X, (GrLocY * (20 + Grid) + Grid) + Y), C!
               END IF
             NEXT Y
           NEXT X 'Does Transparency
          
           'Begining block compare
           DIM Found(1) AS INTEGER, TCount(2) AS INTEGER, TempIcon(203)AS INTEGER
           Found(1) = False: TCount(1) = 0:
           GET ((GrlocX * (20 + Grid) + 1 + Grid), (GrLocY * (20 + Grid) + 1 + Grid))-STEP(19, 19), TempIcon(1)
           DO
           TCount(1) = TCount(1) + 1
              
               Found(1) = 1: TCount(2) = 0
               DO
                 TCount(2) = TCount(2) + 1
                 TempNum = PreLoadedIcons(TCount(2), TCount(1))
                 IF TempNum <> TempIcon(TCount(2)) THEN Found(1) = 0
               LOOP UNTIL Found(1) = 0 OR TCount(2) = 203
               
           LOOP UNTIL (Found(1) = 1) OR TCount(1) = TotalIcons

           IF Found(1) = 1 THEN
                 Map(WorldX, WorldY, 1) = TCount(1)
                 Map(WorldX, WorldY, 2) = PreLoadedIconsA(TCount(1))
                 
              ELSE 'If The Icon created is a totally new one
                 TotalIcons = TotalIcons + 1
                 Icons(TotalIcons) = MID$(STR$(TotalIcons), 2, LEN(STR$(TotalIcons)) - 1) + ".MMI"
                 PATHNAME$ = MMIPATH$ + MID$(STR$(TotalIcons), 2, LEN(STR$(TotalIcons)) - 1) + ".MMI"
                 OPEN PATHNAME$ FOR OUTPUT AS #1
                   FOR X = 1 TO 203
                     WRITE #1, TempIcon(X)
                     PreLoadedIcons(X, TotalIcons) = TempIcon(X)
                   NEXT X
                     IF SelectedIconAttrib <> 6 THEN
                          WRITE #1, SelectedIconAttrib
                          PreLoadedIconsA(TotalIcons) = SelectedIconAttrib
                        ELSE
                          WRITE #1, Map(WorldX, WorldY, 2)
                          PreLoadedIconsA(TotalIcons) = Map(WorldX, WorldY, 2)
                     END IF
                     
                 CLOSE #1
                 Icons(TotalIcons) = MID$(STR$(TotalIcons), 2, LEN(STR$(TotalIcons)) - 1) + ".MMI"
                 'PRINT "Made New Icon"
                 OPEN MMIPATH$ + IconList$ FOR APPEND AS #1
                   PRINT #1, (MID$(STR$(TotalIcons), 2, LEN(STR$(TotalIcons)) - 1) + ".MMI")
                 CLOSE #1
                 Map(WorldX, WorldY, 1) = TotalIcons
                 IF SelectedIconAttrib <> 6 THEN
                    Map(WorldX, WorldY, 2) = SelectedIconAttrib
                 END IF
                 

          
           END IF
           ERASE Found, TCount, TempIcon
           PUT (240, 60), TIcon(1), XOR
           'GET ((GrLocX * (20 + Grid) + 1 + Grid), (GrLocY * (20 + Grid) + 1 + Grid))-STEP(19, 19), Pic(1, GrLocX + 1, GrLocY + 1)
        '  ^^^ Stores in array
        ' End of Tranparent
          ERASE TIcon  'Free's first Temp Icon
          END IF

          'If Icon has been rotated or changed The following will handle it
          IF (IconAltered = True) AND Map(WorldX, WorldY, 1) = 0 THEN
            DIM Found(1) AS INTEGER, TCount(2) AS INTEGER, TempIcon(203) AS INTEGER
            Found(1) = False: TCount(1) = 0:
            GET ((GrlocX * (20 + Grid) + 1 + Grid), (GrLocY * (20 + Grid) + 1 + Grid))-STEP(19, 19), TempIcon(1)
            DO
              TCount(1) = TCount(1) + 1
              Found(1) = 1: TCount(2) = 0
              DO
                TCount(2) = TCount(2) + 1
                TempNum = PreLoadedIcons(TCount(2), TCount(1))
                IF TempNum <> TempIcon(TCount(2)) THEN Found(1) = 0
              LOOP UNTIL Found(1) = 0 OR TCount(2) = 203
               
            LOOP UNTIL (Found(1) = 1) OR TCount(1) = TotalIcons

              IF Found(1) = 1 THEN
                 Map(WorldX, WorldY, 1) = TCount(1)
                 Map(WorldX, WorldY, 2) = PreLoadedIconsA(TCount(1))
              ELSE 'If The Icon created is a totally new one
                 TotalIcons = TotalIcons + 1
                 Icons(TotalIcons) = MID$(STR$(TotalIcons), 2, LEN(STR$(TotalIcons)) - 1) + ".MMI"
                 PATHNAME$ = MMIPATH$ + MID$(STR$(TotalIcons), 2, LEN(STR$(TotalIcons)) - 1) + ".MMI"
                 OPEN PATHNAME$ FOR OUTPUT AS #1
                   FOR X = 1 TO 203
                     WRITE #1, TempIcon(X)
                     PreLoadedIcons(X, TotalIcons) = TempIcon(X)
                   NEXT X
                     IF SelectedIconAttrib <> 6 THEN
                          WRITE #1, SelectedIconAttrib
                          PreLoadedIconsA(TotalIcons) = SelectedIconAttrib
                        ELSE
                          WRITE #1, Map(WorldX, WorldY, 2)
                          PreLoadedIconsA(TotalIcons) = Map(WorldX, WorldY, 2)
                     END IF
                   
                 Icons(TotalIcons) = MID$(STR$(TotalIcons), 2, LEN(STR$(TotalIcons)) - 1) + ".MMI"
                 CLOSE #1
                 'PRINT "Made New Icon"
                 OPEN MMIPATH$ + IconList$ FOR APPEND AS #1
                   PRINT #1, (MID$(STR$(TotalIcons), 2, LEN(STR$(TotalIcons)) - 1) + ".MMI")
                 CLOSE #1
                 Map(WorldX, WorldY, 1) = TotalIcons
                 IF SelectedIconAttrib <> 6 THEN
                    Map(WorldX, WorldY, 2) = SelectedIconAttrib
                 END IF
                 
              END IF
             ERASE Found, TCount, TempIcon
             END IF '(IconAltered = True)
          
           'If the icon is being plaed over a blank space and is in original form
           IF (IconAltered = False) AND (Map(WorldX, WorldY, 1) = 0) THEN
             Map(WorldX, WorldY, 1) = IconLoc
             Map(WorldX, WorldY, 2) = SelectedIconAttrib
           END IF
           CALL PrintAttrib(False)
           SOUND 56, .5
   
    CASE IS = "d", "D" 'Change Dimensions
       Apply = False
       CALL ChangeScreenDim(Apply, TMapDimX, TMapDimY)
       IF Apply = True THEN
          PATHNAME$ = MapPath$ + "Temp.MMM"
          OPEN PATHNAME$ FOR OUTPUT AS #1
          WRITE #1, MapName$  'Saves MapName On First Line
          WRITE #1, MapDimX   'Saves MapDimX on second Line
          WRITE #1, MapDimY   'Saves MapDimY on 3rd Line
            FOR X = 1 TO MapDimX
              FOR Y = 1 TO MapDimY
                WRITE #1, Map(X, Y, 1) 'Alternates Between the Icon # and the
                WRITE #1, Map(X, Y, 2) 'Attrib for each coordinate of the map
              NEXT Y
            NEXT X
          CLOSE #1

          REDIM SHARED Map(TMapDimX, TMapDimY, 2) AS INTEGER

          OPEN PATHNAME$ FOR INPUT AS #1
            INPUT #1,MapName$
            INPUT #1,MapDimX
            INPUT #1,MapDimY
            FOR X = 1 TO MapDimX
              FOR Y = 1 TO MapDimY
                INPUT #1,Map(X, Y, 1)
                INPUT #1,Map(X, Y, 2)
              NEXT Y
            NEXT X
          CLOSE #1
            IF (MapDimX <> TMapDimX) XOR (MapDimY <> TMapDimY) THEN
                IF MapDimX = TMapDimX THEN
                    FOR X = 1 TO MapDimX
                      FOR Y = MapDimY TO TMapDimY
                        Map(X, Y, 1) = 0
                        Map(X, Y, 2) = 1
                      NEXT Y
                    NEXT X
                  ELSE
                    FOR X = MapDimX TO TMapDimX
                      FOR Y = 1 TO MapDimY
                        Map(X, Y, 1) = 0
                        Map(X, Y, 2) = 1
                      NEXT Y
                    NEXT X
                 END IF 'XOR TEST
                  
            ELSE
              FOR X = 1 TO TMapDimX
                FOR Y = MapDimY TO TMapDimY
                  Map(X, Y, 1) = 0
                  Map(X, Y, 2) = 1
                NEXT Y
              NEXT X
              FOR X = MapDimX TO TMapDimX
                FOR Y = 1 TO TMapDimY
                  Map(X, Y, 1) = 0
                  Map(X, Y, 2) = 1
                NEXT Y
              NEXT X
            END IF
            MapDimX = TMapDimX
            MapDimY = TMapDimY
       END IF
       CALL DispScreen
       CALL DispText
       CALL RefreshIcons
   
   
    CASE IS = "S", "s"
      CALL SaveMap
  
    CASE IS = "O", "o"
      CALL LoadMap

    CASE IS = "M", "m"
      LOCATE 23, 31: PRINT FRE(-1)
      WHILE INKEY$ = "": WEND
      LOCATE 23, 31: PRINT "       "
      CALL DispText
   
    CASE IS = "c", "C"
      CALL ClearPic

    CASE IS = "p", "P"
      CALL DispPic
   
    CASE IS = "q", "Q"
      SCREEN 9: CLS
        COLOR 4
        PRINT "Note: By quiting you will loose all unsaved changes!"
        COLOR 7
        PRINT "Are you sure you want to quit back to the main menu? (Y/N): "
        DO
          YesNo$ = INKEY$
        LOOP UNTIL YesNo$ <> ""
        IF UCASE$(YesNo$) = "Y" THEN
              Done = True
           ELSE
              SCREEN 13: CLS
              CALL DispScreen
              CALL DispText
              CALL RefreshIcons
        END IF

    CASE IS = "1", "2", "3", "4", "5", "7", "8", "9"
      Map(WorldX, WorldY, 2) = VAL(A$)
      CALL PrintAttrib(False)

    CASE IS = "G", "g"
      IF Map(WorldX, WorldY, 1) <> 0 THEN
           GET (GrlocX * (20 + Grid) + 1 + Grid, GrLocY * (20 + Grid) + 1 + Grid)-STEP(19, 19), SelectedIcon
         PUT (236, 37), SelectedIcon, PSET
         SelectedIconAttrib = Map(WorldX, WorldY, 2)
         IconLoc = Map(WorldX, WorldY, 1)
         IF IconLoc = TotalIcons THEN
           ArrowLoc = 5
             ELSEIF IconLoc = TotalIcons - 1 THEN
               ArrowLoc = 4
                 ELSEIF (IconLoc = 1) OR (IconLoc = 2) THEN
                   ArrowLoc = IconLoc
                     ELSE
                       ArrowLoc = 3
         END IF
                     

         CALL RefreshIcons
         CALL PrintAttrib(False)
         IconAltered = False
        ELSE
          SOUND 63, 1
       END IF

   
    CASE IS = "E", "e"
      CALL IconMaker("G")

    CASE IS = "N", "n"  'Rename Map
      SCREEN 9: CLS
      INPUT "Enter new map name: ", TMapName$
      TMapName$ = MID$(TMapName$, 1, 8)
      PRINT "New Name: "; TMapName$
      INPUT "Is this the name you want? (Y/N): ", YesNo$
      IF UCASE$(MID$(YesNo$, 1, 1)) = "Y" THEN MapName$ = TMapName$
      SCREEN 13
      CALL DispScreen
      CALL RefreshIcons
      CALL DispText


    CASE IS = "w", "W"
      CALL WalkThrough

    CASE IS = "`"
           LINE (5 * (20 + Grid) + Grid + 1, 4 * (20 + Grid) + Grid + 1)-STEP(19, 19), 0, BF
           'LINE (107, 86)-(126, 105), 0, BF
           'LINE (1, YY + 1)-(XX + 20, YY + 20), 0, BF
           Map(WorldX, WorldY, 1) = 0
           Map(WorldX, WorldY, 2) = 1

    CASE IS = "l", "L"
      IF MKLock = True THEN
         MKLock = False
         LINE (239, 175)-(300, 185), 0, BF
         CALL Textfont("Lock Off", 240, 180, 7, 0)
        ELSE
          MKLock = True
          LINE (239, 175)-(300, 185), 0, BF
          CALL Textfont("Lock On", 240, 180, 7, 0)
      END IF
      Changed = True 'Sets Location Changed to True
           
END SELECT
LOCATE 1, 31: PRINT "X:"; WorldX; " "
LOCATE 2, 31: PRINT "Y:"; WorldY; " "
IF (MKLock = True) AND (Changed = True) THEN
   GOSUB LockOn:
   CALL DispScreen
END IF

LOOP UNTIL Done = True
ERASE Map
RETURN ' MainMapMaker Gosub



SetNewMap: 'Gosub Label
CLS
INPUT "Enter the name of the new map: ", MapName$
INPUT "Do you want enemies on this map?: ", YesNo$
IF UCASE$(MID$(YesNo$, 1, 1)) = "Y" THEN
     Enemies = True
  ELSE Enemies = False
END IF
INPUT "Enter the Horizontal Dimensions of the Map: ", X$
INPUT "Enter the Vertical Dimensions of the Map: ", Y$
MapDimX = VAL(X$)
MapDimY = VAL(Y$)
IF MapDimX < 20 THEN MapDimX = 20
IF MapDimY < 20 THEN MapDimY = 20
MapName$ = UCASE$(MapName$)
PRINT
PRINT "Map Name: "; MapName$
PRINT "Map X Dimension: "; MapDimX
PRINT "Map Y Dimension: "; MapDimY
PRINT: PRINT "Is this correct? (Y/N): "
DO
  A$ = INKEY$
LOOP UNTIL UCASE$(A$) = "Y" OR UCASE$(A$) = "N"
IF UCASE$(A$) = "Y" THEN
  PRINT "Allocating Memory..."
  DIM SHARED Map(MapDimX, MapDimY, 2) AS INTEGER
  WorldX = INT(MapDimX / 2)
  WorldY = INT(MapDimY / 2)
  '^^ ^^^^^^ ^^^ Makes Array for screen by the new dimensions
  FOR X = 1 TO MapDimX
    FOR Y = 1 TO MapDimY
      Map(X, Y, 1) = 0
      Map(X, Y, 2) = 1
    NEXT Y
  NEXT X
  GOSUB MainMapMaker:
END IF
  
RETURN

LoadOldMap:
  CLS
  INPUT "Enter The name of the map to load: ", Name$
  IF LEN(Name$) > 8 THEN Name$ = MID$(Name$, 1, 8)
  IF Name$ <> "" THEN
    Name$ = UCASE$(Name$) + ".MMM"
    PRINT "Loading: "; Name$
    PRINT: PRINT "Is this correct? (Y/N): "
    DO
     YesNo$ = INKEY$
    LOOP UNTIL (UCASE$(YesNo$) = "Y") OR (UCASE$(YesNo$) = "N")
     PATHNAME$ = MapPath$ + Name$
     IF UCASE$(YesNo$) = "Y" THEN
       PRINT "Loading..."
       OPEN PATHNAME$ FOR INPUT AS #1
         INPUT #1,MapName$   'Loads MapName From First Line
         INPUT #1,MapDimX    'Loads MapDimX From Second Line
         INPUT #1,MapDimY    'Loads MapDimY From 3rd Line
         INPUT #1,Enemies    'Loads if enemies are on current map
         DIM SHARED Map(MapDimX, MapDimY, 2) AS INTEGER
         WorldX = INT(MapDimX / 2)
         WorldY = INT(MapDimY / 2)
         '^^ ^^^^^^ ^^^ Makes Array for screen by the Loaded dimensions
         FOR X = 1 TO MapDimX
           FOR Y = 1 TO MapDimY
             INPUT #1,Map(X, Y, 1)  'Alternates Between the Icon # and the
             INPUT #1,Map(X, Y, 2)  'Attrib for each coordinate of the map
           NEXT Y
         NEXT X
       CLOSE #1
     PRINT "File Loaded!"
     GOSUB MainMapMaker:
    END IF
  END IF

RETURN

LockOn:
     Map(WorldX, WorldY, 1) = IconLoc
     Map(WorldX, WorldY, 2) = SelectedIconAttrib
RETURN


DEFSNG A-Z
SUB DispScreenDown
DIM TempIcon(203) AS INTEGER
XI = 1
YI = 1
FOR T = 1 TO 4
XX = XI
YY = YI
YL = 0   'X Loop Counter
  FOR Y = WorldY + 4 TO WorldY - 4 STEP -1
    IF Y <= 0 THEN
      MapY = MapDimY + Y
     ELSEIF Y >= (MapDimY + 1) THEN
      MapY = Y - MapDimY
     ELSE
      MapY = Y
    END IF
    YL = YL + 1

    FOR X = WorldX - 5 TO WorldX + 5
      IF X <= 0 THEN
        MapX = MapDimX + X
       ELSEIF X >= (MapDimX + 1) THEN
        MapX = X - MapDimX
       ELSE
        MapX = X
      END IF
     
      IF YL = 1 THEN
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (299, 0), PreLoadedIcons(1, Map(MapX, MapY, 1)), PSET
          GET (299, 0)-STEP(19, (T * 5) - 1), TempIcon
          PUT (XX, 180 + YI - (T * 5)), TempIcon(1), PSET
         ELSE
          LINE (XX, 180 + YI - (T * 5))-STEP(19, (T * 5) - 1), 0, BF
        END IF
          'FOR M = 1 TO 50000: NEXT M
      END IF
                   
      IF (YL > 1) AND (YL < 9) THEN
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (XX, YY), PreLoadedIcons(1, Map(MapX, MapY, 1)), PSET
         ELSE
          LINE (XX, YY)-STEP(19, 19), 0, BF
        END IF
      END IF

      IF YL = 9 THEN
        GET (XX, YI + 5)-STEP(19, (4 - T) * 5 - 1), TempIcon
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (XX, YY), PreLoadedIcons(1, Map(MapX, MapY, 1)), PSET
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
    
  NEXT Y
'FOR M = 1 TO 30000: NEXT M
NEXT T
ERASE TempIcon
LINE (299, 0)-STEP(19, 19), 24, BF


END SUB

SUB DispScreenLeft
DIM TempIcon(203) AS INTEGER
XI = 1
YI = 1
FOR T = 1 TO 4

XX = XI
YY = YI
XL = 0   'X Loop Counter
YL = 0   'Y Loop Counter
  FOR Y = WorldY - 4 TO WorldY + 4
    IF Y <= 0 THEN
      MapY = MapDimY + Y
     ELSEIF Y >= (MapDimY + 1) THEN
      MapY = Y - MapDimY
     ELSE
      MapY = Y
    END IF
    YL = YL + 1

    FOR X = WorldX - 5 TO WorldX + 5
      IF X <= 0 THEN
        MapX = MapDimX + X
       ELSEIF X >= (MapDimX + 1) THEN
        MapX = X - MapDimX
       ELSE
        MapX = X
      END IF
      XL = XL + 1
      
      IF XL = 1 THEN
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (299, 0), PreLoadedIcons(1, Map(MapX, MapY, 1)), PSET
          GET (299 + (4 - T) * 5, 0)-STEP((T * 5) - 1, 19), TempIcon
          PUT (XI, YY), TempIcon(1), PSET
         ELSE
          LINE (XI, YY)-STEP((T * 5) - 1, 19), 0, BF
        END IF
          'FOR M = 1 TO 50000: NEXT M
      END IF
                     
      IF (XL > 1) AND (XL < 11) THEN
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (XX, YY), PreLoadedIcons(1, Map(MapX, MapY, 1)), PSET
         ELSE
          LINE (XX, YY)-STEP(19, 19), 0, BF
        END IF
      END IF

      IF XL = 11 THEN
        GET (195 + (T * 5) + XI, YY)-STEP((4 - T) * 5 - 1, 19), TempIcon
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (XX, YY), PreLoadedIcons(1, Map(MapX, MapY, 1)), PSET
         ELSE
          LINE (XX, YY)-STEP(19, 19), 0, BF
        END IF
        IF T < 3 THEN
          PUT (200 + (T * 5) + XI, YY), TempIcon(1), PSET
        END IF
          'FOR M = 1 TO 50000: NEXT M
      END IF

      IF XL = 1 THEN
        XX = XI + (T * 5)
       ELSE
        XX = XX + 20
      END IF
      
    NEXT X
    YY = YY + 20
    XX = XI
    XL = 0
  NEXT Y
'FOR M = 1 TO 30000: NEXT M
NEXT T
ERASE TempIcon
LINE (299, 0)-STEP(19, 19), 24, BF

END SUB

SUB DispScreenRight
DIM TempIcon(203) AS INTEGER
XI = 1
YI = 1
FOR T = 1 TO 4
XX = XI
YY = YI
XL = 0   'X Loop Counter
  FOR Y = WorldY - 4 TO WorldY + 4
    IF Y <= 0 THEN
      MapY = MapDimY + Y
     ELSEIF Y >= (MapDimY + 1) THEN
      MapY = Y - MapDimY
     ELSE
      MapY = Y
    END IF

    FOR X = WorldX + 5 TO WorldX - 5 STEP -1
      IF X <= 0 THEN
        MapX = MapDimX + X
       ELSEIF X >= (MapDimX + 1) THEN
        MapX = X - MapDimX
       ELSE
        MapX = X
      END IF
      XL = XL + 1
     
      IF XL = 1 THEN
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (299, 0), PreLoadedIcons(1, Map(MapX, MapY, 1)), PSET
          GET (299, 0)-STEP((T * 5) - 1, 19), TempIcon
          PUT (220 + XI - (T * 5), YY), TempIcon(1), PSET
         ELSE
          LINE (220 + XI - (T * 5), YY)-STEP((T * 5) - 1, 19), 0, BF
        END IF
          'FOR M = 1 TO 50000: NEXT M
      END IF
                    
      IF (XL > 1) AND (XL < 11) THEN
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (XX, YY), PreLoadedIcons(1, Map(MapX, MapY, 1)), PSET
         ELSE
          LINE (XX, YY)-STEP(19, 19), 0, BF
        END IF
      END IF

      IF XL = 11 THEN
        GET (XI + 5, YY)-STEP((4 - T) * 5 - 1, 19), TempIcon
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (XX, YY), PreLoadedIcons(1, Map(MapX, MapY, 1)), PSET
         ELSE
          LINE (XX, YY)-STEP(19, 19), 0, BF
        END IF
        IF T < 3 THEN
          PUT (XI, YY), TempIcon(1), PSET
        END IF
          'FOR M = 1 TO 50000: NEXT M
      END IF

      IF XL = 1 THEN
        XX = XI + 200 - (T * 5)
       ELSE
        XX = XX - 20
      END IF
     
    NEXT X
    YY = YY + 20
    XX = IX
    XL = 0
  NEXT Y
'FOR M = 1 TO 30000: NEXT M
NEXT T
ERASE TempIcon
LINE (299, 0)-STEP(19, 19), 24, BF


END SUB

SUB DispScreenUp
DIM TempIcon(203) AS INTEGER
XI = 1
YI = 1
FOR T = 1 TO 4

XX = XI
YY = YI
YL = 0   'Y Loop Counter
  FOR Y = WorldY - 4 TO WorldY + 4
    IF Y <= 0 THEN
      MapY = MapDimY + Y
     ELSEIF Y >= (MapDimY + 1) THEN
      MapY = Y - MapDimY
     ELSE
      MapY = Y
    END IF
    YL = YL + 1

    FOR X = WorldX - 5 TO WorldX + 5
      IF X <= 0 THEN
        MapX = MapDimX + X
       ELSEIF X >= (MapDimX + 1) THEN
        MapX = X - MapDimX
       ELSE
        MapX = X
      END IF
     
      IF YL = 1 THEN
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (299, 0), PreLoadedIcons(1, Map(MapX, MapY, 1)), PSET
          GET (299, 0 + (4 - T) * 5)-STEP(19, (T * 5) - 1), TempIcon
          PUT (XX, YI), TempIcon(1), PSET
         ELSE
          LINE (XX, YI)-STEP(19, (T * 5) - 1), 0, BF
        END IF
          'FOR M = 1 TO 50000: NEXT M
      END IF
                    
      IF (YL > 1) AND (YL < 9) THEN
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (XX, YY), PreLoadedIcons(1, Map(MapX, MapY, 1)), PSET
         ELSE
          LINE (XX, YY)-STEP(19, 19), 0, BF
        END IF
      END IF

      IF YL = 9 THEN
        GET (XX, 155 + (T * 5) + YI)-STEP(19, (4 - T) * 5 - 1), TempIcon
        IF Map(MapX, MapY, 1) <> 0 THEN
          PUT (XX, YY), PreLoadedIcons(1, Map(MapX, MapY, 1)), PSET
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
    
  NEXT Y
'FOR M = 1 TO 30000: NEXT M
NEXT T
ERASE TempIcon
LINE (299, 0)-STEP(19, 19), 24, BF

END SUB

SUB DispText
CALL Textfont(MapName$, 240, 170, 7, 0)
IF MKLock = False THEN
    CALL Textfont("Lock Off", 240, 180, 7, 0)
 ELSE
    CALL Textfont("Lock On", 240, 180, 7, 0)
END IF
IF Enemies = True THEN
      CALL Textfont("Enemies On", 240, 25, 25, 0)
   ELSE
      CALL Textfont("No Enemies", 240, 25, 25, 0)
END IF
PUT (236, 37), SelectedIcon, PSET

END SUB

SUB DRAWIN (X!, Y!)
SHIFTX = 15
SHIFTY = 1
ICON(XCorr, YCorr) = Col
LINE (SHIFTX + 1 + ((X - 1) * 9), SHIFTY + 1 + ((Y - 1) * 9))-(SHIFTX + 8 + ((X - 1) * 9), 1 + SHIFTY + 7 + ((Y - 1) * 9)), Col, BF

END SUB

SUB IconMaker (S$)

END SUB

SUB IconSelection
LOCATE 23, 32: PRINT "     "
TDone = False
CALL RefreshIcons
CALL PrintAttrib(2)
DO
B$ = INKEY$
SELECT CASE B$
  
   CASE IS = "1", "2", "3", "4", "5", "6", "7", "8", "9"
     ActiveIconsAttrib(ArrowLoc) = VAL(B$)
     PATHNAME$ = MMIPATH$ + Icons(IconLoc)
     OPEN PATHNAME$ FOR OUTPUT AS #1
       FOR X = 1 TO 203
         WRITE #1, ActiveIcons(X, ArrowLoc)
       NEXT X
       WRITE #1, ActiveIconsAttrib(ArrowLoc)
       PreLoadedIconsA(IconLoc) = VAL(B$)
     CLOSE #1
     CALL PrintAttrib(2)

  
   CASE IS = CHR$(0) + ";"  'F1
      CALL Help(2)

   CASE IS = "Q", "q"
     IDone = True
  
   CASE IS = " "

         FOR X = 1 TO 203
            SelectedIcon(X) = ActiveIcons(X, ArrowLoc)
         NEXT X
         SelectedIconAttrib = ActiveIconsAttrib(ArrowLoc)
 
   CASE IS = CHR$(0) + "H"  'UP
     IF IconLoc <> 1 THEN
        IconLoc = IconLoc - 1
        ELSE SOUND 56, .5
     END IF
     IF (ArrowLoc = 3) AND ((IconLoc - 2 = 0) OR (IconLoc - 1 = 0)) THEN
         ArrowLoc = ArrowLoc - 1
         Changed = 1
     END IF
     IF (ArrowLoc = 3) THEN CALL RefreshIcons
     IF (Changed = 0) AND (ArrowLoc <> 1) AND (ArrowLoc <> 3) THEN ArrowLoc = ArrowLoc - 1
     Changed = 0
     CALL PrintAttrib(2)
  
   CASE IS = CHR$(0) + "P"  'DOWN
     IF IconLoc <> TotalIcons THEN
        IconLoc = IconLoc + 1
       ELSE SOUND 56, .5
     END IF
     IF (ArrowLoc = 3) AND (IconLoc + 1 = TotalIcons) THEN
         ArrowLoc = ArrowLoc + 1
         Changed = 1
     END IF
     IF (ArrowLoc = 3) THEN CALL RefreshIcons
     IF (Changed = 0) AND (ArrowLoc <> 5) AND ((ArrowLoc <> 3) AND (IconLoc <> TotalIcons + 2)) THEN ArrowLoc = ArrowLoc + 1
     Changed = 0
     CALL PrintAttrib(2)

   CASE IS = CHR$(0) + "I" 'PAGE UP
     IF IconLoc > 7 THEN
        IconLoc = IconLoc - 5
        ELSE IF IconLoc < 4 THEN SOUND 56, .5
     END IF
     IF (ArrowLoc > 3) THEN
         ArrowLoc = 3
         Changed = 1
     END IF
     IF (IconLoc < 7) AND (IconLoc > 3) THEN
        IconLoc = 3
        ArrowLoc = 3
     END IF
     IF (ArrowLoc = 3) THEN CALL RefreshIcons
     CALL PrintAttrib(2)

   CASE IS = CHR$(0) + "Q" 'PAGE DOWN
     IF IconLoc < TotalIcons - 6 THEN
        IconLoc = IconLoc + 5
        ELSE IF IconLoc > TotalIcons - 3 THEN SOUND 56, .5
     END IF
     IF (ArrowLoc < 3) THEN
         ArrowLoc = 3
         Changed = 1
     END IF
     IF (IconLoc > TotalIcons - 6) AND (IconLoc < TotalIcons - 2) THEN
        IconLoc = TotalIcons - 2
        ArrowLoc = 3
     END IF
     IF (ArrowLoc = 3) THEN CALL RefreshIcons
     CALL PrintAttrib(2)

   CASE IS = "E", "e"
     IconMaker ("I")

   CASE IS = CHR$(0) + "G" 'HOME
     ArrowLoc = 1
     IconLoc = 1
     RefreshIcons
 
   CASE IS = CHR$(0) + "O" 'END
     ArrowLoc = 5
     IconLoc = TotalIcons
     RefreshIcons

   CASE IS = "D", "d"  'Deleted Last icon on icon selection list
                       'Then Updates everything
      IF IconLoc = TotalIcons THEN
         FOR X = 1 TO MapDimX
           FOR Y = 1 TO MapDimY
             IF Map(X, Y, 1) = IconLoc THEN
               Map(X, Y, 1) = 0
               Map(X, Y, 2) = 1
             END IF
           NEXT Y
         NEXT X
         Icons(IconLoc) = ""
         TotalIcons = TotalIcons - 1
         OPEN MMIPATH$ + IconList$ FOR OUTPUT AS #1
           FOR X = 1 TO TotalIcons
             PRINT #1, Icons(X)
           NEXT X
         CLOSE #1
         IconLoc = TotalIcons
         CALL RefreshIcons
         CALL DispScreen
         CALL DispText
       ELSE
         SOUND 56, .5
      END IF

END SELECT

SELECT CASE ArrowLoc
     CASE IS = 1
          PSET (280, 55), 7
          DRAW "c7;r5h2f2g2"
          PSET (280, 80), 0
          DRAW "c0;r5h2f2g2"
          PSET (280, 105), 0
          DRAW "c0;r5h2f2g2"
          PSET (280, 155), 0
          DRAW "c0;r5h2f2g2"
          PSET (280, 130), 0
          DRAW "c0;r5h2f2g2"

     CASE IS = 2
          PSET (280, 80), 7
          DRAW "c7;r5h2f2g2"
          PSET (280, 55), 0
          DRAW "c0;r5h2f2g2"
          PSET (280, 105), 0
          DRAW "c0;r5h2f2g2"
          PSET (280, 155), 0
          DRAW "c0;r5h2f2g2"
          PSET (280, 130), 0
          DRAW "c0;r5h2f2g2"
   
     CASE IS = 3
          PSET (280, 105), 7
          DRAW "c7;r5h2f2g2"
          PSET (280, 55), 0
          DRAW "c0;r5h2f2g2"
          PSET (280, 80), 0
          DRAW "c0;r5h2f2g2"
          PSET (280, 155), 0
          DRAW "c0;r5h2f2g2"
          PSET (280, 130), 0
          DRAW "c0;r5h2f2g2"

     CASE IS = 4
          PSET (280, 130), 7
          DRAW "c7;r5h2f2g2"
          PSET (280, 55), 0
          DRAW "c0;r5h2f2g2"
          PSET (280, 80), 0
          DRAW "c0;r5h2f2g2"
          PSET (280, 105), 0
          DRAW "c0;r5h2f2g2"
          PSET (280, 155), 0
          DRAW "c0;r5h2f2g2"
       
     CASE IS = 5
          PSET (280, 155), 7
          DRAW "c7;r5h2f2g2"
          PSET (280, 55), 0
          DRAW "c0;r5h2f2g2"
          PSET (280, 80), 0
          DRAW "c0;r5h2f2g2"
          PSET (280, 105), 0
          DRAW "c0;r5h2f2g2"
          PSET (280, 130), 0
          DRAW "c0;r5h2f2g2"

END SELECT

'LOCATE 1, 1: PRINT "IconLoc: "; IconLoc        'Useful for Debugging
'LOCATE 2, 1: PRINT "TotalIcons: "; TotalIcons  'the Icon ScrollBar
'LOCATE 3, 1: PRINT "ArrowLoc: "; ArrowLoc

LOCATE 23, 32: PRINT IconLoc; "  "
LOOP UNTIL (B$ = " ") OR IDone = True
IF B$ = " " THEN
PUT (236, 37), SelectedIcon, PSET
END IF
LOCATE 23, 32: PRINT "     "
CALL DispText
PSET (280, 55), 0
DRAW "c0;r5h2f2g2"
PSET (280, 80), 0
DRAW "c0;r5h2f2g2"
PSET (280, 105), 0
DRAW "c0;r5h2f2g2"
PSET (280, 155), 0
DRAW "c0;r5h2f2g2"
PSET (280, 130), 0
DRAW "c0;r5h2f2g2"
IconAltered = False



END SUB

SUB IconToGet
'Converts Icon (20 x 20) format into the Get Array(203) format
      FOR X = 1 TO 20
        FOR Y = 1 TO 20
         PSET (235 + X, 36 + Y), ICON(X, Y)
        NEXT Y
      NEXT X
      GET (236, 37)-STEP(19, 19), SelectedIcon(1)

END SUB

SUB LoadIcon
END SUB

SUB MAKEGRID
SHIFTY = 1
SHIFTX = 15
'???
FOR X = SHIFTX TO 9 * 20 + 1 + SHIFTX STEP 9
    LINE (X, SHIFTY)-(X, 20 * 9 + SHIFTY), 7
NEXT X

FOR Y = SHIFTY TO 9 * 20 + 1 + SHIFTY STEP 9
    LINE (SHIFTX, Y)-(20 * 9 + SHIFTX, Y), 7
NEXT Y

FOR Y = 1 TO 20
  FOR X = 1 TO 20
    IF ICON(X, Y) <> -1 THEN
      LINE (SHIFTX + 1 + ((X - 1) * 9), SHIFTY + 1 + ((Y - 1) * 9))-(SHIFTX + 8 + ((X - 1) * 9), 1 + SHIFTY + 7 + ((Y - 1) * 9)), ICON(X, Y), BF
     ELSE
      LINE (SHIFTX + 1 + ((X - 1) * 9), SHIFTY + 1 + ((Y - 1) * 9))-(SHIFTX + 8 + ((X - 1) * 9), 1 + SHIFTY + 7 + ((Y - 1) * 9)), 0, BF
      PSET (SHIFTX + 1 + ((X - 1) * 9 + 4), SHIFTY + 1 + ((Y - 1) * 9) + 4), 7
    END IF
  NEXT X
NEXT Y

END SUB

SUB MAKEPICBOX
SHIFTX = 290
SHIFTY = 150
FOR X = 1 TO 20
  FOR Y = 1 TO 20
   IF ICON(X, Y) = -1 THEN
       PSET (SHIFTX + X, SHIFTY + Y), 0
     ELSEIF ICON(X, Y) = 0 THEN
       PSET (SHIFTX + X, SHIFTY + Y), 250
     ELSE
       PSET (SHIFTX + X, SHIFTY + Y), ICON(X, Y)
   END IF
  NEXT Y
NEXT X

END SUB

SUB RefreshIcons
Count = 0
IF ArrowLoc = 3 THEN
  FOR T = IconLoc - 2 TO IconLoc + 2
    Count = Count + 1
      FOR X = 1 TO 203
        ActiveIcons(X, Count) = PreLoadedIcons(X, T)
      NEXT X
    PUT (291, (Count * 25) + 21), ActiveIcons(1, Count), PSET
    ActiveIconsAttrib(Count) = PreLoadedIconsA(T)
    LINE (289, 19 + (Count * 25))-(311, 42 + (Count * 25)), 7, B
 NEXT T

  ELSE
  FOR T = IconLoc - ArrowLoc + 1 TO IconLoc + 5 - ArrowLoc
    Count = Count + 1
       FOR X = 1 TO 203
         ActiveIcons(X, Count) = PreLoadedIcons(X, T)
       NEXT X
    PUT (291, (Count * 25) + 21), ActiveIcons(1, Count), PSET
    ActiveIconsAttrib(Count) = PreLoadedIconsA(T)
    LINE (289, 19 + (Count * 25))-(311, 42 + (Count * 25)), 7, B
 NEXT T
END IF

END SUB


SUB Rotate (IM AS INTEGER)
DIM NICON(20, 20) AS INTEGER
FOR X = 1 TO 20
  FOR Y = 1 TO 20
    NICON(21 - Y, X) = ICON(X, Y)
  NEXT Y
NEXT X
FOR X = 1 TO 20
  FOR Y = 1 TO 20
    ICON(X, Y) = NICON(X, Y)
  NEXT Y
NEXT X
ERASE NICON
IF IM = True THEN
MAKEGRID
MAKEPICBOX
END IF


END SUB

SUB SaveIcon
DIM TempIcon(203) AS INTEGER
GET (291, 151)-STEP(19, 19), TempIcon(1)
LINE (1, 183)-(200, 200), 0, BF
LINE (15, 1)-(195, 181), 7, B
LINE (16, 2)-(194, 180), 0, BF
CALL Textfont("* SAVE *", 83, 20, 7, 0)
CALL Textfont("Enter the file name:", 25, 50, 7, 0)
X = 125
Name$ = ""
DO
   S$ = INKEY$
   IF (X < 165) AND (S$ <> "") AND (S$ <> CHR$(13)) AND (((S$ > "/") AND (S$ < ":")) OR ((S$ > "@") AND (S$ < "[")) OR ((S$ > CHR$(96)) AND (S$ < "{"))) THEN
      X = X + 5
      CALL Textfont(S$, X, 50, 7, 0)
      Name$ = Name$ + S$
   END IF
   IF (S$ = CHR$(8)) AND (Name$ <> "") THEN
      CALL Textfont("~", X, 50, 0, 0)
      X = X - 5
      Name$ = MID$(Name$, 1, LEN(Name$) - 1)
   END IF
 
LOOP UNTIL S$ = CHR$(13)
IF Name$ <> "" THEN
  Name$ = Name$ + ".MMI"

  CALL Textfont("Saving file as: " + Name$, 25, 60, 7, 0)
  CALL Textfont("Is this Correct? (Y/N): ", 25, 80, 7, 0)

  DO
  YesNo$ = INKEY$
   IF UCASE$(YesNo$) = "Y" THEN
     CALL Textfont("Y", 140, 80, 7, 1)
       ELSEIF UCASE$(YesNo$) = "N" THEN
         CALL Textfont("N", 140, 80, 7, 1)
   END IF
  LOOP UNTIL (UCASE$(YesNo$) = "Y") OR (UCASE$(YesNo$) = "N")
PATHNAME$ = MMIPATH$ + Name$
  IF UCASE$(YesNo$) = "Y" THEN
    OPEN PATHNAME$ FOR OUTPUT AS #1
    FOR X = 1 TO 203
      WRITE #1, TempIcon(X)
    NEXT X
    WRITE #1, IconAttribute
    CLOSE #1
  END IF

CALL Textfont("File Saved!", 25, 90, 7, 0)
WHILE INKEY$ = "": WEND
END IF
ERASE TempIcon
CALL MAKEGRID
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)

END SUB

SUB SaveMap
CLS
'LINE (210, 0)-(319, 15), 0, BF
LINE (1, 1)-(232, 190), 7, B
'LINE (2, 2)-(231, 189), 0, BF
CALL Textfont("* SAVE *", 83, 20, 7, 0)
Name$ = MapName$ + ".MMM"
CALL Textfont("Saving file as: " + Name$, 25, 60, 7, 0)
  CALL Textfont("Is this Correct? (Y/N): ", 25, 80, 7, 0)

  DO
  YesNo$ = INKEY$
   IF UCASE$(YesNo$) = "Y" THEN
     CALL Textfont("Y", 140, 80, 7, 1)
       ELSEIF UCASE$(YesNo$) = "N" THEN
         CALL Textfont("N", 140, 80, 7, 1)
   END IF
  LOOP UNTIL (UCASE$(YesNo$) = "Y") OR (UCASE$(YesNo$) = "N")
PATHNAME$ = MapPath$ + Name$
IF UCASE$(YesNo$) = "Y" THEN
    CALL Textfont("Now saving...", 25, 90, 7, 0)
    OPEN PATHNAME$ FOR OUTPUT AS #1
    WRITE #1, MapName$  'Saves MapName On First Line
    WRITE #1, MapDimX   'Saves MapDimX on second Line
    WRITE #1, MapDimY   'Saves MapDimY on 3rd Line
    WRITE #1, Enemies   'Saves if enemies will be on current map
    FOR X = 1 TO MapDimX
      FOR Y = 1 TO MapDimY
        WRITE #1, Map(X, Y, 1) 'Alternates Between the Icon # and the
        WRITE #1, Map(X, Y, 2) 'Attrib for each coordinate of the map
      NEXT Y
    NEXT X
    CLOSE #1
  
CALL Textfont("File Saved!", 25, 100, 7, 0)
WHILE INKEY$ = "": WEND
END IF
'The File Format is:
'The Name of The Map (Line 1)
'MapDimX (Line 2)
'MapDimY (Line 3)
'Enemies (Line 4)
'Map(1,1,1) (Line 5) *Icon #
'Map(1,1,2) (Line 6) *Attrib
'Map(2,1,1) ...
'Map(2,1,2) ...
CLS
CALL DispScreen
CALL RefreshIcons
CALL DispText
END SUB



SUB SelectCol

END SUB

SUB UpdateIcon
SelectedIconAttrib = IconAttribute
           


DIM Found(1) AS INTEGER, TCount(2) AS INTEGER, TempIcon(203) AS INTEGER
  Found(1) = False: TCount(1) = 0:
          GET (291, 151)-STEP(19, 19), TempIcon(1)
            DO
              TCount(1) = TCount(1) + 1
              Found(1) = 1: TCount(2) = 0
              DO
                TCount(2) = TCount(2) + 1
                TempNum = PreLoadedIcons(TCount(2), TCount(1))
                IF TempNum <> TempIcon(TCount(2)) THEN Found(1) = 0
              LOOP UNTIL Found(1) = 0 OR TCount(2) = 203
              
            LOOP UNTIL (Found(1) = 1) OR TCount(1) = TotalIcons


     IF Found(1) = 1 THEN
       FOR X = 1 TO 203
         SelectedIcon(X) = PreLoadedIcons(X, TCount(1))
       NEXT X
       SelectedIconAttrib = PreLoadedIconsA(TCount(1))
       IconLoc = TCount(1)
       IF (IconLoc = 1) OR (IconLoc = 2) THEN
          ArrowLoc = IconLoc
        ELSEIF IconLoc = TotalIcons THEN
           ArrowLoc = 5
         ELSEIF IconLoc = TotalIcons - 1 THEN
           ArrowLoc = 4
        ELSE
          ArrowLoc = 3
       END IF


     ELSE 'If The Icon being applied is a totally new one
       TotalIcons = TotalIcons + 1
       Icons(TotalIcons) = MID$(STR$(TotalIcons), 2, LEN(STR$(TotalIcons)) - 1) + ".MMI"
       PATHNAME$ = MMIPATH$ + MID$(STR$(TotalIcons), 2, LEN(STR$(TotalIcons)) - 1) + ".MMI"
       OPEN PATHNAME$ FOR OUTPUT AS #1
         FOR X = 1 TO 203
           WRITE #1, TempIcon(X)
           PreLoadedIcons(X, TotalIcons) = TempIcon(X)
           SelectedIcon(X) = PreLoadedIcons(X, TotalIcons)
         NEXT X
         WRITE #1, SelectedIconAttrib
         PreLoadedIconsA(TotalIcons) = SelectedIconAttrib
         PRINT SelectedIconAttrib
         Icons(TotalIcons) = MID$(STR$(TotalIcons), 2, LEN(STR$(TotalIcons)) - 1) + ".MMI"
       CLOSE #1
         'Made New Icon and added it to the Icons(..) and PreLoaded..
       OPEN MMIPATH$ + IconList$ FOR APPEND AS #1
         PRINT #1, (MID$(STR$(TotalIcons), 2, LEN(STR$(TotalIcons)) - 1) + ".MMI")
       CLOSE #1
         'Added New Icon to the MMIFiles.txt
       IconLoc = TotalIcons
       ArrowLoc = 5
    END IF
ERASE Found, TCount, TempIcon
END SUB