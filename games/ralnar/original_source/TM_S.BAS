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
CONST MaxIcons = 300
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
'Music = False
'ON PLAY(3) GOSUB GetNote

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
     BEEP: BEEP: COLOR 4
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
Grid = True  'Screen Grid On
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
'$INCLUDE: 'C:\BASCOMP\Sample.BAS'
'GOSUB GetNote:
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
LINE (5 * (20 + Grid) + Grid, 4 * (20 + Grid) + Grid)-STEP(21, 21), 15, B
A$ = INKEY$
SELECT CASE A$
       CASE IS = CHR$(0) + "H"    'UP
         IF WorldY = 1 THEN
            WorldY = MapDimY
           ELSE
             WorldY = WorldY - 1
          END IF
          IF MKLock = True THEN GOSUB LockOn:
          CALL DispScreen

      CASE IS = CHR$(0) + "P"    'DOWN
          IF WorldY = MapDimY THEN
             WorldY = 1
            ELSE WorldY = WorldY + 1
          END IF
          IF MKLock = True THEN GOSUB LockOn:
          CALL DispScreen
    
      CASE IS = CHR$(0) + "K"    'LEFT
         IF WorldX = 1 THEN
            WorldX = MapDimX
           ELSE
             WorldX = WorldX - 1
          END IF
          IF MKLock = True THEN GOSUB LockOn
          CALL DispScreen
    
      CASE IS = CHR$(0) + "M"    'RIGHT
          IF WorldX = MapDimX THEN
             WorldX = 1
            ELSE WorldX = WorldX + 1
          END IF
           IF MKLock = True THEN GOSUB LockOn:
           CALL DispScreen

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
       LINE (1, 1)-(232, 190), 0, BF
       IF Grid = 1 THEN Grid = 0 ELSE Grid = 1
       CALL DispScreen

    CASE IS = CHR$(0) + ">"  'F4
       IF Music = True THEN
          Music = False
          PLAY OFF
            ELSE
          Music = True
          PLAY ON
          Num = 1
          'GOSUB GetNote:
       END IF
   
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

    CASE IS = "1", "2", "3", "4", "5", ",7", "8", "9"
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



'GetNote:
'SELECT CASE Num
'
'     CASE IS = 1
'          PLAY "o5 L4  MB T255  <F#. >A. < F# F# > B< F# E..  F#. > C# <"
'
'     CASE IS = 2
'          PLAY "MB F# F#> D C# A... < F#. >C# F# <F# E E C# G# F#.. P1"
'
'
'     CASE IS = 3
'          PLAY "MB T255  F#. >A. < F# F# > B< F# E..  F#. > C# <F# F#> D C# A..."
'
'     CASE IS = 4
'          PLAY "MB < F#. >C# F# <F# E E C# G# F#....P5 "
'
'
'     CASE IS = 5
'          PLAY "MB T235 <<<< F#.. > F#. < E > E < C# > C# < E F#.. > F#...< C# >"
'
'     CASE IS = 6
'          PLAY "MB C# E F# < D.. > D.. < E > E < C# E F# > F#...... E C# < B  L3 A. "
'
'     CASE IS = 7
'          PLAY "L4 MB T235  F#.. > F#. < E > E < C# > C# < E F#.. > F#..."
'
'     CASE IS = 8
'          PLAY "MB < C# > C# E F# < D.. > D.. < E > E < C# E F# > F#...... E C# < B >>>>"
'END SELECT
'IF Num < 8 THEN Num = Num + 1 ELSE Num = 1
'RETURN

DEFSNG A-Z
SUB ChangeScreenDim (Apply, TMapDimX, TMapDimY)
SCREEN 9: CLS
LOCATE 1, 1: PRINT "Change Screen Dimensions"
LOCATE 20, 1: PRINT "Press ESC to Abort Changes"
LOCATE 3, 1: PRINT "H)orizontal Change"
LOCATE 4, 1: PRINT "V)ertical Change"
LOCATE 5, 1: PRINT "A)pply Changes and Exit"
LOCATE 1, 30: PRINT "MapDimX: "; MapDimX
LOCATE 2, 30: PRINT "MapDimY: "; MapDimY
TMapDimX = MapDimX
TMapDimY = MapDimY

DO
A$ = INKEY$
  SELECT CASE A$
     CASE IS = "h", "H"
        GOSUB ChangeH
     CASE IS = "v", "V"
        GOSUB ChangeV
     CASE IS = "a", "A"
        IF (TMapDimX <> MapDimX) OR (TMapDimY <> MapDimY) THEN Apply = True

  END SELECT

LOOP UNTIL (A$ = CHR$(27)) OR (UCASE$(A$) = "A")


SCREEN 13
EXIT SUB


