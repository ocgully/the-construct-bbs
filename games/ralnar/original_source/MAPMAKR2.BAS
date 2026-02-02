OPTION BASE 1
DECLARE SUB BlockHelp ()
DECLARE SUB IconSelectHelp ()
DECLARE SUB NewIcon ()
DECLARE SUB SaveMapLocation ()
DECLARE SUB BlockAttribScreen ()
DECLARE SUB Quit ()
DECLARE SUB ClearPic ()
DECLARE SUB Help ()
DECLARE SUB Load ()
DECLARE SUB Save ()
DECLARE SUB Textfont (S$, X!, Y!, C!, S!)
DECLARE SUB IconSelection ()
DECLARE SUB RefreshIcons ()
DECLARE SUB OpeningScreen ()
DECLARE SUB MoveIconBoxLocator ()
DECLARE SUB MakeIconPlaceScreen ()
CONST Ver$ = "Ver 1.1"
CONST False = 0
CONST True = 1
CONST IconPath$ = "E:\QB\MMIS\" 'C:\BG_RPG\BASIC\PICS\"
CONST ScreenPath$ = "E:\QB\SCREENS\" 'C:\BG_RPG\BASIC\Screens\"
CONST Land = 1
CONST Water = 2
CONST Slow = 3
CONST Damaging = 4 'Damaging is slow as well
CONST Impassible = 5
CONST Town = 6
CONST Cave = 7
CONST Dock = 8
CONST AnyPass = 9

COMMON SHARED IconLoc, TotalIcons, ArrowLoc, Initial, Done, GrLocX, GrLocY, OGrLocX, OGrLocY, SelectedIconAttrib
Done = False
Initial = True
IconLoc = 1
ArrowLoc = 1
SCREEN 13
CLS
CALL OpeningScreen
REM $DYNAMIC
DIM SHARED Pic(204, 10, 9) AS INTEGER
DIM SHARED IconNames(100) AS STRING 'MAKE LOF(PicFiles.txt)
DIM SHARED ActiveIcons(203, 5) AS INTEGER
DIM SHARED SelectedIcon(203) AS INTEGER
DIM SHARED BlockAttributes(10, 9) AS INTEGER
DIM SHARED BlockUsed(10, 9)AS INTEGER
DIM SHARED ActiveIconsAttrib(5) AS INTEGER

SelectedIconAttrib = 1

COLOR 7: LOCATE 1, 1: PRINT "Loading..."
FOR X = 1 TO 10
  FOR Y = 1 TO 9
     BlockAttributes(X, Y) = 1
  NEXT Y
NEXT X

T = 0
PATHNAME$ = IconPath$ + "MMIFILES.TXT"
OPEN PATHNAME$ FOR INPUT AS #1
    WHILE NOT EOF(1)
       T = T + 1
       INPUT #1,Temp$
       IconNames(T) = Temp$
    WEND
TotalIcons = T
CLOSE #1
COLOR 15


GrLocX = 0 'Grid Location (X Coordinate)
GrLocY = 0 'Grid Location (Y Coordinate)
OGrLocY = 0
OGrLocX = 0

CALL MakeIconPlaceScreen
CALL MoveIconBoxLocator

