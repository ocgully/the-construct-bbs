OPTION BASE 1
DECLARE SUB UpdateIcon ()
DECLARE SUB FlipH (IM AS INTEGER)
DECLARE SUB FlipV (IM AS INTEGER)
DECLARE SUB Rotate (IM AS INTEGER)
DECLARE SUB SaveIcon2 ()
DECLARE SUB LoadIcon2 ()
DECLARE SUB Attributes ()
DECLARE SUB PrintAttrib ()
DECLARE SUB ClearIcon (Initial!)
DECLARE SUB LoadIcon ()
DECLARE SUB SaveIcon ()
DECLARE SUB HelpIconMaker ()
DECLARE SUB SelectCol ()
DECLARE SUB MAKEPICBOX ()
DECLARE SUB MAKEGRID ()
DECLARE SUB DRAWIN (X!, Y!)
DECLARE SUB IconMaker (S$)

'From HERE up = IM Subs

DECLARE SUB GetToICON ()
DECLARE SUB NewNFlag (NFlag$)
DECLARE SUB CoordinateLoad (XCrr$, YCrr$, NFlag$)
DECLARE SUB SetCoordinates (XCrr$, YCrr$)
DECLARE SUB CoordinateSave (XCrr$, YCrr$, NFlag$)
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
DECLARE SUB LoadScr ()
DECLARE SUB SaveScr ()
DECLARE SUB Textfont (S$, X!, Y!, C!, S!)
DECLARE SUB IconSelection ()
DECLARE SUB RefreshIcons ()
DECLARE SUB OpeningScreen ()
DECLARE SUB MoveIconBoxLocator ()
DECLARE SUB MakeIconPlaceScreen ()
CONST Ver$ = "Ver 1.1A"
CONST False = 0
CONST True = 1
CONST PicPATH$ = "C:\BG_RPG\BASIC\PICS\"
CONST IconPath$ = "C:\BG_RPG\BASIC\PICS\"
CONST ScreenPath$ = "C:\BG_RPG\BASIC\Screens\"
CONST Land = 1
CONST Water = 2
CONST Slow = 3
CONST Damaging = 4 'Damaging is slow as well
CONST Impassible = 5
CONST Town = 6
CONST Cave = 7
CONST Dock = 8
CONST AnyPass = 9

COMMON SHARED IconLoc, TotalIcons, ArrowLoc, Initial, Done, GrlocX, GrLocY, OGrLocX, OGrLocY, SelectedIconAttrib, Grid
COMMON SHARED OXCorr, XCorr, OYCorr, YCorr, CHANGE.CORR, Attrib$, IconAttribute AS INTEGER, Col AS INTEGER
'^^^^^ For IconMaker
Done = False
Initial = True
Grid = False
IconLoc = 1
ArrowLoc = 1
XCrr$ = "0"
YCrr$ = "0"
NFlag$ = "M"
SCREEN 13
CLS
CALL OpeningScreen
REM $DYNAMIC
DIM SHARED Pic(205, 10, 9) AS INTEGER
DIM SHARED IconNames(100) AS STRING 'MAKE LOF(PicFiles.txt)
DIM SHARED ActiveIcons(203, 5) AS INTEGER
DIM SHARED SelectedIcon(203) AS INTEGER
DIM SHARED BlockAttributes(10, 9) AS INTEGER
DIM SHARED BlockUsed(10, 9)AS INTEGER
DIM SHARED ActiveIconsAttrib(5) AS INTEGER
DIM SHARED WhiteBox(255) AS INTEGER
DIM SHARED ICON(20, 20) AS INTEGER
SelectedIconAttrib = 1

LINE (0, 100)-STEP(21, 21), 15, B
GET (0, 100)-STEP(21, 21), WhiteBox
PUT (0, 100), WhiteBox, OR

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


