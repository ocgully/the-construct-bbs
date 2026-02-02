DECLARE SUB LoadIconFromList ()
DECLARE SUB IconEdit ()
DECLARE SUB Attributes ()
DECLARE SUB ClearIconPic (Initial!)
DECLARE SUB LoadIcon ()
DECLARE SUB SaveIcon ()
DECLARE SUB UpdateIconBox (XCorr!, YCorr!)
DECLARE SUB UpdateIconPic (XCorr!, YCorr!)
DECLARE SUB IconHelp ()
DECLARE SUB SelectCol ()
DECLARE SUB PrintAttrib ()
DECLARE SUB MakePicBox ()
DECLARE SUB MakeGrid ()
DECLARE SUB Inverse ()
DECLARE SUB VertFlip ()
DECLARE SUB NewNFlag (NFlag$)
DECLARE SUB CoordinateLoad (XCorr$, YCorr$, NFlag$)
DECLARE SUB SetCoordinates (XCorr$, YCorr$)
DECLARE SUB CoordinateSave (XCorr$, YCorr$, NFlag$)
DECLARE SUB BlockHelp ()
DECLARE SUB IconSelectHelp ()
DECLARE SUB NewIcon ()
DECLARE SUB SaveMapLocation ()
DECLARE SUB BlockAttribScreen ()
DECLARE SUB Quit ()
DECLARE SUB ClearPic ()
DECLARE SUB Help ()
DECLARE SUB Load (XCorr$, YCorr$, NFlag$)
DECLARE SUB Save ()
DECLARE SUB Textfont (S$, X!, Y!, C!, S!)
DECLARE SUB IconSelection ()
DECLARE SUB RefreshIcons ()
DECLARE SUB OpeningScreen ()
DECLARE SUB MoveIconBoxLocator ()
DECLARE SUB MakeIconPlaceScreen ()
CONST Ver$ = "Ver 3.3"
CONST False = 0
CONST True = 1
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
CONST XDIM = 20
CONST YDIM = 20

COMMON SHARED OXCorr!, XCorr!, OYCorr!, YCorr!, CHANGE.CORR, Attrib$, IconAttribute, IconDone
COMMON SHARED IconLoc, TotalIcons, ArrowLoc, Initial, Done, GrLocX, GrLocY, OGrLocX, OGrLocY, SelectedIconAttrib

XCorr = 1
YCorr = 1
OYCorr = 1
OXCorr = 1
IconAttribute = 1
Done = False
Initial = True
IconLoc = 1
ArrowLoc = 1
XCorr$ = "0"
YCorr$ = "0"
NFlag$ = "M"
SCREEN 13
CLS
CALL OpeningScreen
REM $DYNAMIC
OPTION BASE 1
DIM SHARED Pic(200, 180) AS INTEGER
DIM SHARED IconNames(150) AS STRING 'MAKE LOF(PicFiles.txt)
DIM SHARED ActiveIcons(5, 20, 20) AS INTEGER
DIM SHARED SelectedIcon(20, 20) AS INTEGER
DIM SHARED BlockAttributes(10, 9) AS INTEGER
DIM SHARED ActiveIconsAttrib(5) AS INTEGER
DIM SHARED TempIcon(20, 20)AS INTEGER
DIM SHARED Icon(XDIM, YDIM)
DIM SHARED Col AS INTEGER

SelectedIconAttrib = 1

COLOR 7: LOCATE 1, 1: PRINT "Loading 0 %"
FOR X = 1 TO 10
  FOR Y = 1 TO 9
     BlockAttributes(X, Y) = 1
  NEXT Y
NEXT X

FOR T = 1 TO 200
  FOR B = 1 TO 180
    Pic(T, B) = 0     'THIS INITIALIZES THE PICTURE FILE AND MAKES EVERYTHING
  NEXT B              'THE COLOR BLACK
  LOCATE 1, 9: PRINT INT(T / 2); "%"