DO
ChangedGr = False
A$ = INKEY$
SELECT CASE A$
    CASE IS = CHR$(0) + "H"    'UP
       IF GrLocY <> 0 THEN
          OGrLocY = GrLocY
          OGrLocX = GrLocX
          GrLocY = GrLocY - 1
          ChangedGr = True
       ELSEIF GrLocY = 0 THENBEEP
       END IF

    CASE IS = CHR$(0) + "P"    'DOWN
       IF GrLocY <> 8 THEN
          OGrLocY = GrLocY
          OGrLocX = GrLocX
          GrLocY = GrLocY + 1
          ChangedGr = True
       ELSEIF GrLocY = 8 THENBEEP
       END IF

    CASE IS = CHR$(0) + "M"    'RIGHT
       IF GrLocX <> 9 THEN
          OGrLocX = GrLocX
          OGrLocY = GrLocY
          GrLocX = GrLocX + 1
          ChangedGr = True
       ELSEIF GrLocX = 9 THENBEEP
       END IF
   
    CASE IS = CHR$(0) + "K"    'LEFT
       IF GrLocX <> 0 THEN
          OGrLocX = GrLocX
          OGrLocY = GrLocY
          GrLocX = GrLocX - 1
          ChangedGr = True
       ELSEIF GrLocX = 0 THENBEEP
       END IF

    CASE IS = "I", "i"
       CALL IconSelection

    CASE IS = " "
           PUT ((GrLocX * 21 + 2), (GrLocY * 21 + 2)), SelectedIcon, PSET
        ' For Transparent:
           IF BlockUsed(GrLocX + 1, GrLocY + 1) = True THEN
           PUT (250, 100), Pic(1, GrLocX + 1, GrLocY + 1), PSET
           FOR X = 1 TO 20  ' ^^^ Haven't yet stored new icon in Pic.
             FOR Y = 1 TO 20
               IF POINT((GrLocX * 21 + 1) + X, (GrLocY * 21 + 1) + Y) = 0 THEN
                  C! = POINT(249 + X, 99 + Y)
                  PSET ((GrLocX * 21 + 1) + X, (GrLocY * 21 + 1) + Y), C!
               END IF
             NEXT Y
           NEXT X
           PUT (270, 170), Pic(1, GrLocX + 1, GrLocY + 1), XOR
           GET ((GrLocX * 21 + 2), (GrLocY * 21 + 2))-((GrLocX * 21 + 22), (GrLocY * 21 + 22)), Pic(1, GrLocX + 1, GrLocY + 1)
        '  ^^^ Stores in array
        ' End of Tranparent
           ELSE
           BlockUsed(GrLocX + 1, GrLocY + 1) = True
           GET (236, 21)-(256, 41), Pic(1, GrLocX + 1, GrLocY + 1)
        '  ^^^ Stores in array.
           END IF
           BlockAttributes(GrLocX + 1, GrLocY + 1) = SelectedIconAttrib
   
    CASE IS = "S", "s"
      CALL Save
   
    CASE IS = "L", "l", "O", "o"
      CALL Load

    CASE IS = "H", "h"
      CALL Help

    CASE IS = "e", "E"
      CALL ClearPic

    CASE IS = "q", "Q"
      CALL Quit

    CASE IS = "B", "b"
      CALL BlockAttribScreen
      CALL IconSelection

    CASE IS = "G", "g"
      FOR X = 1 TO 203
           IF BlockUsed(GrLocX + 1, GrLocY + 1) THEN
             SelectedIcon(X) = Pic(X, GrLocX + 1, GrLocY + 1)
             PUT (236, 21), SelectedIcon, PSET
           END IF
      NEXT X
    SelectedIconAttrib = BlockAttributes(GrLocX + 1, GrLocY + 1)

    CASE IS = "n", "N"
      CALL NewIcon

END SELECT
IF ChangedGr = True THEN CALL MoveIconBoxLocator

LOCATE 1, 28: PRINT "("; GrLocX + 1; ","; GrLocY + 1; ")  "


LOOP UNTIL Done = True
CLS

DEFSNG A-Z
SUB BlockAttribScreen
LINE (230, 1)-(320, 200), 0, BF
CALL Textfont("Block Attributes", 230, 20, 7, 0)
CALL Textfont("(1) Land", 250, 30, 7, 0)
CALL Textfont("(2) Water", 250, 40, 7, 0)
CALL Textfont("(3) Slow", 250, 50, 7, 0)
CALL Textfont("(4) Damaging", 250, 60, 7, 0)
CALL Textfont("(5) Impassable", 250, 70, 7, 0)
CALL Textfont("(6) Town", 250, 80, 7, 0)
CALL Textfont("(7) Cave", 250, 90, 7, 0)
CALL Textfont("(8) Dock", 250, 100, 7, 0)
CALL Textfont("(9) Passable to All", 250, 110, 7, 0)
SELECT CASE BlockAttributes(GrLocX + 1, GrLocY + 1)
        CASE IS = 1
            Attrib$ = "Land Passable Only"
        CASE IS = 2
            Attrib$ = "Water Passage Only"
        CASE IS = 3
            Attrib$ = "Slow Movement"
        CASE IS = 4
            Attrib$ = "Damaging Area"
        CASE IS = 5
            Attrib$ = "Impassable"
        CASE IS = 6
            Attrib$ = "Town"
        CASE IS = 7
            Attrib$ = "Cave"
        CASE IS = 8
            Attrib$ = "Dock"
        CASE IS = 9
            Attrib$ = "Passable by All"