GrlocX = 0 'Grid Location (X Coordinate)
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


    CASE IS = CHR$(0) + "P"    'DOWN

    CASE IS = CHR$(0) + "M"    'RIGHT
   
    CASE IS = CHR$(0) + "K"    'LEFT


    CASE IS = "R", "r"
      CALL GetToICON
      CALL Rotate(False)
      FOR X = 1 TO 20
        FOR Y = 1 TO 20
         PSET ((GrlocX * (20 + Grid) + Grid) + X, (GrLocY * (20 + Grid) + Grid) + Y), ICON(X, Y)
        NEXT Y
      NEXT X
      GET ((GrlocX * (20 + Grid) + Grid) + 1, (GrLocY * (20 + Grid) + Grid) + 1)-STEP(19, 19), Pic(1, GrlocX + 1, GrLocY + 1)
     
    CASE IS = "F", "f"
      CALL GetToICON
      CALL FlipH(False)
      FOR X = 1 TO 20
        FOR Y = 1 TO 20
         PSET ((GrlocX * (20 + Grid) + Grid) + X, (GrLocY * (20 + Grid) + Grid) + Y), ICON(X, Y)
        NEXT Y
      NEXT X
      GET ((GrlocX * (20 + Grid) + Grid) + 1, (GrLocY * (20 + Grid) + Grid) + 1)-STEP(19, 19), Pic(1, GrlocX + 1, GrLocY + 1)
    
    CASE IS = "V", "v"
      CALL GetToICON
      CALL FlipV(False)
      FOR X = 1 TO 20
        FOR Y = 1 TO 20
         PSET ((GrlocX * (20 + Grid) + Grid) + X, (GrLocY * (20 + Grid) + Grid) + Y), ICON(X, Y)
        NEXT Y
      NEXT X
      GET ((GrlocX * (20 + Grid) + Grid) + 1, (GrLocY * (20 + Grid) + Grid) + 1)-STEP(19, 19), Pic(1, GrlocX + 1, GrLocY + 1)
    
    CASE IS = CHR$(0) + "<"    'F2
       IF BlockUsed(GrlocX + 1, GrLocY + 1) THEN
         CALL IconMaker("G")
        ELSE
         CALL IconMaker("N")
        END IF
   
    CASE IS = CHR$(0) + "="    'F3
       IF Grid = 1 THEN Grid = 0 ELSE Grid = 1
       CALL MakeIconPlaceScreen
   
    CASE IS = "I", "i"
       CALL IconSelection

    CASE IS = " "
           PUT ((GrlocX * (20 + Grid) + 1 + Grid), (GrLocY * (20 + Grid) + 1 + Grid)), SelectedIcon, PSET
        ' For Transparent:
           IF BlockUsed(GrlocX + 1, GrLocY + 1) = True THEN
           PUT (220, 60), Pic(1, GrlocX + 1, GrLocY + 1), PSET
           FOR X = 1 TO 20  ' ^^^ Haven't yet stored new icon in Pic.
             FOR Y = 1 TO 20
               IF POINT((GrlocX * (20 + Grid) + Grid) + X, (GrLocY * (20 + Grid) + Grid) + Y) = 0 THEN
                  C! = POINT(219 + X, 59 + Y)
                  PSET ((GrlocX * (20 + Grid) + Grid) + X, (GrLocY * (20 + Grid) + Grid) + Y), C!
               END IF
             NEXT Y
           NEXT X
           PUT (220, 60), Pic(1, GrlocX + 1, GrLocY + 1), XOR
           GET ((GrlocX * (20 + Grid) + 1 + Grid), (GrLocY * (20 + Grid) + 1 + Grid))-STEP(19, 19), Pic(1, GrlocX + 1, GrLocY + 1)
        '  ^^^ Stores in array
        ' End of Tranparent
           ELSE
           BlockUsed(GrlocX + 1, GrLocY + 1) = True
           GET (236, 21)-(255, 40), Pic(1, GrlocX + 1, GrLocY + 1)
        '  ^^^ Stores in array.
           END IF
           BlockAttributes(GrlocX + 1, GrLocY + 1) = SelectedIconAttrib
   
    CASE IS = "S", "s"
      CALL SaveScr
   
    CASE IS = "L", "l", "O", "o"
      CALL LoadScr

    CASE IS = "Z", "z"
      CALL CoordinateSave(XCrr$, YCrr$, NFlag$)

    CASE IS = "M", "m"
      CALL CoordinateLoad(XCrr$, YCrr$, NFlag$)
  
    CASE IS = "C", "c"
      CALL SetCoordinates(XCrr$, YCrr$)
   
    CASE IS = "H", "h", CHR$(0) + ";" 'H or F1
      CALL Help

    CASE IS = "e", "E"
      CALL ClearPic

    CASE IS = "q", "Q"
      CALL Quit

    CASE IS = "B", "b"
      CALL BlockAttribScreen
      CALL IconSelection

    CASE IS = "G", "g"
      GET (GrlocX * (20 + Grid) + 1 + Grid, GrLocY * (20 + Grid) + 1 + Grid)-STEP(19, 19), SelectedIcon
      PUT (236, 21), SelectedIcon, PSET
      SelectedIconAttrib = BlockAttributes(GrlocX + 1, GrLocY + 1)
     
    CASE IS = "B", "b"
      CALL NewIcon

    CASE IS = "N", "n"
      CALL NewNFlag(NFlag$)

END SELECT
IF ChangedGr = True THEN CALL MoveIconBoxLocator

LOCATE 1, 28: PRINT "("; GrlocX + 1; ","; GrLocY + 1; ")  "
LOCATE 23, 28: PRINT NFlag$; "("; XCrr$; ","; YCrr$; ")   "

LOOP UNTIL Done = True
CLS