NEXT T
T = 0
PATHNAME$ = IconPath$ + "PICFILES.TXT"
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
           FOR X = 1 TO 20
             FOR Y = 1 TO 20
               IF SelectedIcon(X, Y) <> -1 THEN
                   PSET (GrLocX * 21 + 1 + X, GrLocY * 21 + 1 + Y), SelectedIcon(X, Y)
                   Pic(GrLocX * 20 + X, GrLocY * 20 + Y) = SelectedIcon(X, Y)
               END IF
             NEXT Y
           NEXT X
           BlockAttributes(GrLocX + 1, GrLocY + 1) = SelectedIconAttrib
    CASE IS = "S", "s"
      CALL Save

    CASE IS = "L", "l", "O", "o"
      CALL Load(XCorr$, YCorr$, NFlag$)

    CASE IS = "Z", "z"
      CALL CoordinateSave(XCorr$, YCorr$, NFlag$)

    CASE IS = "M", "m"
      CALL CoordinateLoad(XCorr$, YCorr$, NFlag$)
   
    CASE IS = "H", "h"
      CALL Help

    CASE IS = "c", "C"
      CALL SetCoordinates(XCorr$, YCorr$)

    CASE IS = "e", "E"
      CALL ClearPic

    CASE IS = "q", "Q"
      CALL Quit

    CASE IS = "B", "b"
      CALL BlockAttribScreen
      CALL IconSelection

    CASE IS = "G", "g"
      FOR X = 1 TO 20
         FOR Y = 1 TO 20
           SelectedIcon(X, Y) = Pic(GrLocX * 20 + X, GrLocY * 20 + Y)
           PSET (235 + X, 20 + Y), SelectedIcon(X, Y)
         NEXT Y
      NEXT X
    SelectedIconAttrib = BlockAttributes(GrLocX + 1, GrLocY + 1)

    CASE IS = "n", "N"
      CALL NewNFlag(NFlag$)

    CASE IS = "r", "R"
      CALL Inverse

    CASE IS = "v", "V"
      CALL VertFlip

    CASE IS = "d", "d"
      CALL IconEdit

END SELECT
IF ChangedGr = True THEN CALL MoveIconBoxLocator

LOCATE 1, 28: PRINT "("; GrLocX + 1; ","; GrLocY + 1; ")  "
LOCATE 23, 28: PRINT NFlag$; "("; XCorr$; ","; YCorr$; ")   "

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
CALL MakeGrid  'Draws Grid to Screen
CALL MakePicBox 'Makes the picture box in the bottom right
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

SUB ClearIconPic (Initial)
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
FOR X = 1 TO XDIM        ' This sets all the dimensions to clear
  FOR Y = 1 TO YDIM      '
    Icon(X, Y) = -1      '
  NEXT Y                 '
NEXT X                   '

Col = 0
LINE (290, 170)-(311, 191), 7, B
CALL MakeGrid  'Draws Grid to Screen
CALL MakePicBox 'Makes the picture box in the bottom right
CALL SelectCol

OXCorr = 1
OYCorr = 1
XCorr = 1
YCorr = 1
CHANGE.CORR = 0
END IF
CALL PrintAttrib
CALL MakeGrid
CALL MakePicBox
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
      FOR X = 1 TO 200
        FOR Y = 1 TO 180
          Pic(X, Y) = 0
        NEXT Y
      NEXT X
      FOR X = 1 TO 10
        FOR Y = 1 TO 9
           BlockAttributes(X, Y) = 1
        NEXT Y
      NEXT X
   END IF


CALL MakeIconPlaceScreen

END SUB