ChangeH:
LOCATE 15, 1: PRINT "Horizontal Change (+/-)"
DO
  B$ = INKEY$
  SELECT CASE B$
    CASE IS = "+"
       TMapDimX = TMapDimX + 1
       LOCATE 1, 39: PRINT TMapDimX; " "

    CASE IS = "-"
       IF TMapDimX > MapDimX THEN
          TMapDimX = TMapDimX - 1
          LOCATE 1, 39: PRINT TMapDimX; " "
        ELSE
          BEEP
       END IF
  END SELECT

LOOP UNTIL (B$ = CHR$(13)) OR (B$ = " ")
LOCATE 15, 1: PRINT "                       "
RETURN

ChangeV:
LOCATE 15, 1: PRINT "Vertical Change (+/-)"
DO
  B$ = INKEY$
  SELECT CASE B$
    CASE IS = "+"
       TMapDimY = TMapDimY + 1
       LOCATE 2, 39: PRINT TMapDimY; " "

    CASE IS = "-"
       IF TMapDimY > MapDimY THEN
          TMapDimY = TMapDimY - 1
          LOCATE 2, 39: PRINT TMapDimY; " "
        ELSE
          BEEP
       END IF
  END SELECT

LOOP UNTIL (B$ = CHR$(13)) OR (B$ = " ")
LOCATE 15, 1: PRINT "                     "
RETURN


END SUB



SUB ClearIcon (Initial)
IF Initial = 1 THEN
  LINE (1, 183)-(200, 200), 0, BF
  LINE (15, 1)-(195, 181), 7, B
  LINE (16, 2)-(194, 180), 0, BF
  CALL Textfont("* CLEAR PICTURE *", 75, 20, 7, 0)
  CALL Textfont("Clear the picture! ", 25, 60, 7, 0)
  CALL Textfont("Is this Correct? (Y/N): ", 25, 80, 7, 0)
  DO
  YesNo$ = INKEY$
   IF UCASE$(YesNo$) = "Y" THEN
     CALL Textfont("Y", 140, 80, 7, 1)
       ELSEIF UCASE$(YesNo$) = "N" THEN
         CALL Textfont("N", 140, 80, 7, 1)
   END IF
   LOOP UNTIL (UCASE$(YesNo$) = "Y") OR (UCASE$(YesNo$) = "N")
   IF UCASE$(YesNo$) = "Y" THEN Initial = 0
END IF

IF Initial = 0 THEN
FOR X = 1 TO 20        ' This sets all the dimensions to clear
  FOR Y = 1 TO 20      '
    ICON(X, Y) = -1      '
  NEXT Y                 '
NEXT X                   '

Col = 0
'LINE (290, 170)-(311, 191), 7, B
CALL MAKEGRID  'Draws Grid to Screen
CALL MAKEPICBOX 'Makes the picture box in the bottom right
CALL SelectCol

IconAttribute = 1
OXCorr = 1
OYCorr = 1
XCorr = 1
YCorr = 1
CHANGE.CORR = 0
END IF
CALL PrintAttrib(True)
CALL MAKEGRID
CALL MAKEPICBOX
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)

END SUB

SUB ClearPic
  LINE (1, 1)-(232, 190), 7, B
  LINE (2, 2)-(231, 189), 0, BF
  CALL Textfont("* CLEAR PICTURE *", 75, 20, 7, 0)
  CALL Textfont("Clear the picture! ", 25, 60, 7, 0)
  CALL Textfont("Is this Correct? (Y/N): ", 25, 80, 7, 0)
  DO
  YesNo$ = INKEY$
   IF UCASE$(YesNo$) = "Y" THEN
     CALL Textfont("Y", 140, 80, 7, 1)
       ELSEIF UCASE$(YesNo$) = "N" THEN
         CALL Textfont("N", 140, 80, 7, 1)
   END IF
   LOOP UNTIL (UCASE$(YesNo$) = "Y") OR (UCASE$(YesNo$) = "N")
   IF UCASE$(YesNo$) = "Y" THEN
          FOR X = 1 TO MapDimX
               FOR Y = 1 TO MapDimY
                    Map(X, Y, 1) = 0
                    Map(X, Y, 2) = 1
               NEXT Y
          NEXT X
   END IF
CALL DispScreen
CALL DispText
CALL IconSelection


END SUB

SUB DispPic
CLS
IF (MapDimX < 320) AND (MapDimY < 200) THEN
   LINE (159 - INT(MapDimX / 2), 99 - INT(MapDimY / 2))-(160 - INT(MapDimX / 2) + MapDimX, 100 - INT(MapDimY / 2) + MapDimY), 7, B
   YY = 0: XX = 0
   FOR X = (160 - INT(MapDimX / 2)) TO (159 - INT(MapDimX / 2) + MapDimX)
     XX = XX + 1
     FOR Y = (100 - INT(MapDimY / 2)) TO (99 - INT(MapDimY / 2) + MapDimY)
     YY = YY + 1
       GOSUB GetCol:
       PSET (X, Y), C
     NEXT Y
     YY = 0
   NEXT X



