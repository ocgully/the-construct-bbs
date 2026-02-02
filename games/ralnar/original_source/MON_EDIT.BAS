DECLARE SUB ChangeCol ()
DECLARE SUB SLCurs (MODE AS INTEGER)
DECLARE SUB SetSize ()
DECLARE SUB PutCol ()
DECLARE SUB Help ()
DECLARE SUB ChangeCurs ()
DECLARE SUB WaitKey ()
DECLARE SUB MakeEditScreen ()
DECLARE SUB SelectCol ()
DECLARE SUB Textfont (S$, X!, Y!, C!, S!)
DECLARE SUB OpeningScreen ()
DECLARE SUB Move (X AS INTEGER, Y AS INTEGER)
OPTION BASE 1
TYPE Spell
  SpellName AS STRING * 10
  SpellDesc AS STRING * 30
  Effect AS INTEGER
  EStrength AS INTEGER
  MPCost AS INTEGER
' HPCost AS INTEGER 'Possibly for sacrifice-Spells,
'                    Take 100 Damage & kill Monster of choice?
END TYPE

CONST Damaging = 0
CONST Healing = 1
CONST Poison = 2
CONST Paralyze = 3
CONST Slow = 4
CONST AttackUp = 5
CONST DefenseUp = 6
CONST DrainHP = 7
CONST DrainMP = 8
CONST Speed = 9
CONST Confuse = 10

TYPE Monster
  HP AS INTEGER
  MP AS INTEGER
  HPMax AS INTEGER
  MPMax AS INTEGER
  Attack AS INTEGER
  Defense AS INTEGER
  MAttack AS INTEGER
  MDefense AS INTEGER
  Speed AS INTEGER
  Item1 AS INTEGER
  Item2 AS INTEGER
' ^^ Indices of items more & less likely to leave when killed
  ItemChanceRating AS INTEGER
  Status AS INTEGER
  Special1 AS Spell
  SChance1 AS INTEGER
  Special2 AS Spell
  SChance2 AS INTEGER
END TYPE

CONST VER$ = "Ver 1.0 Alpha"
CONST PATH$ = "C:\BG_RPG\MONSTER\"
CONST S1X = 40
CONST S1Y = 40 'NO GREATER THAN 40, Grid Purposes!
CONST S2X = 60
CONST S2Y = 80
CONST S3X = 80
CONST S3Y = 60
CONST S4X = 100
CONST S4Y = 100
CONST S5X = 200
CONST S5Y = 120
CONST Normal = 0
CONST Boss = 1
CONST true = 1
CONST false = 0
DIM SHARED Size AS INTEGER, Attributes AS Monster, MonType AS INTEGER, MNumber AS INTEGER, CoordX AS INTEGER, CoordY AS INTEGER, Col AS INTEGER, XLim AS INTEGER, YLim AS INTEGER, Place AS INTEGER
DIM SHARED Image(12002) AS INTEGER 'largest possible size accounted for
DIM SHARED Cursor(54) AS INTEGER '10 by 10 Get/Put array + Offsets
DIM SHARED Arrow(34) AS INTEGER '8 by 8 Get/Put array
SCREEN 13
LINE (0, 0)-(9, 9), 15, B
LINE (0, 0)-(9, 9), 15
LINE (0, 9)-(9, 0), 15
LINE (4, 4)-(5, 5), 0, BF
GET (0, 0)-(9, 9), Cursor
Cursor(53) = 4 'X Offset for HotSpot
Cursor(54) = 4 'Y Offset for HotSpot
LINE (0, 0)-(9, 9), 0, BF
PSET (1, 1), 15
DRAW "c15d4r2m9,9m5,3u2l4"
PAINT (3, 3), 15
PSET (9, 9), 0
PSET (9, 8), 0
PSET (8, 9), 0
PSET (8, 7), 0
PSET (7, 8), 0
PSET (7, 7), 15
GET (0, 0)-(7, 7), Arrow
PALETTE 255, 0
PALETTE 0, 0
FOR X = 0 TO S1X - 1
  FOR Y = 0 TO S1Y - 1
    PSET (X, Y), 255
  NEXT Y