SUB CoordinateLoad (XCorr$, YCorr$, NFlag$)
DO
LOCATE 17, 28: PRINT "NF: "; NFlag$
LOCATE 18, 28: INPUT "X: ", XCorr$
LOCATE 19, 28: INPUT "Y: ", YCorr$
LOCATE 18, 28: PRINT "       "
LOCATE 19, 28: PRINT "       "
LOOP UNTIL (LEN(XCorr$) > 0) AND (LEN(XCorr$) <= 2) AND (LEN(YCorr$) > 0) AND (LEN(YCorr$) <= 2) AND (LEN(NFlag$) > 0) AND (LEN(NFlag$) <= 2)
LINE (1, 1)-(211, 190), 7, B
LINE (2, 2)-(210, 189), 0, BF
CALL Textfont("* Coordinate Load *", 53, 20, 7, 0)
Name$ = NFlag$ + "X" + XCorr$ + "Y" + YCorr$ + ".SCR"

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
        INPUT #1,Pic(X, Y)
      NEXT Y
    NEXT X
    FOR X = 1 TO 10
      FOR Y = 1 TO 9
        INPUT #1,BlockAttributes(X, Y)
      NEXT Y
    NEXT X
  CLOSE #1
  CALL Textfont("File Loaded!", 25, 90, 7, 0)
  END IF


WHILE INKEY$ = "": WEND
CALL MakeIconPlaceScreen

END SUB

SUB CoordinateSave (XCorr$, YCorr$, NFlag$)
LINE (1, 1)-(211, 190), 7, B
LINE (2, 2)-(210, 189), 0, BF
CALL Textfont("* Coordinate Save *", 43, 20, 7, 0)
Name$ = NFlag$ + "X" + XCorr$ + "Y" + YCorr$ + ".SCR"

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
    OPEN PATHNAME$ FOR OUTPUT AS #1
    FOR X = 1 TO 200
      FOR Y = 1 TO 180
        WRITE #1, Pic(X, Y)
      NEXT Y
    NEXT X
    FOR X = 1 TO 10
      FOR Y = 1 TO 9
        WRITE #1, BlockAttributes(X, Y)
      NEXT Y
    NEXT X
    CLOSE #1
    CALL Textfont("File Saved!", 25, 90, 7, 0)
  END IF
WHILE INKEY$ = "": WEND
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

SUB IconEdit
CLS
KLOCK = False
IconDone = False
'NAME$ = ""
CALL Textfont("Lock OFF", 210, 170, 7, 0)
FOR X = 1 TO XDIM
  FOR Y = 1 TO YDIM
    Icon(X, Y) = SelectedIcon(X, Y)
  NEXT Y
NEXT X

CALL MakeGrid
CALL MakePicBox
CALL PrintAttrib
CALL SelectCol
DO 'User Input


IF CHANGE.CORR = 1 THEN LINE ((OXCorr * 9) + 6, ((OYCorr - 1) * 9) + 1)-((OXCorr * 9) + 6 + 9, ((OYCorr - 1) * 9) + 1 + 9), 7, B
LINE ((XCorr * 9) + 6, ((YCorr - 1) * 9) + 1)-((XCorr * 9) + 6 + 9, ((YCorr - 1) * 9) + 1 + 9), 15, B
A$ = INKEY$
A$ = UCASE$(A$)
OXCorr = XCorr
OYCorr = YCorr

SELECT CASE A$
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
                 
          CASE IS = "H"
               CALL IconHelp
         
          CASE IS = "F"
               CALL LoadIconFromList
         
          CASE IS = "S"
               CALL SaveIcon
                        
          CASE IS = "O"
               CALL LoadIcon
                             
          CASE IS = "G"
               Col = Icon(XCorr, YCorr)
               CALL SelectCol
              
          CASE IS = "A"
               CALL Attributes
                                 
          CASE IS = "E"
               CALL ClearIconPic(1)
                                     
          CASE IS = "L"
               IF KLOCK = 1 THEN KLOCK = 0 ELSE KLOCK = 1
               LINE (205, 165)-(280, 175), 0, BF
               IF KLOCK = 1 THEN
                  CALL Textfont("Lock ON", 210, 170, 7, 0)
                  ELSE CALL Textfont("Lock OFF", 210, 170, 7, 0)
               END IF
               A$ = " "
                                 
          CASE IS = "`"
               Icon(XCorr, YCorr) = -1
               CALL UpdateIconPic(XCorr, YCorr)
               CALL UpdateIconBox(XCorr, YCorr)