ELSE
  SCREEN 9
   YY = 0: XX = 0
   FOR X = (320 - INT(MapDimX / 2)) TO (319 - INT(MapDimX / 2) + MapDimX)
     XX = XX + 1
     FOR Y = (200 - INT(MapDimY / 2)) TO (199 - INT(MapDimY / 2) + MapDimY)
     YY = YY + 1
       GOSUB GetCol:
       PSET (X, Y), C
     NEXT Y
     YY = 0
   NEXT X
END IF

DO
A$ = INKEY$
IF (MapDimX < 320) AND (MapDimY < 200) THEN 'Screen 13?
    PSET (WorldX + 160 - INT(MapDimX / 2), WorldY + 99 - INT(MapDimY / 2)), 15
    PSET (WorldX + 160 - INT(MapDimX / 2), WorldY + 99 - INT(MapDimY / 2)), 0
  ELSE
    PSET (WorldX + 320 - INT(MapDimX / 2), WorldY + 199 - INT(MapDimY / 2)), 15
    PSET (WorldX + 320 - INT(MapDimX / 2), WorldY + 199 - INT(MapDimY / 2)), 0
END IF

LOOP UNTIL A$ <> ""

SCREEN 13: CLS
CALL DispScreen
CALL RefreshIcons
CALL DispText

EXIT SUB
GetCol:    'Gosub Label
C = 0
SELECT CASE Map(XX, YY, 1)
      
     CASE IS = 8 'Grass
         C = 10

     CASE IS = 9, 10 'Desert
         C = 14 'Yellow

     CASE IS = 11  'Water
         C = 9

     CASE IS = 48 'Lava
         C = 4

     CASE IS = 64 ' Castle Floor
         C = 7

     CASE IS = 91 'Sky
         C = 53

END SELECT

IF C = 0 THEN
    SELECT CASE Map(XX, YY, 2)

          CASE IS = 3 'Event Passable
              C = 15
         
          CASE IS = 4  'Event Impassable
              C = 5

          CASE IS = 5 'Impassable
              C = 8

          CASE IS = 1 'Land
              C = 2

     END SELECT
END IF
IF Map(XX, YY, 1) = 0 THEN C = 0

RETURN 'End Gosub GetCol

END SUB

SUB DispScreen
IF Grid = 1 THEN
  FOR X = 1 TO 252 STEP 21
      LINE (X, 1)-(X, 190), 7
  NEXT X

  FOR Y = 1 TO 210 STEP 21
    IF (5 - WorldY = INT(Y / 21)) OR (MapDimY - WorldY + 5 = INT(Y / 21)) THEN
      LINE (2, Y)-(231, Y), 9
     ELSE
      LINE (1, Y)-(232, Y), 7
    END IF
  NEXT Y

  FOR X = 1 TO 252 STEP 21  'Redraws Blue line after it gray Y lines go trough it
   IF (6 - WorldX = INT(X / 21)) OR (MapDimX - WorldX + 6 = INT(X / 21)) THEN
     LINE (X, 2)-(X, 189), 9
   END IF
  NEXT X

END IF


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

CALL PrintAttrib(False)

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

SUB FlipH (IM AS INTEGER)
DIM NICON(20, 20) AS INTEGER
FOR X = 1 TO 20
  FOR Y = 1 TO 20
    NICON(21 - X, Y) = ICON(X, Y)
  NEXT Y
NEXT X
FOR X = 1 TO 20
  FOR Y = 1 TO 20
    ICON(X, Y) = NICON(X, Y)
  NEXT Y
NEXT X
ERASE NICON
IF IM = True THEN
CALL MAKEGRID
CALL MAKEPICBOX
END IF

END SUB

SUB FlipV (IM AS INTEGER)
DIM NICON(20, 20) AS INTEGER
FOR X = 1 TO 20
  FOR Y = 1 TO 20
    NICON(X, 21 - Y) = ICON(X, Y)
  NEXT Y
NEXT X
FOR X = 1 TO 20
  FOR Y = 1 TO 20
    ICON(X, Y) = NICON(X, Y)
  NEXT Y
NEXT X
ERASE NICON
IF IM = True THEN
CALL MAKEGRID
CALL MAKEPICBOX
END IF
END SUB

SUB GetToIcon
FOR X = 1 TO 20
  FOR Y = 1 TO 20
  ICON(X, Y) = POINT(235 + X, 36 + Y)
  NEXT Y
NEXT X

END SUB

SUB Help (HelpMode AS INTEGER)
MaxPages = 5
SCREEN 9: CLS
GOSUB DispHelp
DO
A$ = INKEY$
   SELECT CASE A$
      CASE IS = (CHR$(0) + "H"), (CHR$(0) + "I")   'UP or PageUp
         IF HelpMode = 1 THEN
            HelpMode = MaxPages
          ELSE
            HelpMode = HelpMode - 1
         END IF
         GOSUB DispHelp

     
      CASE IS = (CHR$(0) + "P"), (CHR$(0) + "Q")   'DOWN or Page Down
         IF HelpMode = MaxPages THEN
            HelpMode = 1
          ELSE
            HelpMode = HelpMode + 1
         END IF
         GOSUB DispHelp
   END SELECT