NEXT X
GET (0, 0)-(S1Y - 1, S1Y - 1), Image
CALL OpeningScreen
SCREEN 13
PALETTE 255, 4136448
PALETTE 0, 0
Col = 0
CoordX = 0
CoordY = 0
Size = 2
MonType = Normal
MNumber = 0
XLim = S2X
YLim = S2Y
1 CALL MakeEditScreen
CALL SelectCol
DO
A$ = INKEY$
SELECT CASE A$
  CASE IS = CHR$(0) + "M" 'Right
    IF CoordX = XLim - 1 THEN
      SOUND 56, .5
     ELSE
      CALL Move(CoordX + 1, CoordY)
     END IF
  CASE IS = CHR$(0) + "K" 'Left
    IF CoordX = 0 THEN
      SOUND 56, .5
     ELSE
      CALL Move(CoordX - 1, CoordY)
     END IF
  CASE IS = CHR$(0) + "P" 'Down
    IF CoordY = YLim - 1 THEN
      SOUND 56, .5
     ELSE
      CALL Move(CoordX, CoordY + 1)
     END IF
  CASE IS = CHR$(0) + "H" 'Up
    IF CoordY = 0 THEN
      SOUND 56, .5
     ELSE
      CALL Move(CoordX, CoordY - 1)
     END IF
  CASE " "
    CALL PutCol
  CASE IS = "C", "c"
    CALL SelectCol
  CASE IS = "T", "t"
    LOCATE 6, 27, 0
    INPUT "Type: ", t$
    SELECT CASE UCASE$(LEFT$(t$, 1))
      CASE IS = "0", "N"
        MonType = Normal
      CASE IS = "1", "B"
        MonType = Boss
    END SELECT
    LOCATE 6, 27, 0
    IF MonType = Normal THEN PRINT "Normal      " ELSE PRINT " Boss       "
  CASE IS = CHR$(0) + ";", "H", "h" 'F1 or H
    CALL Help
  CASE IS = CHR$(0) + "<"
    CALL ChangeCurs
  CASE IS = CHR$(0) + "="
    CALL SetSize
  CASE "G", "g"
    IF Size = 1 THEN
      Col = POINT(CoordX + 202, CoordX + 158)
    ELSE
      Col = POINT(CoordX, CoordY)
    END IF
    SelectCol
END SELECT
LOOP UNTIL A$ = "q" OR A$ = "Q"
CLS
COLOR 7
INPUT "Quit? ", A$
IF LEFT$(UCASE$(A$), 1) = "N" THEN GOTO 1


DEFSNG A-Z
SUB ChangeCol
CLS
PRINT "* Color Editor *"
PRINT ""
IF Col = 255 THEN PRINT "CLEAR" ELSE PRINT Col
LINE (0, 110)-(219, 199), Col, BF
CALL Textfont("(Up)(Down) Switch Color", 221, 111, 7, 0)
CALL Textfont("(Left)(Right) Change Intensity", 221, 121, 7, 0)
CALL Textfont("(Space) Color OK", 221, 131, 7, 0)
LINE (0, 25)-STEP(128, 21), 15, B
LINE (0, 55)-STEP(128, 21), 7, B
LINE (0, 85)-STEP(128, 21), 7, B
R = 0
B = 0
G = 0
Cedit = 1
DO
  PALETTE Col, (65536 * B + 256 * G + R)
  LINE (1, 26)-STEP(126, 19), 0, BF
  LINE (1, 56)-STEP(126, 19), 0, BF
  LINE (1, 86)-STEP(126, 19), 0, BF
  LINE (1, 26)-STEP(R * 2, 19), 4, BF
  LINE (1, 56)-STEP(B * 2, 19), 1, BF
  LINE (1, 86)-STEP(G * 2, 19), 10, BF
  LOCATE 5, 18: PRINT "Red = "; R
  LOCATE 9, 18: PRINT "Blue = "; B
  LOCATE 12, 18: PRINT "Green = "; G
  A$ = INKEY$
  SELECT CASE A$
    CASE IS = CHR$(0) + "M" 'Right
      SELECT CASE Cedit
        CASE 1
          IF R <> 63 THEN R = R + 1
        CASE 2
          IF B <> 63 THEN B = B + 1
        CASE 3
          IF G <> 63 THEN G = G + 1
      END SELECT
    CASE IS = CHR$(0) + "K" 'Left
      SELECT CASE Cedit
        CASE 1
          IF R <> 0 THEN R = R - 1
        CASE 2
          IF B <> 0 THEN B = B - 1
        CASE 3
          IF G <> 0 THEN G = G - 1
      END SELECT
    CASE IS = CHR$(0) + "P" 'Down
      IF Cedit <> 3 THEN
        SELECT CASE Cedit
          CASE 2
            LINE (0, 55)-STEP(128, 21), 7, B
            Cedit = Cedit + 1
            LINE (0, 25 + (30 * (Cedit - 1)))-STEP(128, 21), 15, B
          CASE 1
            LINE (0, 25)-STEP(128, 21), 7, B
            Cedit = Cedit + 1
            LINE (0, 25 + (30 * (Cedit - 1)))-STEP(128, 21), 15, B
        END SELECT
      END IF
    CASE IS = CHR$(0) + "H" 'Up
      IF Cedit <> 1 THEN
        LINE (0, 25 + (30 * (Cedit - 1)))-STEP(128, 21), 7, B
        Cedit = Cedit - 1
        LINE (0, 25 + (30 * (Cedit - 1)))-STEP(128, 21), 15, B
      END IF
  END SELECT