END SELECT

IF (OXCorr <> XCorr) OR (OYCorr <> YCorr) THEN CHANGE.CORR = 1 ELSE CHANGE.CORR = 0
LOCATE 20, 26: PRINT "("; XCorr; ","; YCorr; ")    "

IF (A$ = " ") THEN
    Icon(XCorr, YCorr) = Col
    CALL UpdateIconPic(XCorr, YCorr)
    CALL UpdateIconBox(XCorr, YCorr)
END IF

IF (CHANGE.CORR = 1) AND (KLOCK = 1) THEN
   Icon(XCorr, YCorr) = Col
   CALL UpdateIconPic(XCorr, YCorr)
   CALL UpdateIconBox(XCorr, YCorr)
END IF

LOOP UNTIL UCASE$(A$) = "Q" OR IconDone = True  'End User Input



PATHNAME$ = IconPath$ + "PICFILES.TXT"
OPEN PATHNAME$ FOR INPUT AS #1
    WHILE NOT EOF(1)
       T = T + 1
       INPUT #1,Temp$
       IconNames(T) = Temp$
    WEND
TotalIcons = T
CLOSE #1

CALL MakeIconPlaceScreen

END SUB

SUB IconHelp
LINE (1, 183)-(200, 200), 0, BF
LINE (15, 1)-(195, 181), 7, B
LINE (16, 2)-(194, 180), 0, BF
CALL Textfont("* Help *", 83, 20, 7, 0)
CALL Textfont("(E) Clears Icon", 50, 50, 7, 0)
CALL Textfont("(Space) Places Color", 50, 60, 7, 0)
CALL Textfont("(G) Get Color corsor is on", 50, 70, 7, 0)
CALL Textfont("(C) Select Color", 50, 80, 7, 0)
CALL Textfont("(S) Saves Icon", 50, 90, 7, 0)
CALL Textfont("(O) Opens Previous Icon", 50, 100, 7, 0)
CALL Textfont("(A) Change Attribute", 50, 110, 7, 0)
CALL Textfont("(L) lock's the color so you", 50, 120, 7, 0)
CALL Textfont("         can drag the color", 50, 127, 7, 0)

DO
LOOP UNTIL INKEY$ <> ""
CALL MakeGrid
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)


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
DO
B$ = INKEY$
SELECT CASE B$
   CASE IS = "H", "h"
     CALL IconSelectHelp
     
   CASE IS = "q", "Q"
      CALL Quit

   CASE IS = " "

         FOR X = 1 TO 20
           FOR Y = 1 TO 20
             SelectedIcon(X, Y) = ActiveIcons(ArrowLoc, X, Y)
           NEXT Y
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

   CASE IS = CHR$(0) + "G" 'HOME
     IconLoc = 1
     ArrowLoc = 1
     CALL RefreshIcons
  
   CASE IS = CHR$(0) + "O" 'END
     IconLoc = TotalIcons
     ArrowLoc = 5
     CALL RefreshIcons

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
FOR X = 1 TO 20
  FOR Y = 1 TO 20
    PSET (235 + X, 20 + Y), SelectedIcon(X, Y)
  NEXT Y
NEXT X
         
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

SUB Inverse
FOR X = 20 TO 1 STEP -1
  FOR Y = 1 TO 20
    TempIcon(21 - X, Y) = SelectedIcon(X, Y)
  NEXT Y
NEXT X

FOR X = 1 TO 20
  FOR Y = 1 TO 20
    SelectedIcon(X, Y) = TempIcon(X, Y)
  NEXT Y
NEXT X
FOR X = 1 TO 20
  FOR Y = 1 TO 20
    PSET (235 + X, 20 + Y), SelectedIcon(X, Y)
  NEXT Y
NEXT X

END SUB