END SELECT
CALL Textfont("Current Block", 220, 150, 7, 0)
CALL Textfont(Attrib$ + "                    ", 220, 160, 7, 0)


DO
ChangedGr = False
A$ = INKEY$
SELECT CASE A$
    CASE IS = CHR$(0) + "H"    'UP
       IF GrLocY <> 0 THEN
          OGrLocY = GrLocY
          OGrLocX = GrLocX
          GrLocY = GrLocY - 1
          ChangedGr = True
       ELSEIF GrLocY = 0 THENBEEP
       END IF

    CASE IS = CHR$(0) + "P"    'DOWN
       IF GrLocY <> 8 THEN
          OGrLocY = GrLocY
          OGrLocX = GrLocX
          GrLocY = GrLocY + 1
          ChangedGr = True
       ELSEIF GrLocY = 8 THENBEEP
       END IF

    CASE IS = CHR$(0) + "M"    'RIGHT
       IF GrLocX <> 9 THEN
          OGrLocX = GrLocX
          OGrLocY = GrLocY
          GrLocX = GrLocX + 1
          ChangedGr = True
       ELSEIF GrLocX = 9 THENBEEP
       END IF
  
    CASE IS = CHR$(0) + "K"    'LEFT
       IF GrLocX <> 0 THEN
          OGrLocX = GrLocX
          OGrLocY = GrLocY
          GrLocX = GrLocX - 1
          ChangedGr = True
       ELSEIF GrLocX = 0 THENBEEP
       END IF

    CASE IS = "I", "i"
       A$ = CHR$(27)

    CASE IS = "H", "h"
      CALL BlockHelp

    CASE IS = "q", "Q"
      CALL Quit

    CASE IS = "1", "2", "3", "4", "5", "6", "7", "8", "9"
      BlockAttributes(GrLocX + 1, GrLocY + 1) = ASC(A$) - ASC("0")
      ChangedGr = True

END SELECT
IF ChangedGr = True THEN
     CALL MoveIconBoxLocator
     SELECT CASE BlockAttributes(GrLocX + 1, GrLocY + 1)
        CASE IS = 1
            Attrib$ = "Land Passable Only"
        CASE IS = 2
            Attrib$ = "Water Passage Only"
        CASE IS = 3
            Attrib$ = "Slow Movement"
        CASE IS = 4
            Attrib$ = "Damaging Area"
        CASE IS = 5
            Attrib$ = "Impassable"
        CASE IS = 6
            Attrib$ = "Town"
        CASE IS = 7
            Attrib$ = "Cave"
        CASE IS = 8
            Attrib$ = "Dock"
        CASE IS = 9
            Attrib$ = "Passable by All"
     END SELECT
     LINE (219, 156)-(320, 170), 0, BF
     CALL Textfont(Attrib$ + "                    ", 220, 160, 7, 0)
END IF

LOCATE 1, 28: PRINT "("; GrLocX + 1; ","; GrLocY + 1; ")  "


LOOP UNTIL (Done = True) OR (A$ = CHR$(27))

END SUB