LOOP UNTIL A$ = " "
CLS
MakeEditScreen
END SUB

SUB ChangeCurs
Place = 1
IF Size = 1 THEN
  LOCATE 1, 1, 0
  PRINT "Not Available"
  PRINT "Press a key"
  CALL WaitKey
  CALL MakeEditScreen
  GOTO 2
END IF
CLS
FOR X = 0 TO 9: LINE (X * 10, 0)-STEP(0, 100), 7: NEXT X
FOR Y = 0 TO 9: LINE (0, Y * 10)-STEP(100, 0), 7: NEXT Y '<^Draws Grid
PUT (105, 180), Cursor, OR
DIM TCurs(10, 10)
FOR X = 0 TO 9
  FOR Y = 0 TO 9
    TCurs(X + 1, Y + 1) = POINT(X + 105, Y + 180)
    LINE (X * 10 + 1, Y * 10 + 1)-STEP(8, 8), TCurs(X + 1, Y + 1), BF
  NEXT Y
NEXT X
SelectCol
CX = 0
CY = 0
LINE (0, 0)-(11, 11), 15, B
CALL Textfont("(C) Change Color", 101, 0, 7, 0)
CALL Textfont("(H) Set HotSpot", 101, 10, 7, 0)
CALL Textfont("(F1) Help", 101, 20, 7, 0)
DO
LOCATE 5, 14
PRINT "("; CX + 1; ","; CY + 1; ")  "
LOCATE 6, 14
PRINT "("; Cursor(53) + 1; ","; Cursor(54) + 1; ") :HotSpot  "
A$ = INKEY$
SELECT CASE A$
  CASE IS = CHR$(0) + "M" 'Right
    IF CX = 9 THEN
      SOUND 56, .5
     ELSE
      LINE (CX * 10, CY * 10)-STEP(10, 10), 7, B
      CX = CX + 1
      LINE (CX * 10, CY * 10)-STEP(10, 10), 15, B
     END IF
  CASE IS = CHR$(0) + "K" 'Left
    IF CX = 0 THEN
      SOUND 56, .5
     ELSE
      LINE (CX * 10, CY * 10)-STEP(10, 10), 7, B
      CX = CX - 1
      LINE (CX * 10, CY * 10)-STEP(10, 10), 15, B
     END IF
  CASE IS = CHR$(0) + "P" 'Down
    IF CY = 9 THEN
      SOUND 56, .5
     ELSE
      LINE (CX * 10, CY * 10)-STEP(10, 10), 7, B
      CY = CY + 1
      LINE (CX * 10, CY * 10)-STEP(10, 10), 15, B
     END IF
  CASE IS = CHR$(0) + "H" 'Up
    IF CY = 0 THEN
      SOUND 56, .5
     ELSE
      LINE (CX * 10, CY * 10)-STEP(10, 10), 7, B
      CY = CY - 1
      LINE (CX * 10, CY * 10)-STEP(10, 10), 15, B
     END IF
  CASE " "
    TCurs(CX + 1, CY + 1) = Col
    LINE (CX * 10 + 1, CY * 10 + 1)-STEP(8, 8), Col, BF
    PSET (CX + 105, CY + 180), TCurs(CX + 1, CY + 1)
  CASE "H", "h"
    Cursor(53) = CX
    Cursor(54) = CY
  CASE "c", "C"
    CALL SelectCol
  CASE IS = CHR$(0) + ";" ' F1
    CALL Help
    CLS
    FOR X = 0 TO 10: LINE (X * 10, 0)-STEP(0, 100), 7: NEXT X
    FOR Y = 0 TO 10: LINE (0, Y * 10)-STEP(100, 0), 7: NEXT Y '<^Draws Grid
    FOR X = 0 TO 9
      FOR Y = 0 TO 9
        LINE (X * 10 + 1, Y * 10 + 1)-STEP(8, 8), TCurs(X + 1, Y + 1), BF
      NEXT Y
    NEXT X
    CALL Textfont("(C) Change Color", 101, 0, 7, 0)
    CALL Textfont("(H) Set HotSpot", 101, 10, 7, 0)
    CALL Textfont("(F1) Help", 101, 20, 7, 0)
 CASE "S", "s"
    CALL SLCurs(0)
    CLS
    FOR X = 0 TO 10: LINE (X * 10, 0)-STEP(0, 100), 7: NEXT X
    FOR Y = 0 TO 10: LINE (0, Y * 10)-STEP(100, 0), 7: NEXT Y '<^Draws Grid
    FOR X = 0 TO 9
      FOR Y = 0 TO 9
        LINE (X * 10 + 1, Y * 10 + 1)-STEP(8, 8), TCurs(X + 1, Y + 1), BF
      NEXT Y
    NEXT X
    CALL Textfont("(C) Change Color", 101, 0, 7, 0)
    CALL Textfont("(H) Set HotSpot", 101, 10, 7, 0)
    CALL Textfont("(F1) Help", 101, 20, 7, 0)
  CASE "L", "l"
    CALL SLCurs(1)
    CLS
    FOR X = 0 TO 10: LINE (X * 10, 0)-STEP(0, 100), 7: NEXT X
    FOR Y = 0 TO 10: LINE (0, Y * 10)-STEP(100, 0), 7: NEXT Y '<^Draws Grid
    PUT (105, 180), Cursor, OR
    FOR X = 0 TO 9
      FOR Y = 0 TO 9
        TCurs(X + 1, Y + 1) = POINT(X + 105, Y + 180)
        LINE (X * 10 + 1, Y * 10 + 1)-STEP(8, 8), TCurs(X + 1, Y + 1), BF
      NEXT Y
    NEXT X
    CALL Textfont("(C) Change Color", 101, 0, 7, 0)
    CALL Textfont("(H) Set HotSpot", 101, 10, 7, 0)
    CALL Textfont("(F1) Help", 101, 20, 7, 0)
  CASE "G", "g"
    Col = TCurs(CX + 1, CY + 1)
    SelectCol