DEFSNG A-Z
SUB Attributes
LINE (1, 183)-(200, 200), 0, BF
LINE (15, 1)-(195, 181), 7, B
LINE (16, 2)-(194, 180), 0, BF
CALL Textfont("Attributes", 50, 20, 7, 0)
CALL Textfont("(1) Land", 50, 30, 7, 0)
CALL Textfont("(2) Water", 50, 40, 7, 0)
CALL Textfont("(3) Slow", 50, 50, 7, 0)
CALL Textfont("(4) Damaging", 50, 60, 7, 0)
CALL Textfont("(5) Impassable", 50, 70, 7, 0)
CALL Textfont("(6) Town", 50, 80, 7, 0)
CALL Textfont("(7) Cave", 50, 90, 7, 0)
CALL Textfont("(8) Dock", 50, 100, 7, 0)
CALL Textfont("(9) Passable to All", 50, 110, 7, 0)
CALL Textfont("Choose an Attribute: ", 25, 130, 7, 0)
X = 125
Choice$ = ""
DO
   S$ = INKEY$
   IF (X < 130) AND (S$ <> "") AND ((S$ >= "1") AND (S$ <= "9")) THEN
      X = X + 5
      CALL Textfont(S$, X, 130, 7, 0)
      Choice$ = Choice$ + S$
   END IF
   IF (S$ = CHR$(8)) AND (Choice$ <> "") THEN
      CALL Textfont("~", X, 50, 0, 0)
      X = X - 5
      Choice$ = MID$(Choice$, 1, LEN(Choice$) - 1)
   END IF

LOOP UNTIL S$ = CHR$(13)



CALL Textfont("Is this Correct? (Y/N): ", 25, 150, 7, 0)
  DO
  YesNo$ = INKEY$
   IF UCASE$(YesNo$) = "Y" THEN
     CALL Textfont("Y", 140, 150, 7, 1)
       ELSEIF UCASE$(YesNo$) = "N" THEN
         CALL Textfont("N", 140, 150, 7, 1)
     END IF
  
   LOOP UNTIL (UCASE$(YesNo$) = "Y") OR (UCASE$(YesNo$) = "N")


IF UCASE$(YesNo$) = "Y" THEN IconAttribute = ASC(Choice$) - ASC("0")

CALL PrintAttrib

LINE (290, 170)-(311, 191), 7, B
CALL MAKEGRID  'Draws Grid to Screen
CALL MAKEPICBOX 'Makes the picture box in the bottom right
CALL SelectCol

END SUB

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
SELECT CASE BlockAttributes(GrlocX + 1, GrLocY + 1)
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
          OGrLocX = GrlocX
          GrLocY = GrLocY - 1
          ChangedGr = True
       ELSEIF GrLocY = 0 THENBEEP
       END IF

    CASE IS = CHR$(0) + "P"    'DOWN
       IF GrLocY <> 8 THEN
          OGrLocY = GrLocY
          OGrLocX = GrlocX
          GrLocY = GrLocY + 1
          ChangedGr = True
       ELSEIF GrLocY = 8 THENBEEP
       END IF

    CASE IS = CHR$(0) + "M"    'RIGHT
       IF GrlocX <> 9 THEN
          OGrLocX = GrlocX
          OGrLocY = GrLocY
          GrlocX = GrlocX + 1
          ChangedGr = True
       ELSEIF GrlocX = 9 THENBEEP
       END IF
  
    CASE IS = CHR$(0) + "K"    'LEFT
       IF GrlocX <> 0 THEN
          OGrLocX = GrlocX
          OGrLocY = GrLocY
          GrlocX = GrlocX - 1
          ChangedGr = True
       ELSEIF GrlocX = 0 THENBEEP
       END IF

    CASE IS = "I", "i"
       A$ = CHR$(27)

    CASE IS = "H", "h"
      CALL BlockHelp

    CASE IS = "q", "Q"
      CALL Quit

    CASE IS = "1", "2", "3", "4", "5", "6", "7", "8", "9"
      BlockAttributes(GrlocX + 1, GrLocY + 1) = ASC(A$) - ASC("0")
      ChangedGr = True

END SELECT
IF ChangedGr = True THEN
     CALL MoveIconBoxLocator
     SELECT CASE BlockAttributes(GrlocX + 1, GrLocY + 1)
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

LOCATE 1, 28: PRINT "("; GrlocX + 1; ","; GrLocY + 1; ")  "


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

SUB ClearIcon (Initial!)
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
LINE (290, 170)-(311, 191), 7, B
CALL MAKEGRID  'Draws Grid to Screen
CALL MAKEPICBOX 'Makes the picture box in the bottom right
CALL SelectCol

OXCorr = 1
OYCorr = 1
XCorr = 1
YCorr = 1
CHANGE.CORR = 0
END IF
CALL PrintAttrib
CALL MAKEGRID
CALL MAKEPICBOX
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)

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