SUB BlockHelp
LINE (1, 1)-(211, 190), 7, B
LINE (2, 2)-(210, 189), 0, BF
CALL Textfont("* Block Attribute Help *", 43, 20, 7, 0)
CALL Textfont("(#) Sets Attribute to # ", 40, 60, 7, 0)
CALL Textfont("(Up) Moves Up One Block", 40, 70, 7, 0)
CALL Textfont("(Down) Moves Down One Block", 40, 80, 7, 0)
CALL Textfont("(Right) Moves Right One Block", 40, 90, 7, 0)
CALL Textfont("(Left) Moves Left One Block", 40, 100, 7, 0)
CALL Textfont("(H) Block Attribute Help", 40, 110, 7, 0)
CALL Textfont("(Q) Exits Program", 40, 120, 7, 0)
CALL Textfont("(ESC/I) Returns to Main Program", 40, 130, 7, 0)

DO
LOOP UNTIL INKEY$ <> ""
CALL MakeIconPlaceScreen


END SUB

SUB ClearPic
  LINE (1, 1)-(211, 190), 7, B
  LINE (2, 2)-(210, 189), 0, BF
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
      FOR X = 1 TO 10
        FOR Y = 1 TO 9
          BlockUsed(X, Y) = False 'THIS CLEARS THE PICTURE
        NEXT Y                    'FILE AND MAKES EVERYTHING
      NEXT X                      'BLACK.
      FOR X = 1 TO 10
        FOR Y = 1 TO 9
           BlockAttributes(X, Y) = 1
        NEXT Y
      NEXT X
   END IF


CALL MakeIconPlaceScreen

END SUB

SUB Help
LINE (1, 1)-(211, 190), 7, B
LINE (2, 2)-(210, 189), 0, BF
CALL Textfont("* Help *", 83, 20, 7, 0)
CALL Textfont("(E) Clears Screen", 40, 50, 7, 0)
CALL Textfont("(Space) Places/Selects Icon ", 40, 60, 7, 0)
CALL Textfont("(G) Get Icon corsor is on", 40, 70, 7, 0)
CALL Textfont("(I) Select Icons", 40, 80, 7, 0)
CALL Textfont("(S) Saves Screen as Filename", 40, 90, 7, 0)
CALL Textfont("(L) Opens Previous Screen", 40, 100, 7, 0)
CALL Textfont("(B) Block Attribute Screen", 40, 110, 7, 0)
CALL Textfont("(H) This Help Screen", 40, 120, 7, 0)
CALL Textfont("(Q) Exits Program", 40, 130, 7, 0)

DO
LOOP UNTIL INKEY$ <> ""
CALL MakeIconPlaceScreen

END SUB

SUB IconSelectHelp
LINE (1, 1)-(211, 190), 7, B
LINE (2, 2)-(210, 189), 0, BF
CALL Textfont("* Icon Selection Help *", 43, 20, 7, 0)
CALL Textfont("(Space) Selects Icon ", 40, 60, 7, 0)
CALL Textfont("(Up) Move Up One Icon", 40, 70, 7, 0)
CALL Textfont("(Down) Move Down One Icon", 40, 80, 7, 0)
CALL Textfont("(Pg Up) Moves Up 5 Icons", 40, 90, 7, 0)
CALL Textfont("(Pg Dn) Moves Down 5 Icons", 40, 100, 7, 0)
CALL Textfont("(H) Icon Selection Help", 40, 110, 7, 0)
CALL Textfont("(Q) Exits Program", 40, 120, 7, 0)

DO
LOOP UNTIL INKEY$ <> ""
CALL MakeIconPlaceScreen

END SUB