SUB Load (XCorr$, YCorr$, NFlag$)
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
        INPUT #1,Pic(X, Y)
      NEXT Y
    NEXT X
    FOR X = 1 TO 10
      FOR Y = 1 TO 9
        INPUT #1,BlockAttributes(X, Y)
      NEXT Y
    NEXT X
  CLOSE #1
  END IF

CALL Textfont("File Loaded!", 25, 90, 7, 0)
LOCATE 1, 1:
Name$ = MID$(Name$, 1, LEN(Name$) - 4)
Name$ = UCASE$(Name$)
IF (MID$(Name$, 3, 1)) = "X" THEN
   NFlag$ = MID$(Name$, 1, 2)
   ELSE
   NFlag$ = MID$(Name$, 1, 1)
END IF

IF (MID$(Name$, 3, 1)) = "X" AND (MID$(Name$, 5, 1)) = "Y" THEN XCorr$ = MID$(Name$, 4, 1)
IF (MID$(Name$, 3, 1)) = "X" AND (MID$(Name$, 6, 1)) = "Y" THEN XCorr$ = MID$(Name$, 4, 2)
IF (MID$(Name$, 2, 1)) = "X" AND (MID$(Name$, 4, 1)) = "Y" THEN XCorr$ = MID$(Name$, 3, 1)
IF (MID$(Name$, 2, 1)) = "X" AND (MID$(Name$, 5, 1)) = "Y" THEN XCorr$ = MID$(Name$, 3, 2)

IF LEN(Name$) = 5 THEN YCorr$ = MID$(Name$, 5)
IF LEN(Name$) = 6 AND LEN(NFlag$) = 1 AND LEN(XCorr$) = 1 THEN YCorr$ = MID$(Name$, 5, 2)
IF LEN(Name$) = 6 AND ((LEN(NFlag$) = 2 AND LEN(XCorr$) = 1) OR (LEN(NFlag$) = 1 AND LEN(XCorr$) = 2)) THEN YCorr$ = MID$(Name$, 6)
IF LEN(Name$) = 7 AND ((LEN(NFlag$) = 1) OR (LEN(XCorr$) = 1)) THEN YCorr$ = MID$(Name$, 6, 2)
IF LEN(Name$) = 7 AND LEN(NFlag$) = 2 AND LEN(XCorr$) = 2 THEN YCorr$ = MID$(Name$, 7)
IF LEN(Name$) = 8 THEN YCorr$ = MID$(Name$, 7, 2)
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
  PATHNAME$ = IconPath$ + Name$
  IF UCASE$(YesNo$) = "Y" THEN
    OPEN PATHNAME$ FOR INPUT AS #1
    FOR X = 1 TO XDIM
      FOR Y = 1 TO YDIM
        INPUT #1,Icon(X, Y)
      NEXT Y
    NEXT X
    INPUT #1,IconAttribute
  CLOSE #1
  END IF


WHILE INKEY$ = "": WEND
END IF
CALL PrintAttrib
CALL MakeGrid
CALL MakePicBox
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)


END SUB

SUB LoadIconFromList
CLS
PATHNAME$ = IconPath$ + "PICFILES.TXT"
OPEN PATHNAME$ FOR INPUT AS #1
    WHILE NOT EOF(1)
       T = T + 1
       INPUT #1,Temp$
       IconNames(T) = UCASE$(Temp$)
    WEND
TotalIcons = T
CLOSE #1
INum = 1

Num$ = STR$(INum)
TI$ = STR$(TotalIcons)
CALL Textfont("Select File to Load", 5, 10, 5, 0)
CALL Textfont("Icon #: " + Num$, 5, 20, 7, 0)
CALL Textfont("Total Icons: " + TI$, 5, 30, 7, 0)
CALL Textfont("Current File: " + IconNames(INum), 5, 50, 3, 0)
CALL Textfont("Press Up to Move up one File, and Down to Move Down", 5, 180, 7, 0)
CALL Textfont("Press ESC to Abort Load", 5, 190, 7, 0)
   OPEN IconPath$ + IconNames(INum) FOR INPUT AS #1
     FOR X = 1 TO 20
       FOR Y = 1 TO 20
         INPUT #1,TempIcon(X, Y)
         PSET (X + 100, Y + 100), TempIcon(X, Y)
       NEXT Y
     NEXT X
     INPUT #1,TempAttribNum
   CLOSE #1