SUB CoordinateLoad (XCrr$, YCrr$, NFlag$)
DO
LOCATE 17, 28: INPUT "NF: ", NFlag$
LOCATE 18, 28: INPUT "X: ", XCrr$
LOCATE 19, 28: INPUT "Y: ", YCrr$
LOCATE 18, 28: PRINT "       "
LOCATE 19, 28: PRINT "       "
LOOP UNTIL (LEN(XCrr$) > 0) AND (LEN(XCrr$) <= 2) AND (LEN(YCrr$) > 0) AND (LEN(YCrr$) <= 2) AND (LEN(NFlag$) > 0) AND (LEN(NFlag$) <= 2)
LINE (1, 1)-(211, 190), 7, B
LINE (2, 2)-(210, 189), 0, BF
CALL Textfont("* Coordinate Load *", 53, 20, 7, 0)
Name$ = NFlag$ + "X" + XCrr$ + "Y" + YCrr$ + ".SCR"

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
    OPEN PATHNAME$ FOR INPUT AS #1
    FOR X = 1 TO 200
      FOR Y = 1 TO 180
        INPUT #1,Temp
        PSET (X, Y), Temp
      NEXT Y
    NEXT X
    FOR X = 1 TO 10
      FOR Y = 1 TO 9
        INPUT #1,BlockAttributes(X, Y)
      NEXT Y
    NEXT X
  CLOSE #1
  FOR X = 1 TO 10
    FOR Y = 1 TO 9
      BlockUsed(X, Y) = True
      GET ((X - 1) * 20 + 1, (Y - 1) * 20 + 1)-STEP(19, 19), Pic(1, X, Y)
    NEXT Y
  NEXT X
  CLS
  CALL Textfont("File Loaded!", 3, 3, 7, 0)
  END IF

WHILE INKEY$ = "": WEND
CALL MakeIconPlaceScreen

END SUB

SUB CoordinateSave (XCrr$, YCrr$, NFlag$)
LINE (1, 1)-(211, 190), 7, B
LINE (2, 2)-(210, 189), 0, BF
CALL Textfont("* Coordinate Save *", 43, 20, 7, 0)
Name$ = NFlag$ + "X" + XCrr$ + "Y" + YCrr$ + ".SCR"

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
    OPEN PATHNAME$ FOR OUTPUT AS #1
    FOR X = 1 TO 10
      FOR Y = 1 TO 9
        IF BlockUsed(X, Y) THEN
          PUT ((X - 1) * 20 + 1, (Y - 1) * 20 + 1), Pic(1, X, Y), OR
         ELSE
          LINE ((X - 1) * 20 + 1, (Y - 1) * 20 + 1)-STEP(19, 19), 0, BF
        END IF
      NEXT Y
    NEXT X
    FOR X = 1 TO 200
      FOR Y = 1 TO 180
        Temp = POINT(X, Y)
        PRINT #1, Temp
      NEXT Y
    NEXT X
    FOR X = 1 TO 10
      FOR Y = 1 TO 9
        PRINT #1, BlockAttributes(X, Y)
      NEXT Y
    NEXT X
  CLOSE #1
  CLS
  CALL Textfont("File Saved!", 3, 3, 7, 0)
  END IF
WHILE INKEY$ = "": WEND
CALL MakeIconPlaceScreen


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
MAKEGRID
MAKEPICBOX
END IF
END SUB

SUB GetToICON
FOR X = 1 TO 20
  FOR Y = 1 TO 20
  ICON(X, Y) = POINT((GrlocX * (20 + Grid) + Grid) + X, (GrLocY * (20 + Grid) + Grid) + Y)
  NEXT Y
NEXT X
END SUB

SUB Help
LINE (1, 1)-(211, 190), 7, B
LINE (2, 2)-(210, 189), 0, BF
CALL Textfont("* Help *", 83, 20, 7, 0)
CALL Textfont("(C) Changes Coordrinates", 40, 40, 7, 0)
CALL Textfont("(E) Clears Screen", 40, 50, 7, 0)
CALL Textfont("(Space) Places/Selects Icon ", 40, 60, 7, 0)
CALL Textfont("(G) Get Icon corsor is on", 40, 70, 7, 0)
CALL Textfont("(I) Select Icons", 40, 80, 7, 0)
CALL Textfont("(S) Saves Screen as Filename", 40, 90, 7, 0)
CALL Textfont("(L) Opens Previous Screen", 40, 100, 7, 0)
CALL Textfont("(B) Block Attribute Screen", 40, 110, 7, 0)
CALL Textfont("(H) This Help Screen", 40, 120, 7, 0)
CALL Textfont("(R) Rotate Icon Under Cursor 90Deg", 40, 130, 7, 0)
CALL Textfont("(F) Flip Icon Under Cursor Across", 40, 140, 7, 0)
CALL Textfont("(V) Flip Icon Under Cursor Up+Down", 40, 150, 7, 0)
CALL Textfont("(F2) Edit Icon Under Cursor ", 40, 160, 7, 0)
CALL Textfont("(Q) Exits Program", 40, 180, 7, 0)
DO
LOOP UNTIL INKEY$ <> ""
CALL MakeIconPlaceScreen

END SUB