SUB IconSelection
LINE (220, 10)-(320, 200), 0, BF
CALL RefreshIcons
IF SelectedIcon(1) MOD 100 = 60 THEN PUT (236, 21), SelectedIcon, PSET
DO
B$ = INKEY$
SELECT CASE B$
   CASE IS = "H", "h"
     CALL IconSelectHelp
     
   CASE IS = "q", "Q"
      CALL Quit

   CASE IS = " "

         FOR X = 1 TO 203
            SelectedIcon(X) = ActiveIcons(X, ArrowLoc)
         NEXT X
         SelectedIconAttrib = ActiveIconsAttrib(ArrowLoc)
  
   CASE IS = CHR$(0) + "H"  'UP
     IF IconLoc <> 1 THEN
        IconLoc = IconLoc - 1
        ELSE BEEP
     END IF
     IF (ArrowLoc = 3) AND ((IconLoc - 2 = 0) OR (IconLoc - 1 = 0)) THEN
         ArrowLoc = ArrowLoc - 1
         Changed = 1
     END IF
     IF (ArrowLoc = 3) THEN CALL RefreshIcons
     IF (Changed = 0) AND (ArrowLoc <> 1) AND (ArrowLoc <> 3) THEN ArrowLoc = ArrowLoc - 1
     Changed = 0
  
   CASE IS = CHR$(0) + "P"  'DOWN
     IF IconLoc <> TotalIcons THEN
        IconLoc = IconLoc + 1
        ELSE BEEP
     END IF
     IF (ArrowLoc = 3) AND (IconLoc + 1 = TotalIcons) THEN
         ArrowLoc = ArrowLoc + 1
         Changed = 1
     END IF
     IF (ArrowLoc = 3) THEN CALL RefreshIcons
     IF (Changed = 0) AND (ArrowLoc <> 5) AND ((ArrowLoc <> 3) AND (IconLoc <> TotalIcons + 2)) THEN ArrowLoc = ArrowLoc + 1
     Changed = 0

   CASE IS = CHR$(0) + "I" 'PAGE UP
     IF IconLoc > 7 THEN
        IconLoc = IconLoc - 5
        ELSE IF IconLoc < 4 THEN BEEP
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
    
   CASE IS = CHR$(0) + "Q" 'PAGE DOWN
     IF IconLoc < TotalIcons - 6 THEN
        IconLoc = IconLoc + 5
        ELSE IF IconLoc > TotalIcons - 3 THEN BEEP
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

LOOP UNTIL (B$ = " ") OR Done = True
IF B$ = " " THEN
PUT (236, 21), SelectedIcon, PSET
END IF
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

END SUB

SUB Load
LINE (1, 1)-(211, 190), 7, B
LINE (2, 2)-(210, 189), 0, BF
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
  Name$ = Name$ + ".SCR"

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
  PATHNAME$ = ScreenPath$ + Name$
  IF UCASE$(YesNo$) = "Y" THEN
    CALL Textfont("Now Loading...", 25, 90, 7, 0)
    OPEN PATHNAME$ FOR RANDOM AS #1 LEN = LEN(BlockAttributes(1, 1))
    FOR X = 0 TO 9
      FOR Y = 0 TO 8
        FOR P = 1 TO 203
           GET #1, (X * 1836) + (Y * 204) + P, Pic(P, X + 1, Y + 1)
        NEXT P
        GET #1, (X * 1836) + (Y * 204) + 204, BlockAttributes(X + 1, Y + 1)
      NEXT Y
    NEXT X
    CLOSE #1
    CALL Textfont("File Loaded!", 25, 100, 7, 0)
  END IF


WHILE INKEY$ = "": WEND
END IF
CALL MakeIconPlaceScreen

END SUB

SUB MakeIconPlaceScreen
CLS
FOR X = 1 TO 211 STEP 21
  LINE (X, 1)-(X, 190), 7
NEXT X
FOR Y = 1 TO 190 STEP 21
  LINE (1, Y)-(211, Y), 7
NEXT Y


'  This can be used to place an icon in a location when the coordinates are
'  (XX,YY) on Screen Grid.
YC = XC = 0
IF Initial = False THEN
    FOR XX = 0 TO 9
      FOR YY = 0 TO 8
        IF BlockUsed(XX + 1, YY + 1) = True THEN
           PUT ((XX * 21 + 2), (YY * 21 + 2)), Pic(1, XX + 1, YY + 1), OR
         ELSE
           LINE ((XX * 21 + 2), (YY * 21 + 2))-STEP(19, 19), 0, BF
        END IF
      NEXT YY
    NEXT XX
END IF
Initial = False
LINE (235, 20)-(256, 41), 7, B

CALL IconSelection
END SUBXXYY