LOCATE
DO
A$ = INKEY$
Changed = False
SELECT CASE A$
     CASE IS = CHR$(0) + "H"    'UP
          IF INum - 1 > 0 THEN
             INum = INum - 1
             Changed = True
          ELSE
             BEEP
          END IF

     CASE IS = CHR$(0) + "P"    'DOWN
          IF INum + 1 <= TotalIcons THEN
             INum = INum + 1
             Changed = True
          ELSE
             BEEP
          END IF
  
   CASE IS = CHR$(0) + "I" 'PAGE UP
     IF INum - 5 >= 1 THEN
       INum = INum - 5
       Changed = True
     ELSE
       IF INum <> 1 THEN
         INum = 1
         Changed = True
       END IF
     END IF


   CASE IS = CHR$(0) + "Q" 'PAGE DOWN
     IF INum + 5 <= TotalIcons THEN
       INum = INum + 5
       Changed = True
     ELSE
       IF INum <> 1 THEN
         INum = TotalIcons
         Changed = True
       END IF
     END IF


     CASE IS = " "

END SELECT


IF Changed = True THEN
   OPEN IconPath$ + IconNames(INum) FOR INPUT AS #1
     FOR X = 1 TO 20
       FOR Y = 1 TO 20
         INPUT #1,TempIcon(X, Y)
         PSET (X + 100, Y + 100), TempIcon(X, Y)
       NEXT Y
     NEXT X
     INPUT #1,TempAttribNum
   CLOSE #1
LINE (0, 15)-(200, 55), 0, BF
Num$ = STR$(INum)
TI$ = STR$(TotalIcons)
CALL Textfont("Icon #: " + Num$, 5, 20, 7, 0)
CALL Textfont("Total Icons: " + TI$, 5, 30, 7, 0)
CALL Textfont("Current File: " + IconNames(INum), 5, 50, 3, 0)
END IF

LOOP UNTIL A$ = " " OR A$ = CHR$(27)

IF A$ = " " THEN
  FOR X = 1 TO 20
    FOR Y = 1 TO 20
      Icon(X, Y) = TempIcon(X, Y)
    NEXT Y
  NEXT X
  IconAttribute = TempAttribNum
END IF
CLS
CALL MakeGrid
CALL MakePicBox
CALL SelectCol
END SUB

SUB MakeGrid
SHIFTY = 1
SHIFTX = 15
FOR X = SHIFTX TO 9 * XDIM + 1 + SHIFTX STEP 9
    LINE (X, SHIFTY)-(X, XDIM * 9 + SHIFTY), 7
NEXT X
FOR Y = SHIFTY TO 9 * YDIM + 1 + SHIFTY STEP 9
    LINE (SHIFTX, Y)-(YDIM * 9 + SHIFTX, Y), 7
NEXT Y
FOR Y = 1 TO YDIM
 FOR X = 1 TO XDIM
  IF Icon(X, Y) <> -1 THEN
  LINE (SHIFTX + 1 + ((X - 1) * 9), SHIFTY + 1 + ((Y - 1) * 9))-(SHIFTX + 8 + ((X - 1) * 9), 1 + SHIFTY + 7 + ((Y - 1) * 9)), Icon(X, Y), BF
  ELSE
  LINE (SHIFTX + 1 + ((X - 1) * 9), SHIFTY + 1 + ((Y - 1) * 9))-(SHIFTX + 8 + ((X - 1) * 9), 1 + SHIFTY + 7 + ((Y - 1) * 9)), 0, BF
  PSET (SHIFTX + 1 + ((X - 1) * 9 + 4), SHIFTY + 1 + ((Y - 1) * 9) + 4), 7
  END IF
 NEXT X