END SELECT
LOOP UNTIL UCASE$(A$) = "B"
ERASE TCurs
GET (105, 180)-STEP(9, 9), Cursor
CLS
MakeEditScreen
2 END SUB

SUB Help
LINE (0, 0)-(200, 199), 7, B
LINE (1, 1)-(199, 198), 0, BF
CALL Textfont("* Help *", 95, 15, 7, 0)
SELECT CASE Place
  CASE 0
    CALL Textfont("(F3) Change Size", 20, 25, 7, 0)
    CALL Textfont("(S) Save", 20, 35, 7, 0)
    CALL Textfont("(L) Load", 20, 45, 7, 0)
    CALL Textfont("(A)(F4) Change Attributes", 20, 55, 7, 0)
    CALL Textfont("Look At Top Right For More", 20, 65, 7, 0)
  CASE 1
    CALL Textfont("(B) Back to Monster editor", 20, 25, 7, 0)
    CALL Textfont("(S) Save", 20, 35, 7, 0)
    CALL Textfont("(L) Load", 20, 45, 7, 0)
    CALL Textfont("(G) Get color under Cursor", 20, 55, 7, 0)
    CALL Textfont("Look At Top Right For More", 20, 65, 7, 0)
END SELECT
CALL WaitKey
CALL MakeEditScreen
END SUB