SUB MoveIconBoxLocator
LINE (OGrLocX * 21 + 1, OGrLocY * 21 + 1)-(OGrLocX * 21 + 22, OGrLocY * 21 + 22), 7, B
LINE (GrLocX * 21 + 1, GrLocY * 21 + 1)-(GrLocX * 21 + 22, GrLocY * 21 + 22), 15, B
END SUB

SUB NewIcon
LINE (1, 1)-(211, 190), 7, B
LINE (2, 2)-(210, 189), 0, BF
IF BlockUsed(GrLocX + 1, GrLocY + 1) = True THEN
'Then do from HERE
CALL Textfont("* SAVE NEW ICON *", 83, 20, 7, 0)
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
  Name$ = Name$ + ".MMI"

  CALL Textfont("Saving Icon as: " + Name$, 25, 60, 7, 0)
  CALL Textfont("Is this Correct? (Y/N): ", 25, 80, 7, 0)

  DO
  YesNo$ = INKEY$
   IF UCASE$(YesNo$) = "Y" THEN
     CALL Textfont("Y", 140, 80, 7, 1)
       ELSEIF UCASE$(YesNo$) = "N" THEN
         CALL Textfont("N", 140, 80, 7, 1)
   END IF
  LOOP UNTIL (UCASE$(YesNo$) = "Y") OR (UCASE$(YesNo$) = "N")
PATHNAME$ = IconPath$ + Name$
  IF UCASE$(YesNo$) = "Y" THEN
    OPEN PATHNAME$ FOR RANDOM AS #1 LEN = LEN(SelectedIconAttribute)
    FOR X = 1 TO 203
      PUT #1, X, Pic(X, GrLocX, GrLocY)
    NEXT X
    PUT #1, 204, BlockAttributes(GrLocX + 1, GrLocY + 1)
    CLOSE #1
  END IF
NewName = 1
OPEN IconPath$ + "MMIFILES.TXT" FOR INPUT AS #1
    WHILE NOT EOF(1)
       INPUT #1,Test$
       IF UCASE$(Test$) = UCASE$(Name$) THEN NewName = 0
    WEND
CLOSE #1

IF NewName = 1 THEN
  OPEN IconPath$ + "MMIFILES.TXT" FOR APPEND AS #1
    WRITE #1, Name$
  CLOSE #1
  CALL Textfont("New Icon Name!", 25, 90, 7, 0)
END IF

PATHNAME$ = IconPath$ + "MMIFILES.TXT"
OPEN PATHNAME$ FOR INPUT AS #1
    T = 0
    WHILE NOT EOF(1)
       T = T + 1
       INPUT #1,Temp$
       IconNames(T) = Temp$
    WEND
CLOSE #1
TotalIcons = T
IconLoc = 1
ArrowLoc = 1
CALL Textfont("Icon Saved!", 25, 100, 7, 0)
END IF ' For IF name$ <> ""
'To HERE
ELSE
' If False:
CALL Textfont("That is a black square!", 83, 20, 7, 0)
' To HERE
END IF
'Do in any situation:
WHILE INKEY$ = "": WEND
CALL MakeIconPlaceScreen

END SUB

SUB OpeningScreen
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

'FOR T = 1 TO 3
'T1$ = TIME$
'WHILE T1$ = TIME$: WEND   '3 second delay timer
'NEXT T
END SUB

SUB Quit
  LINE (1, 1)-(211, 190), 7, B
  LINE (2, 2)-(210, 189), 0, BF
  CALL Textfont("* QUIT!? *", 75, 20, 7, 0)
  CALL Textfont("Are you sure you want ", 25, 60, 7, 0)
  CALL Textfont("to quit now? (Y/N): ", 25, 70, 7, 0)
  DO
  YesNo$ = INKEY$
   IF UCASE$(YesNo$) = "Y" THEN
     CALL Textfont("Y", 140, 80, 7, 1)
       ELSEIF UCASE$(YesNo$) = "N" THEN
         CALL Textfont("N", 140, 80, 7, 1)
   END IF
   LOOP UNTIL (UCASE$(YesNo$) = "Y") OR (UCASE$(YesNo$) = "N")
   IF UCASE$(YesNo$) = "Y" THEN Done = True ELSE IF UCASE$(YesNo$) = "N" THEN CALL MakeIconPlaceScreen