LOOP UNTIL (UCASE$(A$) = "Q") OR A$ = CHR$(27)
SCREEN 13: CLS
CALL DispScreen
CALL RefreshIcons
CALL DispText
EXIT SUB 'Leaves Sub

DispHelp:    'Gosub Label
CLS
LOCATE 1, 35: PRINT "* * H E L P * *"
COLOR 15: LOCATE 2, 1
  SELECT CASE HelpMode
       CASE IS = 1   'MapMaker
           PRINT "Map Maker Help         "
           PRINT: COLOR 7
           PRINT " W               Walkthrough mode"
           PRINT " I               Goes to icon selection mode"
           PRINT " G               Gets current icon for use"
           PRINT " L               Locks icon for dragging"
           PRINT " O               Opens previous map"
           PRINT " S               Saves map under current map name"
           PRINT " C               Clears and resets map"
           PRINT " P               Displays pixelated map"
           PRINT " M               Shows free conventional memory left "
           PRINT " E               Icon editing mode"
           PRINT " R,H,V           Rotates, or flips icon horizontal or vertical"
           PRINT " Q               Exits program"
           PRINT " `               Clears current icon location"
           PRINT " F1              Help screen"
           PRINT " F2              Enemy (On/Off) Toggle "
           PRINT " F3              Toggles grid on and off"
           PRINT " 1,2,3,4,5,9     Icon Attributes"
           PRINT " (SPACE BAR)     Places selected icon"
           LOCATE 23, 1: PRINT "Page 1 of 5"

       CASE IS = 2   'IconMaker
           PRINT "Icon Maker Help         "
           PRINT: PRINT: COLOR 7
           PRINT " G               Gets current color for use"
           PRINT " L               Locks color for dragging"
           PRINT " O               Opens previous icon"
           PRINT " S               Saves icon"
           PRINT " C               Selects color"
           PRINT " E               Clears icon"
           PRINT " U               Updates MMIFiles.txt with current icon"
           PRINT " R,H,V           Rotates, or flips icon horizontal or vertical"
           PRINT " Q, ESC          Exits back to map mode"
           PRINT " `               Clears current location on icon"
           PRINT " F1              Help screen"
           PRINT " 1,2,3,4,5,9     Icon Attributes"
           PRINT " (SPACE BAR)     Places selected color, and selects color for scroll bar"
           LOCATE 23, 1: PRINT "Page 2 of 5"

       CASE IS = 3   'WalkThrough
           PRINT "Walkthrough Help         "
           PRINT: PRINT: COLOR 7
           PRINT " M               Toggles modes (Air,Land,Boat)"
           PRINT " Q,ESC           Exits back to map mode"
           PRINT " F1              Help screen"
           PRINT " (SPACE BAR)     Triggers event in front of cursor"
           PRINT " (Directional keys for movement)"
           LOCATE 23, 1: PRINT "Page 3 of 5"

      
       CASE IS = 4   'IconSelection
           PRINT "Icon Selection Help     "
           PRINT: PRINT: COLOR 7
           PRINT " E               Icon edit mode on current icon"
           PRINT " D               Deletes last icon in Icon Selection List"
           PRINT " F1              Help screen"
           PRINT " 1-9             Icon Attributes"
           PRINT "                Up an icon"
           PRINT "                Down an icon"
           PRINT "PgUp             Up 5 icons"
           PRINT "PgDn             Down 5 icons"
           PRINT "Home             First icon"
           PRINT "End              Last icon"
           PRINT "(SPACE BAR)      Selects icon"
           LOCATE 23, 1: PRINT "Page 4 of 5"

       CASE IS = 5   'Icon Attributes and info
           PRINT " General Help     "
           PRINT: PRINT: COLOR 7
           PRINT " Icon Attributes:"
           PRINT " 1               Land (Walking movement)"
           PRINT " 2               Water (Boat movement)"
           PRINT " 3               Event Passable (Triggers event)"
           PRINT " 4               Event Impassible (Triggers event)"
           PRINT " 5               Impassible"
           PRINT " 6               Same (used in icon selection only, for icons w/ clear)"
           PRINT " 7               See the top half of the person"
           PRINT " 8               Don't see the person"
           PRINT " 9               Passable by any movement"
           PRINT
           PRINT "Icons:           20x20 pixels"
           PRINT
           PRINT "Map Maker / Icon Maker programmed by BRIAN GULLIVER (1996)"
          
           LOCATE 23, 1: PRINT "Page 5 of 5"



  END SELECT
RETURN



END SUB

SUB IconMaker (S$)