SUB MakeEditScreen
CLS
SELECT CASE Size
  CASE 1
    FOR X = 0 TO S1X
      LINE (X * 5, 0)-STEP(0, 200), 7
    NEXT X
    FOR Y = 0 TO S1Y
      LINE (0, Y * 5)-STEP(200, 0), 7
    NEXT Y
    IF Image(1) <> 0 THEN PUT (202, 158), Image, OR
    FOR X = 1 TO S1X
      FOR Y = 1 TO S1X
        Temp! = POINT(X + 201, Y + 157)
        LINE ((X - 1) * 5 + 1, (Y - 1) * 5 + 1)-STEP(3, 3), Temp!, BF
         IF Temp! = 255 THEN PSET ((X - 1) * 5 + 3, (Y - 1) * 5 + 3), 7
      NEXT Y
    NEXT X
  CASE ELSE
    IF Image(1) <> 0 THEN PUT (0, 0), Image, OR ELSE LINE (0, 0)-STEP(S2X, S2Y), 0, BF
    IF (CoordX - Cursor(53) > 0 AND CoordY - Cursor(54) > 0) THEN
      PUT (CoordX - Cursor(53), CoordY - Cursor(54)), Cursor, OR
     ELSE
      PUT (CoordX, CoordY), Arrow, OR
    END IF
END SELECT
IF Col = 255 THEN
  LOCATE 7, 27, 0
  PRINT "CLEAR"
 ELSE
  LOCATE 7, 27, 0
  PRINT Col
END IF
LOCATE 19, 27, 0
PRINT "("; CoordX + 1; ","; CoordY + 1; ")"
LOCATE 6, 27, 0
IF MonType = Normal THEN PRINT "Normal" ELSE PRINT " Boss "
CALL Textfont("(C) Change color", 201, 1, 15, 0)
CALL Textfont("(T) Change type", 201, 11, 15, 0)
CALL Textfont("(H)(F1) Help", 201, 21, 15, 0)
CALL Textfont("(F2) Change Cursor", 201, 31, 15, 0)
Place = 1
END SUB

SUB Move (X AS INTEGER, Y AS INTEGER)
SELECT CASE Size
  CASE 1
    LINE (CoordX * 5, CoordY * 5)-STEP(5, 5), 7, B
    LINE (X * 5, Y * 5)-STEP(5, 5), 15, B
    CoordX = X
    CoordY = Y
  CASE ELSE
    IF (CoordX - Cursor(53) > 0 AND CoordY - Cursor(54) > 0) THEN
      PUT (CoordX - Cursor(53), CoordY - Cursor(54)), Cursor, OR
     ELSE
      PUT (CoordX, CoordY), Arrow, OR
    END IF
    IF (X - Cursor(53) > 0 AND Y - Cursor(54) > 0) THEN
      PUT (X - Cursor(53), Y - Cursor(54)), Cursor, OR
     ELSE
      PUT (X, Y), Arrow, OR
    END IF
    CoordX = X
    CoordY = Y
END SELECT
LOCATE 19, 27, 0
PRINT "("; CoordX + 1; ","; CoordY + 1; ")    "
END SUB

SUB OpeningScreen
SCREEN 9
'monster editor
COLOR 1
PRINT " лллллллллллл   ллллллллл  лл      лл   лллллллллл   лллллллллл  лллллллллл"
PRINT "  лл   лл   лл   лл     лл  лллл    лл   лл           лл          лл      лл"
PRINT "   лл   лл   лл   лл     лл  лл лл   лл   лл           лл          лллллллллл"
PRINT "    лл   лл   лл   лл     лл  лл  лл  лл   лллллллллл   лллллллллл  лл    лл "
PRINT "     лл   лл   лл   лл     лл  лл   лл лл           лл   лл          лл    лл"
PRINT "      лл        лл   лл     лл  лл    лллл           лл   лл          лл    лл"
PRINT "       лл        лл   ллллллллл  лл      лл   лллллллллл   лллллллллл  лл    лл"
PRINT "                  "
PRINT "                  "
COLOR 7
PRINT "      лллллллллл    лллллл     ллллллллллл   лллллллллллл  лллллллллл  лллллллл"
PRINT "      лл            лл    лл        лл            лл       лл      лл  лл    лл"
PRINT "      лл            лл    лл        лл            лл       лл      лл  лллллллл"
PRINT "      ллллллллл     лл    лл        лл            лл       лл      лл  лл   лл"
PRINT "      лл            лл    лл        лл            лл       лл      лл  лл    лл"
PRINT "      лл            лл    лл        лл            лл       лл      лл  лл    лл"
PRINT "      лллллллллл    лллллл     ллллллллллл        лл       лллллллллл  лл    лл"
PRINT "    " + VER$
WaitKey
END SUB