SUB HelpIconMaker
LINE (1, 183)-(200, 200), 0, BF
LINE (15, 1)-(195, 181), 7, B
LINE (16, 2)-(194, 180), 0, BF
CALL Textfont("* Help *", 83, 20, 7, 0)
CALL Textfont("(B) Back to MapMaker", 50, 40, 7, 0)
CALL Textfont("(E) Clears Icon", 50, 50, 7, 0)
CALL Textfont("(Space) Places Color", 50, 60, 7, 0)
CALL Textfont("(G) Get Color corsor is on", 50, 70, 7, 0)
CALL Textfont("(C) Select Color", 50, 80, 7, 0)
CALL Textfont("(S) Saves Icon", 50, 90, 7, 0)
CALL Textfont("(O) Opens Previous Icon", 50, 100, 7, 0)
CALL Textfont("(A) Change Attribute", 50, 110, 7, 0)
CALL Textfont("(L) Locks the Color so you", 50, 120, 7, 0)
CALL Textfont("         can Drag it.", 50, 127, 7, 0)
CALL Textfont("(F) Flip Icon Across", 50, 137, 7, 0)
CALL Textfont("(V) Flip Icon Up+Down", 50, 147, 7, 0)
CALL Textfont("(R) Rotate Icon 90deg.", 50, 157, 7, 0)
CALL Textfont("(F) Flip Icon Across", 50, 167, 7, 0)
CALL Textfont("(U) Update Icon on Screen", 50, 177, 7, 0)

DO
LOOP UNTIL INKEY$ <> ""

CALL MAKEGRID
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)
END SUB

SUB IconMaker (S$)

CLS
IF S$ = "G" THEN
  PUT (291, 171), Pic(1, GrlocX + 1, GrLocY + 1), PSET
    FOR X = 1 TO 20
       FOR Y = 1 TO 20
          A = POINT((X + 290), (Y + 170))
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
  PUT (291, 171), ActiveIcons(1, ArrowLoc), OR
    FOR X = 1 TO 20
       FOR Y = 1 TO 20
          A = POINT((X + 290), (Y + 170))
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
'10 x 16
SCREEN 13

CONST Land = 1
CONST Water = 2
CONST Slow = 3
CONST Damaging = 4 'Damaging is slow as well
CONST Impassible = 5
CONST Town = 6
CONST Cave = 7
CONST Dock = 8
CONST AnyPass = 9
IconAttribute = 1

KLOCK = False
'NAME$ = ""
CALL MAKEGRID
CALL SelectCol
OXCorr = 1
OYCorr = 1
CHANGE.CORR = 0
CALL PrintAttrib
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
IF UCASE$(A$) = "C" THEN
    CALL SelectCol
 
  ELSEIF A$ = CHR$(0) + "M" AND XCorr <> 20 THEN
    XCorr = XCorr + 1
 
  ELSEIF A$ = CHR$(0) + "K" AND XCorr <> 1 THEN
    XCorr = XCorr - 1
 
  ELSEIF A$ = CHR$(0) + "H" AND YCorr <> 1 THEN
    YCorr = YCorr - 1
 
  ELSEIF A$ = CHR$(0) + "P" AND YCorr <> 20 THEN
    YCorr = YCorr + 1
 
  ELSEIF UCASE$(A$) = "R" THEN
    CALL Rotate(True)
       
  ELSEIF UCASE$(A$) = "F" THEN
    CALL FlipH(True)
 
  ELSEIF UCASE$(A$) = "V" THEN
    CALL FlipV(True)
 
  ELSEIF UCASE$(A$) = "U" THEN
    CALL UpdateIcon
    A$ = "Q"
 
  ELSEIF UCASE$(A$) = "H" THEN
    CALL HelpIconMaker
 
  ELSEIF UCASE$(A$) = "S" THEN
    CALL SaveIcon
 
  ELSEIF UCASE$(A$) = "O" THEN
    CALL LoadIcon
 
  ELSEIF UCASE$(A$) = "Z" THEN
    CALL SaveIcon2
 
  ELSEIF UCASE$(A$) = "X" THEN
    CALL LoadIcon2
 
  ELSEIF UCASE$(A$) = "G" THEN
    Col = ICON(XCorr, YCorr)
    CALL SelectCol
 
  ELSEIF UCASE$(A$) = "A" THEN
    CALL Attributes
 
  ELSEIF UCASE$(A$) = "E" THEN
    CALL ClearIcon(1)
 
  ELSEIF UCASE$(A$) = "L" THEN
    IF KLOCK = 1 THEN KLOCK = 0 ELSE KLOCK = 1
    LINE (205, 165)-(280, 175), 0, BF
    IF KLOCK = 1 THEN
      CALL Textfont("Lock ON", 210, 170, 7, 0)
     ELSE CALL Textfont("Lock OFF", 210, 170, 7, 0)
    END IF
    A$ = " "
 
  ELSEIF UCASE$(A$) = "`" THEN
    ICON(XCorr, YCorr) = -1
    CALL MAKEGRID
    CALL MAKEPICBOX
END IF

IF (OXCorr <> XCorr) OR (OYCorr <> YCorr) THEN CHANGE.CORR = 1 ELSE CHANGE.CORR = 0
LOCATE 20, 26: PRINT "("; XCorr; ","; YCorr; ")    "