CLS
IF S$ = "G" THEN
  IconAttribute = SelectedIconAttrib
  PUT (291, 151), SelectedIcon(1), PSET
    FOR X = 1 TO 20
       FOR Y = 1 TO 20
          A = POINT((X + 290), (Y + 150))
          IF A = 0 THEN
            ICON(X, Y) = -1
          ELSEIF A = 250 THEN
            ICON(X, Y) = 0
          ELSE
            ICON(X, Y) = A
          END IF
       NEXT Y
    NEXT X
ELSEIF S$ = "I" THEN
  IconAttribute = ActiveIconsAttrib(ArrowLoc)
  PUT (291, 151), ActiveIcons(1, ArrowLoc), OR
    FOR X = 1 TO 20
       FOR Y = 1 TO 20
          A = POINT((X + 290), (Y + 150))
          IF A = 0 THEN
            ICON(X, Y) = -1
          ELSEIF A = 250 THEN
            ICON(X, Y) = 0
          ELSE
            ICON(X, Y) = A
          END IF
       NEXT Y
    NEXT X
ELSEIF S$ = "N" THEN
  ClearIcon (0)
END IF

'ICON MAKER
'By Brian Gulliver
'20 x 20
SCREEN 13

KLOCK = False
'NAME$ = ""
CALL MAKEGRID
CALL SelectCol
OXCorr = 1
OYCorr = 1
CHANGE.CORR = 0
CALL PrintAttrib(True)
CALL MAKEGRID
CALL MAKEPICBOX
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)
CALL Textfont("Lock OFF", 210, 170, 7, 0)
XCorr = 1
YCorr = 1

DO

IF CHANGE.CORR = 1 THEN LINE ((OXCorr * 9) + 6, ((OYCorr - 1) * 9) + 1)-((OXCorr * 9) + 6 + 9, ((OYCorr - 1) * 9) + 1 + 9), 7, B
LINE ((XCorr * 9) + 6, ((YCorr - 1) * 9) + 1)-((XCorr * 9) + 6 + 9, ((YCorr - 1) * 9) + 1 + 9), 15, B
A$ = INKEY$
OXCorr = XCorr
OYCorr = YCorr
A$ = UCASE$(A$)

SELECT CASE A$

    CASE IS = CHR$(0) + ";"  'F1
       CALL Help(2)
    
     CASE IS = "C"
          CALL SelectCol

     CASE IS = CHR$(0) + "M"
          IF XCorr <> 20 THEN XCorr = XCorr + 1

     CASE IS = CHR$(0) + "K"
          IF XCorr <> 1 THEN XCorr = XCorr - 1

     CASE IS = CHR$(0) + "H"
          IF YCorr <> 1 THEN YCorr = YCorr - 1

     CASE IS = CHR$(0) + "P"
          IF YCorr <> 20 THEN YCorr = YCorr + 1

     CASE IS = "R"
          CALL Rotate(True)
      
     CASE IS = "F"
          CALL FlipH(True)

     CASE IS = "V"
          CALL FlipV(True)

     CASE IS = "U"
          CALL UpdateIcon
          A$ = "B"

     CASE IS = "H"
          'CALL HelpIconMaker

     CASE IS = "S"
          CALL SaveIcon

     CASE IS = "O"
          CALL LoadIcon

     CASE IS = "Z"
          'CALL SaveIcon2

     CASE IS = "X"
          'CALL LoadIcon2

     CASE IS = "G"
          Col = ICON(XCorr, YCorr)
          CALL SelectCol

     CASE IS = "1", "2", "3", "4", "5", "6", "7", "8", "9"
          IconAttribute = VAL(A$)
          CALL PrintAttrib(True)

     CASE IS = "E"
          CALL ClearIcon(1)

     CASE IS = "L"
          IF KLOCK = 1 THEN KLOCK = 0 ELSE KLOCK = 1
          LINE (205, 165)-(280, 175), 0, BF
          IF KLOCK = 1 THEN
               CALL Textfont("Lock ON", 210, 170, 7, 0)
            ELSE
               CALL Textfont("Lock OFF", 210, 170, 7, 0)
          END IF
          A$ = " "

     CASE IS = "`"
          ICON(XCorr, YCorr) = -1
          CALL MAKEGRID  'try call Drawin?
          CALL MAKEPICBOX

END SELECT

IF (OXCorr <> XCorr) OR (OYCorr <> YCorr) THEN CHANGE.CORR = 1 ELSE CHANGE.CORR = 0
LOCATE 18, 26: PRINT "("; XCorr; ","; YCorr; ")    "

IF (A$ = " ") THEN
    CALL DRAWIN(XCorr, YCorr)
    CALL MAKEPICBOX
END IF

IF (CHANGE.CORR = 1) AND (KLOCK = 1) THEN
   CALL DRAWIN(XCorr, YCorr)
   CALL MAKEPICBOX
END IF

LOOP UNTIL UCASE$(A$) = "B" OR UCASE$(A$) = "Q"