NEXT Y

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
'  (XX,YY) on Screen Grid.  The (X,Y) are the Icon Coordinates as it places
'  it.
YC = XC = 0
IF Initial = False THEN
    FOR XX = 0 TO 9
      FOR YY = 0 TO 8
        FOR X = 1 TO 20
          FOR Y = 1 TO 20
        PSET (XX * 21 + 1 + X, YY * 21 + 1 + Y), Pic((XX * 20) + X, (YY * 20) + Y)
          NEXT Y
        NEXT X
      NEXT YY
    NEXT XX
END IF
Initial = False
LINE (235, 20)-(256, 41), 7, B

CALL IconSelection
END SUB

SUB MakePicBox
SHIFTX = 290
SHIFTY = 170
FOR Y = 1 TO YDIM
  FOR X = 1 TO XDIM
   IF Icon(X, Y) <> -1 THEN
      PSET (SHIFTX + X, SHIFTY + Y), Icon(X, Y)
        ELSE
          PSET (SHIFTX + X, SHIFTY + Y), 0
   END IF
  NEXT X
NEXT Y

END SUB

SUB MoveIconBoxLocator
LINE (OGrLocX * 21 + 1, OGrLocY * 21 + 1)-(OGrLocX * 21 + 22, OGrLocY * 21 + 22), 7, B
LINE (GrLocX * 21 + 1, GrLocY * 21 + 1)-(GrLocX * 21 + 22, GrLocY * 21 + 22), 15, B
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

FOR T = 1 TO 3
T1$ = TIME$
WHILE T1$ = TIME$: WEND   '3 second delay timer
NEXT T
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
    OPEN FileName$ FOR INPUT AS #1
    Count = Count + 1
       FOR X = 1 TO 20
         FOR Y = 1 TO 20
           INPUT #1,ActiveIcons(Count, X, Y)
           PSET (290 + X, 20 + (Count * 25) + Y), ActiveIcons(Count, X, Y)
         NEXT Y
       NEXT X
       INPUT #1,ActiveIconsAttrib(Count)
    CLOSE #1
    LINE (289, 19 + (Count * 25))-(311, 42 + (Count * 25)), 7, B
 NEXT T

  ELSE
  FOR T = IconLoc - ArrowLoc + 1 TO IconLoc + 5 - ArrowLoc
    FileName$ = IconPath$ + IconNames(T)
    OPEN FileName$ FOR INPUT AS #1
    Count = Count + 1
       FOR X = 1 TO 20
         FOR Y = 1 TO 20
           INPUT #1,ActiveIcons(Count, X, Y)
           PSET (290 + X, 20 + (Count * 25) + Y), ActiveIcons(Count, X, Y)
         NEXT Y
       NEXT X
       INPUT #1,ActiveIconsAttrib(Count)
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
    OPEN PATHNAME$ FOR OUTPUT AS #1
    FOR X = 1 TO 200
      FOR Y = 1 TO 180
        WRITE #1, Pic(X, Y)
      NEXT Y
    NEXT X
    FOR X = 1 TO 10
      FOR Y = 1 TO 9
        WRITE #1, BlockAttributes(X, Y)
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
CALL Textfont("File Saved!", 25, 90, 7, 0)
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
PATHNAME$ = IconPath$ + Name$
  IF UCASE$(YesNo$) = "Y" THEN
    OPEN PATHNAME$ FOR OUTPUT AS #1
    FOR X = 1 TO XDIM
      FOR Y = 1 TO YDIM
        WRITE #1, Icon(X, Y)
      NEXT Y
    NEXT X
    WRITE #1, IconAttribute
    CLOSE #1
  END IF