IF (A$ = " ") THEN
    CALL DRAWIN(XCorr, YCorr)
    CALL MAKEPICBOX
END IF

IF (CHANGE.CORR = 1) AND (KLOCK = 1) THEN
   CALL DRAWIN(XCorr, YCorr)
   CALL MAKEPICBOX
END IF

LOOP UNTIL UCASE$(A$) = "B"
MakeIconPlaceScreen
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
CALL Textfont("(Home) Moves to Top", 40, 110, 7, 0)
CALL Textfont("(End) Moves to Bottom", 40, 120, 7, 0)
CALL Textfont("(E)(I) Edit Icon by Arrow", 40, 130, 7, 0)
CALL Textfont("(H) Icon Selection Help", 40, 140, 7, 0)
CALL Textfont("(Q) Exits Program", 40, 150, 7, 0)

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

   CASE IS = "I", "i", "E", "e"
     IconMaker ("I")

   CASE IS = CHR$(0) + "G" 'HOME
     ArrowLoc = 1
     IconLoc = 1
     RefreshIcons
  
   CASE IS = CHR$(0) + "O" 'END
     ArrowLoc = 5
     IconLoc = TotalIcons
     RefreshIcons

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
  Name$ = Name$ + ".SC"

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
        BlockUsed(X + 1, Y + 1) = True
        FOR P = 1 TO 203
           GET #1, (X * 1836) + (Y * 204) + P, Pic(P, X + 1, Y + 1)
        NEXT P
        IF Pic(1, X + 1, Y + 1) = 0 THEN BlockUsed(X + 1, Y + 1) = False
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

SUB LoadIcon
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
  Name$ = Name$ + ".PIC"

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
  PATHNAME$ = PicPATH$ + Name$
  IF UCASE$(YesNo$) = "Y" THEN
    OPEN PATHNAME$ FOR INPUT AS #1
    FOR X = 1 TO 20
      FOR Y = 1 TO 20
        INPUT #1,ICON(X, Y)
      NEXT Y
    NEXT X
    INPUT #1,IconAttribute
  CLOSE #1
  END IF


WHILE INKEY$ = "": WEND
END IF
CALL PrintAttrib
CALL MAKEGRID
CALL MAKEPICBOX
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)
END SUB

SUB LoadIcon2
Name$ = ""
LINE (1, 183)-(200, 200), 0, BF
LINE (15, 1)-(195, 181), 7, B
LINE (16, 2)-(194, 180), 0, BF
CALL Textfont("* LOAD MMI *", 83, 20, 7, 0)
CALL Textfont("Enter the file name:", 25, 50, 7, 0)
X = 125
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
IF (Name$ <> "") THEN
  IF RIGHT$(Name$, 3) <> "MMI" THEN Name$ = Name$ + ".MMI"

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
  PATHNAME$ = IconPath$ + Name$
  IF UCASE$(YesNo$) = "Y" THEN
    DIM MMI(203) AS INTEGER
    OPEN PATHNAME$ FOR RANDOM AS #1 LEN = LEN(IconAttribute)
    FOR X = 1 TO 203
       GET #1, X, MMI(X)
    NEXT X
    GET #1, 204, IconAttribute
    CLOSE #1
    PUT (291, 171), MMI(1), PSET
    FOR X = 1 TO 20
       FOR Y = 1 TO 20
          A = POINT((X + 290), (Y + 170))
          IF A = 0 THEN
            ICON(X, Y) = -1
          ELSEIF A = 250 THEN
            ICON(X, Y) = 0
          ELSE
            ICON(X, Y) = A
          END IF
       NEXT Y
    NEXT X
  ERASE MMI
  END IF


WHILE INKEY$ = "": WEND
END IF
CALL PrintAttrib
CALL MAKEGRID
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)
END SUB

SUB LoadScr
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
    OPEN PATHNAME$ FOR INPUT AS #1
    FOR X = 1 TO 200
      FOR Y = 1 TO 180
        INPUT #1,Temp
        PSET (X, Y), Temp
      NEXT Y
    NEXT X
    FOR X = 1 TO 10
      FOR Y = 1 TO 9
        INPUT #1,BlockAttributes(X, Y)
      NEXT Y
    NEXT X
  CLOSE #1
  FOR X = 1 TO 10
    FOR Y = 1 TO 9
      BlockUsed(X, Y) = True
      GET ((X - 1) * 20 + 1, (Y - 1) * 20 + 1)-STEP(19, 19), Pic(1, X, Y)
    NEXT Y
  NEXT X
  CLS
  CALL Textfont("File Loaded!", 3, 3, 7, 0)
  END IF
LOCATE 1, 1:
Name$ = MID$(Name$, 1, LEN(Name$) - 4)
Name$ = UCASE$(Name$)
IF (MID$(Name$, 3, 1)) = "X" THEN
   NFlag$ = MID$(Name$, 1, 2)
   ELSE
   NFlag$ = MID$(Name$, 1, 1)
END IF