SUB PutCol
SELECT CASE Size
  CASE 1
    LINE (CoordX * 5 + 1, CoordY * 5 + 1)-STEP(3, 3), Col, BF
    PSET (CoordX + 202, CoordY + 158), Col
    GET (202, 158)-STEP(S1Y - 1, S1X - 1), Image
  CASE ELSE
    IF (CoordX - Cursor(53) > 0 AND CoordY - Cursor(54) > 0) THEN
      PUT (CoordX - Cursor(53), CoordY - Cursor(54)), Cursor, OR
     ELSE
      PUT (CoordX, CoordY), Arrow, OR
    END IF
    PSET (CoordX, CoordY), Col
    GET (0, 0)-(XLim, YLim), Image
    IF (CoordX - Cursor(53) > 0 AND CoordY - Cursor(54) > 0) THEN
      PUT (CoordX - Cursor(53), CoordY - Cursor(54)), Cursor, OR
     ELSE
      PUT (CoordX, CoordY), Arrow, OR
    END IF
END SELECT
END SUB

SUB SelectCol
FOR X = 0 TO 15
  FOR Y = 0 TO 15
     LINE ((X * 5) + 202, (Y * 5) + 61)-STEP(3, 3), ((X * 16) + Y), BF
  NEXT Y
NEXT X
ColX = INT(Col / 16)
ColY = Col MOD 16
LINE ((ColX * 5) + 201, (ColY * 5) + 60)-STEP(5, 5), 15, B
DO
A$ = INKEY$
SELECT CASE A$
  CASE IS = CHR$(0) + "M" 'Right
    IF ColX = 15 THEN
      SOUND 56, .5
     ELSE
      LINE ((ColX * 5) + 201, (ColY * 5) + 60)-STEP(5, 5), 0, B
      ColX = ColX + 1
      LINE ((ColX * 5) + 201, (ColY * 5) + 60)-STEP(5, 5), 15, B
      Col = (ColX * 16) + ColY
      IF Col = 255 THEN
        LOCATE 7, 27, 0
        PRINT "CLEAR"
       ELSE
        LOCATE 7, 27, 0
        PRINT Col
      END IF
    END IF
  CASE IS = CHR$(0) + "K" ' Left
    IF ColX = 0 THEN
      SOUND 56, .5
     ELSE
      LINE ((ColX * 5) + 201, (ColY * 5) + 60)-STEP(5, 5), 0, B
      ColX = ColX - 1
      LINE ((ColX * 5) + 201, (ColY * 5) + 60)-STEP(5, 5), 15, B
      Col = (ColX * 16) + ColY
      IF Col = 255 THEN
        LOCATE 7, 27, 0
        PRINT "CLEAR"
       ELSE
        LOCATE 7, 27, 0
        PRINT Col
      END IF
    END IF
  CASE IS = CHR$(0) + "P" ' Down
    IF ColY = 15 THEN
      SOUND 56, .5
     ELSE
      LINE ((ColX * 5) + 201, (ColY * 5) + 60)-STEP(5, 5), 0, B
      ColY = ColY + 1
      LINE ((ColX * 5) + 201, (ColY * 5) + 60)-STEP(5, 5), 15, B
      Col = (ColX * 16) + ColY
      IF Col = 255 THEN
        LOCATE 7, 27, 0
        PRINT "CLEAR"
       ELSE
        LOCATE 7, 27, 0
        PRINT Col
      END IF
    END IF
  CASE IS = CHR$(0) + "H" ' Up
    IF ColY = 0 THEN
      SOUND 56, .5
     ELSE
      LINE ((ColX * 5) + 201, (ColY * 5) + 60)-STEP(5, 5), 0, B
      ColY = ColY - 1
      LINE ((ColX * 5) + 201, (ColY * 5) + 60)-STEP(5, 5), 15, B
      Col = (ColX * 16) + ColY
      IF Col = 255 THEN
        LOCATE 7, 27, 0
        PRINT "CLEAR"
       ELSE
        LOCATE 7, 27, 0
        PRINT Col
      END IF
    END IF
  CASE IS = " "
    Col = (ColX * 16) + ColY
  CASE IS = "C", "c", "E", "e"
    CALL ChangeCol
  CASE IS = "R", "r"
    PALETTE
    PALETTE 255, 4136448
END SELECT
LOOP UNTIL A$ = " "
IF Size <> 1 AND Place = 0 THEN
  IF (CoordX - Cursor(53) > 0 AND CoordY - Cursor(54) > 0) THEN
    PUT (CoordX - Cursor(53), CoordY - Cursor(54)), Cursor, OR
   ELSE
    PUT (CoordX, CoordY), Arrow, OR
  END IF