CLS
CALL DispScreen
CALL RefreshIcons
CALL DispText

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
DIM TempIcon(203) AS INTEGER
LINE (1, 183)-(200, 200), 0, BF
LINE (15, 1)-(195, 181), 7, B
LINE (16, 2)-(194, 180), 0, BF
CALL Textfont("* LOAD *", 83, 20, 7, 0)
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

  CALL Textfont("Loading file: " + Name$, 25, 60, 7, 0)
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
    OPEN PATHNAME$ FOR INPUT AS #1
    FOR X = 1 TO 203
      INPUT #1,TempIcon(X)
    NEXT X
    INPUT #1,IconAttribute
  CLOSE #1
  PUT (291, 151), TempIcon(1), PSET
    FOR X = 1 TO 20
       FOR Y = 1 TO 20
          A = POINT((X + 290), (Y + 150))
          IF A = 0 THEN
            ICON(X, Y) = -1
          ELSEIF A = 250 THEN
            ICON(X, Y) = 0
          ELSE
            ICON(X, Y) = A
          END IF
       NEXT Y
    NEXT X
END IF


END IF
CALL PrintAttrib(True)
CALL MAKEGRID
CALL MAKEPICBOX
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)

END SUB

SUB LoadMap
LINE (210, 0)-(319, 15), 0, BF
LINE (1, 1)-(232, 190), 7, B
LINE (2, 2)-(231, 189), 0, BF
CALL Textfont("* LOAD *", 83, 20, 7, 0)
CALL Textfont("Enter the file name:", 25, 50, 7, 0)
X = 125
Name$ = ""
DO
   S$ = INKEY$
   IF (X < 173) AND (S$ <> "") AND (S$ <> CHR$(13)) AND (((S$ > "/") AND (S$ < ":")) OR ((S$ > "@") AND (S$ < "[")) OR ((S$ > CHR$(96)) AND (S$ < "{"))) THEN
      X = X + 6
      CALL Textfont(S$, X, 50, 7, 0)
      Name$ = Name$ + S$
   END IF
   IF (S$ = CHR$(8)) AND (Name$ <> "") THEN
      CALL Textfont("~", X, 50, 0, 0)
      X = X - 6
      Name$ = MID$(Name$, 1, LEN(Name$) - 1)
   END IF

LOOP UNTIL S$ = CHR$(13)
IF Name$ <> "" THEN
  Name$ = Name$ + ".MMM"

  CALL Textfont("Loading file: " + Name$, 25, 60, 7, 0)
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
    OPEN PATHNAME$ FOR INPUT AS #1
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
  END IF
LOCATE 1, 1:
END IF
CLS
CALL DispScreen
CALL RefreshIcons
CALL DispText
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

SUB OpeningScreen
SCREEN 13
CLS
PSET (10, 10), 25
DRAW "d30r10u20r10d20r10u20r10d20r10u30l50 br60 d10ld10ld10r5u10ru5r7d5rd10r5u10lu10lu10l15bd10br10u5l5d5r5bu10 br15 d30r7u15r10u15l16bd10br10u5l5d5r5bu10bd34"
DRAW "d30r10u20r10d20r10u20r10d20r10u30l50 br60 d10ld10ld10r5u10ru5r7d5rd10r5u10lu10lu10l15bd10br10u5l5d5r5bu10 br15 d30r5u10f10r5u5h13f3e10u5l5g9u9l5 br25"
DRAW "r20d5l15d8r15d5l15d7r15d5l20u30 br25 d30r5u10f10r5u5h10r5u15l15bd10br10u5l5d5r5"
PAINT (15, 12), 23, 25
PAINT (75, 12), 23, 25
PAINT (100, 12), 23, 25
PAINT (130, 45), 23, 25
PAINT (180, 45), 23, 25
PAINT (195, 45), 23, 25
PAINT (230, 45), 23, 25
PAINT (245, 45), 23, 25
COLOR 7
LOCATE 11, 20: PRINT Ver$

END SUB

SUB PrintAttrib (IM AS INTEGER)
C = 1
IF IM = True THEN  'True = 1
    AttribNum = IconAttribute
  ELSEIF IM = False THEN 'False = 0
    AttribNum = Map(WorldX, WorldY, 2)
   ELSEIF IM = 2 THEN  '2 = IconSelection Screen
     AttribNum = ActiveIconsAttrib(ArrowLoc)
END IF

SELECT CASE AttribNum
       
        CASE IS = 0
            Attrib$ = "NO ATTRIB!"
            C = 4
        CASE IS = 1
            Attrib$ = "Land Only"
            C = 2
        CASE IS = 2
            Attrib$ = "Water Only"
            C = 9
        CASE IS = 3
            Attrib$ = "Event Passable"
            C = 5
        CASE IS = 4
            Attrib$ = "Event Impassable"
            C = 3
        CASE IS = 5
            Attrib$ = "Impassable"
            C = 7
        CASE IS = 6
            Attrib$ = "Same"
            C = 6
        CASE IS = 7
            Attrib$ = "Half See"
            C = 10
        CASE IS = 8
            Attrib$ = "No See"
            C = 11
        CASE IS = 9
            Attrib$ = "AnyPass"
            C = 15