IF (MID$(Name$, 3, 1)) = "X" AND (MID$(Name$, 5, 1)) = "Y" THEN XCrr$ = MID$(Name$, 4, 1)
IF (MID$(Name$, 3, 1)) = "X" AND (MID$(Name$, 6, 1)) = "Y" THEN XCrr$ = MID$(Name$, 4, 2)
IF (MID$(Name$, 2, 1)) = "X" AND (MID$(Name$, 4, 1)) = "Y" THEN XCrr$ = MID$(Name$, 3, 1)
IF (MID$(Name$, 2, 1)) = "X" AND (MID$(Name$, 5, 1)) = "Y" THEN XCrr$ = MID$(Name$, 3, 2)

IF LEN(Name$) = 5 THEN YCrr$ = MID$(Name$, 5)
IF LEN(Name$) = 6 AND LEN(NFlag$) = 1 AND LEN(XCrr$) = 1 THEN YCrr$ = MID$(Name$, 5, 2)
IF LEN(Name$) = 6 AND ((LEN(NFlag$) = 2 AND LEN(XCrr$) = 1) OR (LEN(NFlag$) = 1 AND LEN(XCrr$) = 2)) THEN YCrr$ = MID$(Name$, 6)
IF LEN(Name$) = 7 AND ((LEN(NFlag$) = 1) OR (LEN(XCrr$) = 1)) THEN YCrr$ = MID$(Name$, 6, 2)
IF LEN(Name$) = 7 AND LEN(NFlag$) = 2 AND LEN(XCrr$) = 2 THEN YCrr$ = MID$(Name$, 7)
IF LEN(Name$) = 8 THEN YCrr$ = MID$(Name$, 7, 2)
WHILE INKEY$ = "": WEND
END IF
CALL MakeIconPlaceScreen

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

SUB MakeIconPlaceScreen
CLS
IF Grid = True THEN
FOR X = 1 TO 211 STEP 21
  LINE (X, 1)-(X, 190), 7
NEXT X
FOR Y = 1 TO 190 STEP 21
  LINE (1, Y)-(211, Y), 7
NEXT Y
END IF


'  This can be used to place an icon in a location when the coordinates are
'  (XX,YY) on Screen Grid.
IF Initial = False THEN
    FOR XX = 0 TO 9
      FOR YY = 0 TO 8
        IF BlockUsed(XX + 1, YY + 1) = True THEN
           PUT (XX * (20 + Grid) + 1 + Grid, (YY * (20 + Grid) + 1 + Grid)), Pic(1, XX + 1, YY + 1), OR
         ELSE
           LINE ((XX * (20 + Grid) + (1 + Grid)), (YY * (20 + Grid) + 1 + Grid))-STEP(19, 19), 0, BF
        END IF
      NEXT YY
    NEXT XX
END IF
IF Grid = False THEN PUT (GrlocX * 20, GrLocY * 20), WhiteBox, OR
Initial = False
LINE (235, 20)-(256, 41), 7, B

CALL IconSelection
END SUB

SUB MAKEPICBOX

SHIFTX = 290
SHIFTY = 170
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

SUB MoveIconBoxLocator
IF Grid = False THEN
PUT (OGrLocX * 20, OGrLocY * 20), WhiteBox, OR
PUT (GrlocX * 20, GrLocY * 20), WhiteBox, OR
ELSE
LINE (OGrLocX * 21 + 1, OGrLocY * 21 + 1)-STEP(21, 21), 7, B
LINE (GrlocX * 21 + 1, GrLocY * 21 + 1)-STEP(21, 21), 15, B
END IF
END SUB

SUB NewIcon
LINE (1, 1)-(211, 190), 7, B
LINE (2, 2)-(210, 189), 0, BF
IF BlockUsed(GrlocX + 1, GrLocY + 1) = True THEN
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
      PUT #1, X, Pic(X, GrlocX, GrLocY)
    NEXT X
    PUT #1, 204, BlockAttributes(GrlocX + 1, GrLocY + 1)
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

SUB NewNFlag (NFlag$)
DO
LOCATE 17, 28: INPUT "NF: ", NFlag$
LOCATE 17, 28: PRINT "       "
LOOP UNTIL (LEN(NFlag$) > 0) AND (LEN(NFlag$) <= 2)
NFlag$ = UCASE$(NFlag$)

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

SUB PrintAttrib
SELECT CASE IconAttribute
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
LINE (200, 45)-(300, 55), 0, BF
CALL Textfont(Attrib$, 200, 50, 7, 0)
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


SUB Rotate (IM AS INTEGER)
DIM NICON(20, 20)AS INTEGER
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
  Name$ = Name$ + ".SC"

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
    Z = 0
    FOR X = 0 TO 9
      FOR Y = 0 TO 8
        FOR P = 1 TO 203
           IF BlockUsed(X + 1, Y + 1) = True THEN
             PUT #1, (X * 1836) + (Y * 204) + P, Pic(P, X + 1, Y + 1)
            ELSE
             PUT #1, (X * 1836) + (Y * 204) + P, Z
           END IF
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