END SUB

SUB RefreshIcons
Count = 0


IF ArrowLoc = 3 THEN
  FOR T = IconLoc - 2 TO IconLoc + 2
    FileName$ = IconPath$ + IconNames(T)
    OPEN FileName$ FOR RANDOM AS #1 LEN = LEN(ActiveIconsAttrib(1))
    Count = Count + 1
      FOR X = 1 TO 203
        GET #1, X, ActiveIcons(X, Count)
      NEXT X
    PUT (291, (Count * 25) + 21), ActiveIcons(1, Count), PSET
    GET #1, 204, ActiveIconsAttrib(Count)
    CLOSE #1
    LINE (289, 19 + (Count * 25))-(311, 42 + (Count * 25)), 7, B
 NEXT T

  ELSE
  FOR T = IconLoc - ArrowLoc + 1 TO IconLoc + 5 - ArrowLoc
    FileName$ = IconPath$ + IconNames(T)
    OPEN FileName$ FOR RANDOM AS #1 LEN = LEN(ActiveIconsAttrib(1))
    Count = Count + 1
       FOR X = 1 TO 203
         GET #1, X, ActiveIcons(X, Count)
       NEXT X
    PUT (291, (Count * 25) + 21), ActiveIcons(1, Count), PSET
    GET #1, 204, ActiveIconsAttrib(Count)
    CLOSE #1
    LINE (289, 19 + (Count * 25))-(311, 42 + (Count * 25)), 7, B
 NEXT T
END IF
END SUB


SUB Save
'LINE (1, 183)-(200, 200), 0, BF
LINE (1, 1)-(211, 190), 7, B
LINE (2, 2)-(210, 189), 0, BF
CALL Textfont("* SAVE *", 83, 20, 7, 0)
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
  Name$ = Name$ + ".SCR"

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
PATHNAME$ = ScreenPath$ + Name$
  IF UCASE$(YesNo$) = "Y" THEN
    CALL Textfont("Now saving...", 25, 90, 7, 0)
    OPEN PATHNAME$ FOR RANDOM AS #1 LEN = LEN(BlockAttributes(1, 1))
    FOR X = 0 TO 9
      FOR Y = 0 TO 8
        FOR P = 1 TO 203
           PUT #1, (X * 1836) + (Y * 204) + P, Pic(P, X + 1, Y + 1)
        NEXT P
        PUT #1, (X * 1836) + (Y * 204) + 204, BlockAttributes(X + 1, Y + 1)
      NEXT Y
    NEXT X
    CLOSE #1
  END IF
'NewName = 1
'OPEN "C:\BG_RPG\BASIC\PICS\PICFILES.TXT" FOR INPUT AS #1
'    WHILE NOT EOF(1)
'       INPUT #1, Test$
'       IF UCASE$(Test$) = UCASE$(Name$) THEN NewName = 0
'    WEND
'CLOSE #1

'IF NewName = 1 THEN
'  OPEN "C:\BG_RPG\BASIC\PICS\PICFILES.TXT" FOR APPEND AS #1
'    WRITE #1, Name$
'  CLOSE #1
'END IF
CALL Textfont("File Saved!", 25, 100, 7, 0)
WHILE INKEY$ = "": WEND
END IF
CALL MakeIconPlaceScreen

END SUB

'SYNTAX: CALL TEXTFONT ("MESSAGE", X, Y, COLOR,Sound[y/n])
'X AND Y are coordinate locations
'
SUB Textfont (S$, X, Y, C, S)

PSET (-1, Y), C
FOR T = -1 TO X: DRAW "br1": NEXT T
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

IF S = 1 AND UCASE$(MID$(S$, T, 1)) <> " " THEN SOUND 55, .1
IF S = 1 AND UCASE$(MID$(S$, T, 1)) <> " " THEN FOR M = 1 TO (1000 * TD): NEXT M

NEXT T



END SUB