END SELECT
LINE (235, 185)-(319, 199), 0, BF
IF AttribNum = 0 THEN BEEP
CALL Textfont(Attrib$, 235, 190, C, 0)
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
IF Col = 250 THEN Col = 0 '250 is used instead of Black for Col 0
PSET (285, 72): DRAW "c15;r5h2f2g2"
LINE (1, 183)-(200, 200), 0, BF
CALL Textfont("SPACE BAR to Select color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 140, 188, 15, 0)
DO
B$ = INKEY$
Y = 1
FOR T = Col - 5 TO Col + 5
  Y = Y + 1
  IF T = -1 THEN
     LINE (300, 60 - (10 * Col))-(305, 60 - (10 * Col) + 5), 0, B
       ELSE
         LINE (300, Y * 10)-(305, Y * 10 + 5), T, BF
  END IF
NEXT T
IF B$ = CHR$(0) + "P" AND Col <> 255 THEN
   Col = Col + 1
     ELSE IF B$ = CHR$(0) + "H" AND Col <> -1 THEN Col = Col - 1
END IF
LINE (244, 14)-(276, 36), 15, B
IF Col <> -1 THEN
  LINE (245, 15)-(275, 35), Col, BF
  LOCATE 1, 31: PRINT Col; "   "
   ELSEIF Col = -1 THEN
      LINE (245, 15)-(275, 35), 0, BF
      PSET (260, 25), 15
      LOCATE 1, 31: PRINT "CLEAR"
END IF




LOOP UNTIL UCASE$(B$) = "Q" OR UCASE$(B$) = " "

IF Col = 0 THEN Col = 250  'Col 250 is used instead of Black for 0
PSET (285, 72): DRAW "c0;r5h2f2g2"
LINE (1, 183)-(200, 200), 0, BF
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)

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

SUB WalkThrough
CLS
TGrid = Grid: Grid = 0
NWorldX = WorldX: NWorldY = WorldY
Trigger = False
CurrentDir = Down
Mode = 3
CALL DispScreen
GOSUB ShowArrow
GOSUB PrintMode
CALL PrintAttrib(False)
DO
LINE (5 * (20 + Grid) + Grid, 4 * (20 + Grid) + Grid)-STEP(21, 21), 15, B
B$ = INKEY$
SELECT CASE B$

       CASE IS = CHR$(0) + ";"  'F1
          CALL Help(3)
      
       CASE IS = " "
          Trigger = True
          SELECT CASE CurrentDir
            CASE IS = UP
              IF CurrentDir <> UP THEN CurrentDir = UP
              IF WorldY = 1 THEN
                NWorldY = MapDimY
                NWorldX = WorldX
               ELSE
                NWorldY = WorldY - 1
                NWorldX = WorldX
              END IF
             
             CASE IS = Down
              IF CurrentDir <> Down THEN CurrentDir = Down
               IF WorldY = MapDimY THEN
                 NWorldY = 1
                 NWorldX = WorldX
                ELSE
                 NWorldY = WorldY + 1
                 NWorldX = WorldX
               END IF
              
             CASE IS = Right
               IF CurrentDir <> Right THEN CurrentDir = Right
                IF WorldX = MapDimX THEN
                    NWorldX = 1
                    NWorldY = WorldY
                    ELSE
                    NWorldX = WorldX + 1
                    NWorldY = WorldY
                END IF
            
             CASE IS = Left
               IF CurrentDir <> Left THEN CurrentDir = Left
                IF WorldX = 1 THEN
                    NWorldX = MapDimX
                    NWorldY = WorldY
                    ELSE
                    NWorldX = WorldX - 1
                    NWorldY = WorldY
                END IF
        
          END SELECT
          GOSUB CheckDir


       CASE IS = CHR$(0) + "H"    'UP
         IF CurrentDir <> UP THEN CurrentDir = UP
         IF WorldY = 1 THEN
            NWorldY = MapDimY
            NWorldX = WorldX
           ELSE
             NWorldY = WorldY - 1
             NWorldX = WorldX
          END IF
         GOSUB CheckDir
     
      CASE IS = CHR$(0) + "P"    'DOWN
        IF CurrentDir <> Down THEN CurrentDir = Down
         IF WorldY = MapDimY THEN
            NWorldY = 1
            NWorldX = WorldX
           ELSE
             NWorldY = WorldY + 1
             NWorldX = WorldX
          END IF
         GOSUB CheckDir
     
      CASE IS = CHR$(0) + "K"    'LEFT
      IF CurrentDir <> Left THEN CurrentDir = Left
        IF WorldX = 1 THEN
            NWorldX = MapDimX
            NWorldY = WorldY
           ELSE
             NWorldX = WorldX - 1
             NWorldY = WorldY
          END IF
        GOSUB CheckDir
        
   
      CASE IS = CHR$(0) + "M"    'RIGHT
      IF CurrentDir <> Right THEN CurrentDir = Right
        IF WorldX = MapDimX THEN
            NWorldX = 1
            NWorldY = WorldY
           ELSE
             NWorldX = WorldX + 1
             NWorldY = WorldY
          END IF
        GOSUB CheckDir
     
      CASE IS = "m", "M"
        IF Mode = 1 THEN
            Mode = 2
          ELSEIF Mode = 2 THEN
              Mode = 3
            ELSEIF Mode = 3 THEN
                Mode = 1
        END IF
        GOSUB PrintMode