NewName = 1
OPEN "C:\BG_RPG\BASIC\PICS\PICFILES.TXT" FOR INPUT AS #1
    WHILE NOT EOF(1)
       INPUT #1,Test$
       IF UCASE$(Test$) = UCASE$(Name$) THEN NewName = 0
    WEND
CLOSE #1

IF NewName = 1 THEN
  OPEN "C:\BG_RPG\BASIC\PICS\PICFILES.TXT" FOR APPEND AS #1
    WRITE #1, Name$
  CLOSE #1
END IF
CALL Textfont("File Saved!", 25, 90, 7, 0)
WHILE INKEY$ = "": WEND
END IF
CALL MakeGrid
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)


END SUB

SUB SelectCol
PSET (285, 72): DRAW "c15;r5h2f2g2"
LINE (1, 183)-(200, 200), 0, BF
CALL Textfont("SPACE BAR to Select color", 10, 188, 15, 0)
CALL Textfont("(Q)uit to MapMaker", 10, 195, 15, 0)
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
IF B$ = CHR$(0) + "P" AND Col <> 176 THEN
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
   IconDone = True
     ELSEIF UCASE$(B$) = "H" THEN
       CALL IconHelp
END IF

PSET (285, 72): DRAW "c0;r5h2f2g2"
LINE (1, 183)-(200, 200), 0, BF
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit to mapmaker", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)


END SUB

SUB SetCoordinates (XCorr$, YCorr$)
DO
BEEP
LOCATE 18, 28: INPUT "X: ", XCorr$
LOCATE 19, 28: INPUT "Y: ", YCorr$
LOCATE 18, 28: PRINT "       "
LOCATE 19, 28: PRINT "       "
LOOP UNTIL (LEN(XCorr$) > 0) AND (LEN(XCorr$) <= 2) AND (LEN(YCorr$) > 0) AND (LEN(YCorr$) <= 2)
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
IF S = 1 AND UCASE$(MID$(S$, T, 1)) <> " " THEN FOR m = 1 TO (1000 * TD): NEXT m

NEXT T



END SUB

SUB UpdateIconBox (XCorr!, YCorr!)
SHIFTX = 290
SHIFTY = 170
   IF Icon(XCorr, YCorr) <> -1 THEN
      PSET (SHIFTX + XCorr, SHIFTY + YCorr), Icon(XCorr, YCorr)
        ELSE
          PSET (SHIFTX + XCorr, SHIFTY + YCorr), 0
   END IF


END SUB

SUB UpdateIconPic (XCorr!, YCorr!)
SHIFTY = 1
SHIFTX = 15
  IF Icon(XCorr, YCorr) <> -1 THEN
  LINE (SHIFTX + 1 + ((XCorr - 1) * 9), SHIFTY + 1 + ((YCorr - 1) * 9))-(SHIFTX + 8 + ((XCorr - 1) * 9), 1 + SHIFTY + 7 + ((YCorr - 1) * 9)), Icon(XCorr, YCorr), BF
  ELSE
  LINE (SHIFTX + 1 + ((XCorr - 1) * 9), SHIFTY + 1 + ((YCorr - 1) * 9))-(SHIFTX + 8 + ((XCorr - 1) * 9), 1 + SHIFTY + 7 + ((YCorr - 1) * 9)), 0, BF
  PSET (SHIFTX + 1 + ((XCorr - 1) * 9 + 4), SHIFTY + 1 + ((YCorr - 1) * 9) + 4), 7
  END IF

END SUB

SUB VertFlip
FOR X = 1 TO 20
  FOR Y = 20 TO 1 STEP -1
    TempIcon(X, 21 - Y) = SelectedIcon(X, Y)
  NEXT Y
NEXT X

FOR X = 1 TO 20
  FOR Y = 1 TO 20
    SelectedIcon(X, Y) = TempIcon(X, Y)
  NEXT Y
NEXT X

FOR X = 1 TO 20
  FOR Y = 1 TO 20
    PSET (235 + X, 20 + Y), SelectedIcon(X, Y)
  NEXT Y
NEXT X


END SUB