SUB SaveIcon
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
  Name$ = Name$ + ".PIC"

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
PATHNAME$ = PicPATH$ + Name$
  IF UCASE$(YesNo$) = "Y" THEN
    OPEN PATHNAME$ FOR OUTPUT AS #1
    FOR X = 1 TO 20
      FOR Y = 1 TO 20
        WRITE #1, ICON(X, Y)
      NEXT Y
    NEXT X
    WRITE #1, IconAttribute
    CLOSE #1
  END IF
NewName = 1
OPEN ("E:\QB\PICS\" + "PICFILES.TXT") FOR INPUT AS #1'(PicPATH$ + "PICFILES.TXT") FOR INPUT AS #1
    WHILE NOT EOF(1)
       INPUT #1,Test$
       IF UCASE$(Test$) = UCASE$(Name$) THEN NewName = 0
    WEND
CLOSE #1

IF NewName = 1 THEN
  OPEN ("E:\QB\PICS\" + "PICFILES.TXT") FOR APPEND AS #1'(PicPATH$ + "PICFILES.TXT") FOR APPEND AS #1
    WRITE #1, Name$
  CLOSE #1
END IF
CALL Textfont("File Saved!", 25, 90, 7, 0)
WHILE INKEY$ = "": WEND
END IF
CALL MAKEGRID
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)
END SUB

SUB SaveIcon2
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
PATHNAME$ = IconPath$ + Name$
  IF UCASE$(YesNo$) = "Y" THEN
    DIM MMI(203) AS INTEGER
    GET (291, 171)-STEP(20 - 1, 20 - 1), MMI
    OPEN PATHNAME$ FOR RANDOM AS #1 LEN = LEN(ActiveIconsAttrib(1))
    FOR X = 1 TO 203
      PUT #1, X, MMI(X)
    NEXT X
    PUT #1, 204, IconAttribute
    CLOSE #1
    ERASE MMI
  END IF

NewName = 1
OPEN (IconPath$ + "MMIFILES.TXT") FOR INPUT AS #1
    WHILE NOT EOF(1)
       INPUT #1,Test$
       IF UCASE$(Test$) = UCASE$(Name$) THEN NewName = 0
    WEND
CLOSE #1

IF NewName = 1 THEN
  OPEN (IconPath$ + "MMIFILES.TXT") FOR APPEND AS #1
    WRITE #1, Name$
  CLOSE #1
END IF

CALL Textfont("File Saved!", 25, 90, 7, 0)
WHILE INKEY$ = "": WEND
END IF
CALL MAKEGRID
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)

END SUB

SUB SaveScr
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
    OPEN PATHNAME$ FOR OUTPUT AS #1
    FOR X = 1 TO 10
      FOR Y = 1 TO 9
        IF BlockUsed(X, Y) THEN
          PUT ((X - 1) * 20 + 1, (Y - 1) * 20 + 1), Pic(1, X, Y), PSET
         ELSE
          LINE ((X - 1) * 20 + 1, (Y - 1) * 20 + 1)-STEP(19, 19), 0, BF
        END IF
      NEXT Y
    NEXT X
    FOR X = 1 TO 200
      FOR Y = 1 TO 180
        PSET (X - 1, Y), 0
        Temp = POINT(X, Y)
        PRINT #1, Temp
      NEXT Y
    NEXT X
    FOR X = 1 TO 10
      FOR Y = 1 TO 9
        PRINT #1, BlockAttributes(X, Y)
      NEXT Y
    NEXT X
  CLOSE #1
  CLS
  CALL Textfont("File Saved!", 3, 190, 7, 0)
  END IF
LOCATE 1, 1:
Name$ = MID$(Name$, 1, LEN(Name$) - 4)
Name$ = UCASE$(Name$)
WHILE INKEY$ = "": WEND
END IF
CALL MakeIconPlaceScreen

END SUB

SUB SelectCol
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




LOOP UNTIL UCASE$(B$) = "Q" OR UCASE$(B$) = " " OR UCASE$(B$) = "H"
IF UCASE$(B$) = "Q" THEN
   CALL Quit
 ELSEIF UCASE$(B$) = "H" THEN
   CALL HelpIconMaker
END IF

PSET (285, 72): DRAW "c0;r5h2f2g2"
LINE (1, 183)-(200, 200), 0, BF
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)
END SUB

SUB SetCoordinates (XCrr$, YCrr$)
DO
BEEP
LOCATE 18, 28: INPUT "X: ", XCrr$
LOCATE 19, 28: INPUT "Y: ", YCrr$
LOCATE 18, 28: PRINT "       "
LOCATE 19, 28: PRINT "       "
LOOP UNTIL (LEN(XCrr$) > 0) AND (LEN(XCrr$) <= 2) AND (LEN(YCrr$) > 0) AND (LEN(YCrr$) <= 2)
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

SUB UpdateIcon
GET (291, 171)-STEP(19, 19), Pic(1, GrlocX + 1, GrLocY + 1)
END SUB