END SELECT

IF Trigger = True THEN
 
  IF Map(NWorldX, NWorldY, 2) = 4 THEN
     LINE (238, 120)-(319, 138), 0, BF
     CALL Textfont("Event Triggered", 240, 130, 5, 0)
     WHILE INKEY$ = "": WEND
     LINE (238, 120)-(319, 138), 0, BF
   ELSE
     SOUND 56, .5
   END IF
 
Trigger = False
END IF

LOCATE 1, 31: PRINT "X:"; WorldX; " "
LOCATE 2, 31: PRINT "Y:"; WorldY; " "
LOCATE 3, 31: PRINT "NX:"; NWorldX; " "
LOCATE 4, 31: PRINT "NY:"; NWorldY; " "
LOCATE 5, 31: PRINT "NA:"; Map(NWorldX, NWorldY, 2); " "
LOOP UNTIL (UCASE$(B$) = "Q") OR (B$ = CHR$(27)) 'Q or ESC

Grid = TGrid
CLS
CALL DispScreen
CALL RefreshIcons
CALL DispText
EXIT SUB 'Leaves Sub


CheckDir: 'Gosub Label
ValidDir = False
SELECT CASE Mode
         CASE IS = 1  'Land
             SELECT CASE Map(NWorldX, NWorldY, 2)
               
                CASE IS = 0
                  BEEP
               
                CASE IS = 1, 9
                  ValidDir = True

                CASE IS = 2, 4, 5
                  ValidDir = False

                CASE IS = 3
                  ValidDir = False
                  LINE (238, 120)-(319, 138), 0, BF
                  CALL Textfont("Event Passable", 240, 130, 5, 0)
                  WHILE INKEY$ = "": WEND
                  LINE (238, 120)-(319, 138), 0, BF

             END SELECT

         CASE IS = 2  'Water
             SELECT CASE Map(NWorldX, NWorldY, 2)
               
                CASE IS = 0
                  BEEP

                CASE IS = 1, 4, 5
                  ValidDir = False

                CASE IS = 2, 9
                  ValidDir = True

                CASE IS = 3
                  ValidDir = False
                  LINE (238, 120)-(319, 138), 0, BF
                  CALL Textfont("Event Passable", 240, 130, 5, 0)
                  WHILE INKEY$ = "": WEND
                  LINE (238, 120)-(319, 138), 0, BF

             END SELECT

         CASE IS = 3  'Air
             SELECT CASE Map(NWorldX, NWorldY, 2)
                CASE IS = 0
                  BEEP
               
                CASE IS = 1, 2, 4, 5, 9
                  ValidDir = True

                CASE IS = 3
                  ValidDir = True
                  LINE (238, 120)-(319, 138), 0, BF
                  CALL Textfont("Event Passable", 240, 130, 5, 0)
                  WHILE INKEY$ = "": WEND
                  LINE (238, 120)-(319, 138), 0, BF

             END SELECT


END SELECT

IF (ValidDir = True) AND (Trigger = False) THEN
WorldX = NWorldX
WorldY = NWorldY
ELSE IF Trigger = False THEN SOUND 56, .5
END IF
LOCATE 3, 31: PRINT "NX:"; NWorldX; " "
LOCATE 4, 31: PRINT "NY:"; NWorldY; " "

CALL DispScreen
GOSUB ShowArrow
RETURN

PrintMode: 'Gosub Label
LINE (238, 155)-(319, 168), 0, BF
SELECT CASE Mode
     CASE IS = 1
        CALL Textfont("Land Mode", 240, 160, 2, 0)

     CASE IS = 2
        CALL Textfont("Boat Mode", 240, 160, 3, 0)

     CASE IS = 3
        CALL Textfont("Air Mode", 240, 160, 9, 0)

END SELECT
RETURN

ShowArrow:

SELECT CASE CurrentDir
      CASE IS = UP
          PSET (111, 93), 15
          DRAW "C15;u5;g2e2f2"

      CASE IS = Down
          PSET (111, 88), 15
          DRAW "C15;d5;h2f2e2"
     
      CASE IS = Right
          PSET (109, 90), 15
          DRAW "C15;r5;h2f2g2"
     
      CASE IS = Left
          PSET (113, 90), 15
          DRAW "C15;l5;e2g2f2"

END SELECT
RETURN

END SUB