END IF
END SUB

SUB SetSize
CLS
PRINT "Current Size:";
SELECT CASE Size
  CASE 1
    PRINT "Pixie ("; S1X; "x"; S1Y; ")"
  CASE 2
    PRINT "Normal ("; S2X; "x"; S2Y; ")"
  CASE 3
    PRINT "Large ("; S3X; "x"; S3Y; ")"
  CASE 4
    PRINT "Giant ("; S4X; "x"; S4Y; ")"
  CASE 5
    PRINT "BOSS ONLY ("; S5X; "x"; S5Y; ")"
END SELECT
PRINT "1. Pixie"
PRINT "2. Normal"
PRINT "3. Large"
PRINT "4. Giant"
PRINT "5. BOSS ONLY"
INPUT "Your Choice: "; S$
SELECT CASE UCASE$(LEFT$(S$, 1))
  CASE "1", "P"
    Size = 1
    XLim = S1X
    YLim = S1Y
  CASE "2", "N"
    Size = 2
    XLim = S2X
    YLim = S2Y
  CASE "3", "L"
    Size = 3
    XLim = S3X
    YLim = S3Y
  CASE "4", "G"
    Size = 4
    XLim = S4X
    YLim = S4Y
  CASE "5", "B"
    Size = 5
    XLim = S5X
    YLim = S5Y
  CASE ELSE
    PRINT "INVALID SIZE!"
    WaitKey
END SELECT
CALL MakeEditScreen
END SUB

SUB SLCurs (MODE AS INTEGER)
3 LINE (0, 0)-(300, 199), 0, BF
LINE (0, 0)-(300, 199), 7, B
IF MODE = 0 THEN
  CALL Textfont("* Save Cursor *", 145, 10, 15, 0)
  COLOR 7
  LOCATE 3, 2
  INPUT "Enter Filename(no EXT): "; Name$
  IF LEN(Name$) > 8 THEN Name$ = LEFT$(Name$, 1)
  IF Name$ = "" THEN GOTO 4 'END SUB
  Name$ = UCASE$(Name$)
  Name$ = Name$ + ".CUR"
  LOCATE 5, 2
  PRINT Name$; ": Filename, right?";
  INPUT  YN$
  IF LEFT$(UCASE$(YN$), 1) <> "Y" THEN GOTO 3
  Name$ = PATH$ + Name$
  OPEN Name$ FOR RANDOM AS #1 LEN = 2
  FOR X = 1 TO 54
    PUT #1, X, Cursor(X)
  NEXT X
  CLOSE #1
 ELSE
  CALL Textfont("* Load Cursor *", 145, 10, 15, 0)
  COLOR 7
  LOCATE 3, 2
  INPUT "Enter Filename(no EXT): "; Name$
  IF LEN(Name$) > 8 THEN Name$ = LEFT$(Name$, 1)
  IF Name$ = "" THEN GOTO 4 'END SUB
  Name$ = UCASE$(Name$)
  Name$ = Name$ + ".CUR"
  LOCATE 5, 2
  PRINT Name$; ": Filename, right?";
  INPUT  YN$
  IF LEFT$(UCASE$(YN$), 1) <> "Y" THEN GOTO 3
  Name$ = PATH$ + Name$
  OPEN Name$ FOR RANDOM AS #1 LEN = 2
  FOR X = 1 TO 54
    GET #1, X, Cursor(X)
  NEXT X
  CLOSE #1
  PUT (105, 180), Cursor, OR

END IF
4 END SUB

'SYNTAX: CALL TEXTFONT ("MESSAGE", X, Y, COLOR,Sound[y/n])
'X AND Y are coordinate locations
'
SUB Textfont (S$, X, Y, C, S)

Y = Y + 4
DRAW "BM=" + VARPTR$(X) + ",=" + VARPTR$(Y) + "C=" + VARPTR$(C)

FOR t = 1 TO LEN(S$)
SELECT CASE UCASE$(MID$(S$, t, 1))

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

IF S = 1 AND UCASE$(MID$(S$, t, 1)) <> " " THEN SOUND 55, .1
IF S = 1 AND UCASE$(MID$(S$, t, 1)) <> " " THEN FOR M = 1 TO (1000 * TD): NEXT M

NEXT t


END SUB

SUB WaitKey
DO
A$ = INKEY$
LOOP UNTIL A$ <> ""
END SUB