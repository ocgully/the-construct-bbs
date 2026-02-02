DECLARE SUB Save ()
DECLARE SUB Load ()
DECLARE SUB ZoomIn ()
DECLARE SUB ZoomOut ()
DECLARE SUB ZMove (X AS INTEGER, Y AS INTEGER, Z!)
DECLARE SUB CType ()
DECLARE SUB SetSize ()
DECLARE SUB Fill (Xf%, Yf%, FCol AS INTEGER)
DECLARE SUB SpecEdit (Mode AS INTEGER)
DECLARE SUB AttrEdit ()
DECLARE SUB PutCol ()
DECLARE SUB SLCurs (Mode AS INTEGER)
DECLARE SUB SLPal (Mode AS INTEGER)
DECLARE SUB ChangeCol ()
DECLARE SUB SelectCol ()
DECLARE SUB Help (Page AS INTEGER)
DECLARE SUB ChangeCurs ()
DECLARE SUB ShowCols ()
DECLARE SUB Move (X AS INTEGER, Y AS INTEGER)
DECLARE FUNCTION MStr$ (X AS INTEGER)
DECLARE SUB Textfont (S$, X%, Y%, C%, S%)
DECLARE SUB MakeEditScreen ()
DECLARE FUNCTION WaitKey$ ()
DECLARE SUB MakeGrid (X AS INTEGER, Y AS INTEGER, Interval AS INTEGER, Tot AS INTEGER, GrCol AS INTEGER)
DECLARE SUB OpeningScreen ()
DEFINT A-Z
OPTION BASE 1
CONST Ver$ = "Version 1.0Alpha"
CONST PATH$ = "C:\BG_RPG\MONSTER\"
CONST ESC = 27, Down = 80, Up = 72, Left = 75, Right = 77, Enter = 13
CONST HOME = 71, ENDKEY = 79, PGDN = 81, PGUP = 73, FK = 58, Backspace = 8
CONST True = 1, False = 0
CONST Normal = 0, Boss = 1
TYPE Spell
  SpellName AS STRING * 10
  SpellDesc AS STRING * 30
  Effect AS INTEGER
  EStrength AS INTEGER
  MPCost AS INTEGER
  HPCost AS INTEGER 'Possibly for sacrifice-Spells,
                    'Take 100 Damage & kill Monster of choice?
END TYPE
TYPE Monster
  MonName AS STRING * 8
  MonType AS INTEGER
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
  Ststus AS INTEGER
  Special1 AS Spell
  SChance1 AS INTEGER
  Special2 AS Spell
  SChance2 AS INTEGER
END TYPE
TYPE Coord
  X AS INTEGER
  Y AS INTEGER
END TYPE
DIM SHARED Ats AS Monster, XCoord AS INTEGER, YCoord AS INTEGER, MN AS INTEGER
DIM SHARED MNT AS INTEGER, Col AS INTEGER, OSize AS INTEGER, Place AS INTEGER
DIM SHARED Size AS INTEGER, ZX AS INTEGER, ZY AS INTEGER, Zoomed AS INTEGER
DIM SHARED Image(12002) AS INTEGER
DIM SHARED ZBox(244) AS INTEGER
DIM SHARED ZImage(20, 20) AS STRING * 1
DIM SHARED CPal(256) AS LONG
DIM SHARED Cursor(65) AS INTEGER
DIM SHARED ColBox(15) AS INTEGER
DIM SHARED S(7) AS Coord
CLEAR , , 40000
MN = 1
FOR X = 1 TO 255
  CPal(X) = -1
NEXT X
Ats.Special1.SpellName = ""
Ats.Special1.SpellDesc = ""
Ats.Special2.SpellName = ""
Ats.Special2.SpellDesc = ""
Ats.MonName = "Monster"
CPal(256) = 4132864
S(1).X = 40
S(1).Y = 40
S(2).X = 60
S(2).Y = 80
S(3).X = 80
S(3).Y = 60
S(4).X = 100
S(4).Y = 100
S(5).X = 200
S(5).Y = 120
S(6).X = 10
S(6).Y = 10
S(7).X = 20
S(7).Y = 20
SCREEN 13
PALETTE USING CPal
LINE (0, 0)-(4, 4), 15, B
GET (0, 0)-(4, 4), ColBox
LINE (0, 0)-(4, 4), 0, B
LINE (0, 0)-(10, 10), 15, B
LINE (0, 0)-(10, 10), 15
LINE (0, 10)-(10, 0), 15
PSET (5, 5), 0
GET (0, 0)-(10, 10), Cursor
Cursor(64) = 5
Cursor(65) = 5
Size = 2
Place = 0
LINE (0, 0)-STEP(S(2).X - 1, S(2).Y - 1), 255, BF
GET (0, 0)-STEP(S(2).X - 1, S(2).Y - 1), Image
10 CALL OpeningScreen
CALL MakeEditScreen
DO
A$ = WaitKey$
SELECT CASE A$
  CASE CHR$(0) + CHR$(Left)
    IF XCoord <> 0 THEN CALL Move(XCoord - 1, YCoord)
  CASE CHR$(0) + CHR$(Right)
    IF XCoord <> S(Size).X - 1 THEN CALL Move(XCoord + 1, YCoord)
  CASE CHR$(0) + CHR$(Up)
    IF YCoord <> 0 THEN CALL Move(XCoord, YCoord - 1)
  CASE CHR$(0) + CHR$(Down)
    IF YCoord <> S(Size).Y - 1 THEN CALL Move(XCoord, YCoord + 1)
  CASE CHR$(0) + CHR$(FK + 1), "H", "h"
    CALL Help(0)
  CASE CHR$(0) + CHR$(FK + 2)
    PUT (XCoord - Cursor(64) + 10, YCoord + Cursor(65) + 10), Cursor
    CALL ChangeCurs
  CASE CHR$(0) + CHR$(FK + 3)
    CALL SetSize
  CASE "C", "c"
    CALL SelectCol
  CASE "G", "g"
    PUT (ColX * 3 + 211, ColY * 3 + 60), ColBox, PSET
    SELECT CASE Size
      CASE 1
        Col = POINT(XCoord + 202, YCoord + 158)
      CASE 2 TO 5
        Col = POINT(XCoord + 10, YCoord + 10)
      CASE 6
        Col = POINT(XCoord, YCoord + 100)
      CASE 7
        Col = POINT(XCoord + 145, YCoord)
    END SELECT
    CALL SelectCol
  CASE "L", "l"
    SELECT CASE Size
      CASE 1
        LINE (211, 8)-STEP(50, 7), 0, BF
        IF Locked THEN
          CALL Textfont("L)ock", 201, 8, 7, 0)
          Locked = False
        ELSE
          CALL Textfont("(L) Unlock", 201, 8, 7, 0)
          Locked = True
        END IF
      CASE 2 TO 5
        LINE (10, S(5).Y + 20)-STEP(50, 7), 0, BF
        IF Locked THEN
          CALL Textfont("L)ock", 10, S(5).Y + 20, 7, 0)
          Locked = False
        ELSE
          CALL Textfont("(L) Unlock", 10, S(5).Y + 20, 7, 0)
          Locked = True
        END IF
      CASE 6
        LINE (0, 150)-STEP(50, 7), 0, BF
        IF Locked THEN
          CALL Textfont("L)ock", 0, 150, 7, 0)
          Locked = False
        ELSE
          CALL Textfont("(L) Unlock", 0, 150, 7, 0)
          Locked = True
        END IF
      CASE 7
        LINE (0, 151)-STEP(50, 7), 0, BF
        IF Locked THEN
          CALL Textfont("L)ock", 0, 151, 7, 0)
          Locked = False
        ELSE
          CALL Textfont("(L) Unlock", 0, 151, 7, 0)
          Locked = True
        END IF
    END SELECT
  CASE " "
    CALL PutCol
  CASE "N", "n"
    LOCATE 1, 1, 1, 7, 0
    INPUT "Name: ", Ats.MonName
    LOCATE , , 0
    CALL MakeEditScreen
  CASE "A", "a"
    CALL AttrEdit
  CASE "F", "f"
    SELECT CASE Size
      CASE 1
        CALL Fill(XCoord + 202, YCoord + 158, -1)
      CASE 2 TO 5
        CALL Fill(XCoord + 10, YCoord + 10, -1)
        PUT (XCoord - Cursor(64) + 10, YCoord - Cursor(65) + 10), Cursor
      CASE 6
        CALL Fill(XCoord, YCoord + 100, -1)
        FOR X = 0 TO 10
          FOR Y = 0 TO 10
            ZImage(X + 1, Y + 1) = CHR$(POINT(X, Y + 100))
          NEXT Y
        NEXT X
        CALL MakeEditScreen

      CASE 7
        CALL Fill(XCoord + 145, YCoord, -1)
        FOR X = 0 TO 19
          FOR Y = 0 TO 19
            ZImage(X + 1, Y + 1) = CHR$(POINT(X + 145, Y))
          NEXT Y
        NEXT X
        CALL MakeEditScreen

    END SELECT
  CASE "Z", "z"
    IF Zoomed THEN CALL ZoomOut ELSE CALL ZoomIn
  CASE "T", "t"
    CALL CType
  CASE "S", "s"
    IF Size < 6 THEN CALL Save
  CASE "O", "o"
    IF Size < 6 THEN CALL Load
END SELECT
IF Locked THEN CALL PutCol
LOOP UNTIL UCASE$(A$) = "Q" OR A$ = CHR$(ESC)
CLS
INPUT "Quit"; YN$
IF UCASE$(LEFT$(LTRIM$(YN$), 1)) = "N" THEN GOTO 10

DEFSNG A-Z
SUB AttrEdit
SCREEN 0
WIDTH 80
COLOR 15
PRINT "* Attribute Editor *"
PRINT "--------------------"
COLOR 7
PRINT "Hit Point Maximum:  "; Ats.HPMax
PRINT "Magic Point Maximum:"; Ats.MPMax
PRINT "Attack:             "; Ats.Attack
PRINT "Defense:            "; Ats.Defense
PRINT "Magic Attack:       "; Ats.MAttack
PRINT "Magic Defense:      "; Ats.MDefense
PRINT "Speed:              "; Ats.Speed
PRINT "Item Chance:        "; Ats.ItemChanceRating
PRINT "Special Chance 1:   "; Ats.SChance1
PRINT "Special Chance 2:   "; Ats.SChance2
PRINT "Special Attack 1... "
PRINT "Special Attack 2... "
CL% = 3
LOCATE 3, 1, 1, 6, 7
DO
LOCATE 3, 1
PRINT "Hit Point Maximum:  "; Ats.HPMax
PRINT "Magic Point Maximum:"; Ats.MPMax
PRINT "Attack:             "; Ats.Attack
PRINT "Defense:            "; Ats.Defense
PRINT "Magic Attack:       "; Ats.MAttack
PRINT "Magic Defense:      "; Ats.MDefense
PRINT "Speed:              "; Ats.Speed
PRINT "Item Chance:        "; Ats.ItemChanceRating
PRINT "Special Chance 1:   "; Ats.SChance1
PRINT "Special Chance 2:   "; Ats.SChance2
LOCATE CL%, 1
A$ = WaitKey$
SELECT CASE A$
  CASE CHR$(0) + CHR$(Up)
    IF CL% <> 3 THEN CL% = CL% - 1
  CASE CHR$(0) + CHR$(Down)
    IF CL% <> 14 THEN CL% = CL% + 1
  CASE CHR$(Enter)
    IF CL% < 13 THEN CL% = CL% + 1
    IF CL% = 13 THEN
      CALL SpecEdit(1)
      LOCATE 1, 1
      COLOR 15
      PRINT "* Attribute Editor *"
      PRINT "--------------------"
      COLOR 7
      LOCATE 13, 1
      PRINT "Special Attack 1... "
      PRINT "Special Attack 2... "
    END IF
    IF CL% = 14 THEN
      CALL SpecEdit(2)
      LOCATE 13, 1
      PRINT "Special Attack 1... "
      PRINT "Special Attack 2... "
    END IF
  CASE "1" TO "9"
    SELECT CASE CL%
      CASE 3
        Ats.HPMax = Ats.HPMax * 10 + ASC(A$) - 48
      CASE 4
        Ats.MPMax = Ats.MPMax * 10 + ASC(A$) - 48
      CASE 5
        Ats.Attack = Ats.Attack * 10 + ASC(A$) - 48
      CASE 6
        Ats.Defense = Ats.Defense * 10 + ASC(A$) - 48
      CASE 7
        Ats.MAttack = Ats.MAttack * 10 + ASC(A$) - 48
      CASE 8
        Ats.MDefense = Ats.MDefense * 10 + ASC(A$) - 48
      CASE 9
        Ats.Speed = Ats.Speed * 10 + ASC(A$) - 48
      CASE 10
        Ats.ItemChanceRating = Ats.ItemChanceRating * 10 + ASC(A$) - 48
      CASE 11
        Ats.SChance1 = Ats.SChance1 * 10 + ASC(A$) - 48
      CASE 12
        Ats.SChance2 = Ats.SChance2 * 10 + ASC(A$) - 48
    END SELECT
  CASE "0"
    SELECT CASE CL%
      CASE 3
        Ats.HPMax = Ats.HPMax * 10
      CASE 4
        Ats.MPMax = Ats.MPMax * 10
      CASE 5
        Ats.Attack = Ats.Attack * 10
      CASE 6
        Ats.Defense = Ats.Defense * 10
      CASE 7
        Ats.MAttack = Ats.MAttack * 10
      CASE 8
        Ats.MDefense = Ats.MDefense * 10
      CASE 9
        Ats.Speed = Ats.Speed * 10
      CASE 10
        Ats.ItemChanceRating = Ats.ItemChanceRating * 10
      CASE 11
        Ats.SChance1 = Ats.SChance1 * 10
      CASE 12
        Ats.SChance2 = Ats.SChance2 * 10
    END SELECT
  CASE CHR$(Backspace)
    SELECT CASE CL%
      CASE 3
        Ats.HPMax = INT(Ats.HPMax / 10)
      CASE 4
        Ats.MPMax = INT(Ats.MPMax / 10)
      CASE 5
        Ats.Attack = INT(Ats.Attack / 10)
      CASE 6
        Ats.Defense = INT(Ats.Defense / 10)
      CASE 7
        Ats.MAttack = INT(Ats.MAttack / 10)
      CASE 8
        Ats.MDefense = INT(Ats.MDefense / 10)
      CASE 9
        Ats.Speed = INT(Ats.Speed / 10)
      CASE 10
        Ats.ItemChanceRating = INT(Ats.ItemChanceRating / 10)
      CASE 11
        Ats.SChance1 = INT(Ats.SChance1 / 10)
      CASE 12
        Ats.SChance2 = INT(Ats.SChance2 / 10)
    END SELECT
END SELECT
LOOP UNTIL A$ = CHR$(ESC) OR UCASE$(A$) = "B"
SCREEN 13
PALETTE USING CPal
MakeEditScreen
END SUB

SUB ChangeCol
CLS
COLOR 15
PRINT "* Color Editor *"
LINE (0, 100)-(200, 199), Col, BF
IF CPal(Col - 1) = -1 THEN
  R = 0
  G = 0
  B = 0
 ELSE
  B = INT(CPal(Col - 1) / 65536)
  G = INT((CPal(Col - 1) MOD 65536) / 256)
  R = CPal(Col - 1) MOD 256
END IF
LINE (0, 10)-STEP(128, 8), 15, B
LINE (0, 20)-STEP(128, 8), 7, B
LINE (0, 30)-STEP(128, 8), 7, B
C = 0
COLOR 7
DO
  CPal(Col - 1) = 65536 * B + 256 * G + R
  PALETTE Col, CPal(Col - 1)
  A$ = INKEY$
  LINE (1, 11)-STEP(126, 6), 0, BF
  LINE (1, 21)-STEP(126, 6), 0, BF
  LINE (1, 31)-STEP(126, 6), 0, BF
  LINE (1, 11)-STEP(B * 2, 6), 1, BF
  LINE (1, 21)-STEP(G * 2, 6), 10, BF
  LINE (1, 31)-STEP(R * 2, 6), 4, BF
  LOCATE 2, 18
  PRINT "Red:"; R
  LOCATE 3, 18
  PRINT "Blue:"; B
  LOCATE 4, 18
  PRINT "Green:"; G

  SELECT CASE A$
    CASE CHR$(0) + CHR$(Left)
      SELECT CASE C
        CASE 0
          IF B <> 0 THEN B = B - 1
        CASE 1
          IF G <> 0 THEN G = G - 1
        CASE 2
          IF R <> 0 THEN R = R - 1
        END SELECT
    CASE CHR$(0) + CHR$(Right)
      SELECT CASE C
        CASE 0
          IF B <> 63 THEN B = B + 1
        CASE 1
          IF G <> 63 THEN G = G + 1
        CASE 2
          IF R <> 63 THEN R = R + 1
        END SELECT
    CASE CHR$(0) + CHR$(Up)
      IF C <> 0 THEN
        LINE (0, (C + 1) * 10)-STEP(128, 8), 7, B
        C = C - 1
        LINE (0, (C + 1) * 10)-STEP(128, 8), 15, B
      END IF
    CASE CHR$(0) + CHR$(Down)
      IF C <> 3 THEN
        LINE (0, (C + 1) * 10)-STEP(128, 8), 7, B
        C = C + 1
        LINE (0, (C + 1) * 10)-STEP(128, 8), 15, B
      END IF
  END SELECT
LOOP UNTIL A$ = " " OR A$ = CHR$(ESC)
SELECT CASE Place
  CASE 0
    CALL MakeEditScreen
  CASE 1
    CLS
    CALL MakeGrid(0, 0, 10, 11, 7)
    PUT (0, 111), Cursor, PSET
    CALL ShowCols
END SELECT
END SUB

SUB ChangeCurs
Place = 1
DIM Curs(11, 11) AS STRING * 1
CLS
CALL MakeGrid(0, 0, 10, 11, 7)
PUT (0, 111), Cursor, PSET
FOR X = 0 TO 10
  FOR Y = 0 TO 10
    Curs(X + 1, Y + 1) = CHR$(POINT(X, Y + 111))
    LINE (X * 10 + 1, Y * 10 + 1)-STEP(8, 8), ASC(Curs(X + 1, Y + 1)), BF
  NEXT Y
NEXT X
LINE (0, 0)-(10, 10), 15, B
CALL ShowCols
CX = 0
CY = 0
DO
LINE (0, 140)-(100, 155), 0, BF
CALL Textfont("(" + MStr$(CX + 1) + "," + MStr$(CY + 1) + ")   ", 0, 140, 7, 0)
CALL Textfont("(" + MStr$(Cursor(64) + 1) + "," + MStr$(Cursor(65) + 1) + ") HotSpot   ", 0, 148, 7, 0)
A$ = WaitKey$
SELECT CASE A$
  CASE CHR$(0) + CHR$(Left)
    IF CX <> 0 THEN
      LINE (CX * 10, CY * 10)-STEP(10, 10), 7, B
      CX = CX - 1
      LINE (CX * 10, CY * 10)-STEP(10, 10), 15, B
    END IF
  CASE CHR$(0) + CHR$(Right)
    IF CX <> 10 THEN
      LINE (CX * 10, CY * 10)-STEP(10, 10), 7, B
      CX = CX + 1
      LINE (CX * 10, CY * 10)-STEP(10, 10), 15, B
    END IF
  CASE CHR$(0) + CHR$(Up)
    IF CY <> 0 THEN
      LINE (CX * 10, CY * 10)-STEP(10, 10), 7, B
      CY = CY - 1
      LINE (CX * 10, CY * 10)-STEP(10, 10), 15, B
    END IF
  CASE CHR$(0) + CHR$(Down)
    IF CY <> 10 THEN
      LINE (CX * 10, CY * 10)-STEP(10, 10), 7, B
      CY = CY + 1
      LINE (CX * 10, CY * 10)-STEP(10, 10), 15, B
    END IF
  CASE CHR$(0) + CHR$(FK + 1)
    CALL Help(2)
    FOR X = 0 TO 10
      FOR Y = 0 TO 10
        LINE (X * 10 + 1, Y * 10 + 1)-STEP(8, 8), ASC(Curs(X + 1, Y + 1)), BF
      NEXT Y
    NEXT X
    LINE (CX * 10, CY * 10)-STEP(10, 10), 15, B
  CASE " "
    Curs(CX + 1, CY + 1) = CHR$(Col)
    PSET (CX, CY + 111), Col
    LINE (CX * 10 + 1, CY * 10 + 1)-STEP(8, 8), Col, BF
    GET (0, 111)-STEP(10, 10), Cursor
  CASE "C", "c"
    CALL SelectCol
    FOR X = 0 TO 10
      FOR Y = 0 TO 10
        LINE (X * 10 + 1, Y * 10 + 1)-STEP(8, 8), ASC(Curs(X + 1, Y + 1)), BF
      NEXT Y
    NEXT X
    LINE (CX * 10, CY * 10)-STEP(10, 10), 15, B
  CASE "H", "h"
    Cursor(64) = CX
    Cursor(65) = CY
  CASE "S", "s"
    CALL SLCurs(0)
    CALL MakeGrid(0, 0, 10, 11, 7)
    PUT (0, 111), Cursor, PSET
    FOR X = 0 TO 10
      FOR Y = 0 TO 10
        Curs(X + 1, Y + 1) = CHR$(POINT(X, Y + 111))
        LINE (X * 10 + 1, Y * 10 + 1)-STEP(8, 8), ASC(Curs(X + 1, Y + 1)), BF
      NEXT Y
    NEXT X
    LINE (0, 0)-(10, 10), 15, B
    CALL ShowCols
    CX = 0
    CY = 0
  CASE "L", "l"
    CALL SLCurs(1)
    CALL MakeGrid(0, 0, 10, 11, 7)
    PUT (0, 111), Cursor, PSET
    FOR X = 0 TO 10
      FOR Y = 0 TO 10
        Curs(X + 1, Y + 1) = CHR$(POINT(X, Y + 111))
        LINE (X * 10 + 1, Y * 10 + 1)-STEP(8, 8), ASC(Curs(X + 1, Y + 1)), BF
      NEXT Y
    NEXT X
    LINE (0, 0)-(10, 10), 15, B
    CALL ShowCols
    CX = 0
    CY = 0
  CASE "G", "g"
    PUT (ColX * 3 + 211, ColY * 3 + 60), ColBox, PSET
    Col = POINT(CX, CY + 111)
    CALL SelectCol
END SELECT
LOOP UNTIL UCASE$(A$) = "R" OR A$ = CHR$(ESC)
CLS
MakeEditScreen
END SUB

SUB CType
COLOR 7
DO
SELECT CASE Size
  CASE 1
    LOCATE 23, 1
    IF Ats.MonType THEN PRINT " Boss ";  ELSE PRINT "Normal";
  CASE 2 TO 5
    LOCATE 23, 2
    IF Ats.MonType THEN PRINT " Boss ";  ELSE PRINT "Normal";
  CASE 6
    LOCATE 25, 10
    IF Ats.MonType THEN PRINT " Boss ";  ELSE PRINT "Normal";
  CASE 7
    LOCATE 25, 10
    IF Ats.MonType THEN PRINT " Boss ";  ELSE PRINT "Normal";
END SELECT
A$ = WaitKey$
SELECT CASE A$
  CASE "N", "n", "0"
    Ats.MonType = 0
  CASE "B", "b", "1"
    Ats.MonType = 1
END SELECT
LOOP UNTIL A$ = CHR$(Enter) OR A$ = CHR$(ESC)
END SUB

SUB Fill (Xf%, Yf%, FCol AS INTEGER)
IF FCol = -1 THEN
  IF Size > 1 AND Size < 6 THEN PUT (XCoord - Cursor(64) + 10, YCoord - Cursor(65) + 10), Cursor
  FCol = POINT(Xf%, Yf%)
  IF FCol = Col THEN EXIT SUB
END IF
PSET (Xf%, Yf%), Col
FOR X% = Xf% - 1 TO Xf% + 1
  FOR Y% = Yf% - 1 TO Yf% + 1
    IF POINT(X%, Y%) = FCol THEN CALL Fill(X%, Y%, FCol)
  NEXT Y%
NEXT X%
END SUB

SUB Help (Pg AS INTEGER)
SCREEN 0
WIDTH 80
Page = Pg
51 SELECT CASE Page
  CASE 0
    CLS
    COLOR 15
    PRINT "* Main Help *"
    PRINT "============="
    COLOR 7
    PRINT "(Space) Put Color"
    PRINT "(L)ock Color (cursor flashes when locked)"
    PRINT "(C)hange Color"
    PRINT "(Z)oom in/out"
    PRINT "(F1)(H)elp"
    PRINT "(F2) Cursor Edit"
    PRINT "(F3) Change Size"
    PRINT "(N)ame monster"
    PRINT "(T)ype change (Normal/Boss)"
    PRINT "(S)ave"
    PRINT "(O)pen"
    PRINT "(G)et color under cursor"
    PRINT "(R)eset"
    PRINT ""
    PRINT "1. This Screen"
    PRINT "2. Select Color Help"
    PRINT "3. Cursor Edit Help"
  CASE 1
    CLS
    COLOR 15
    PRINT "* Select Color Help *"
    PRINT "====================="
    COLOR 7
    PRINT "(Space) Pick Color"
    PRINT "(C)(E)dit Color"
    PRINT "(R)eset Palette"
    PRINT "(S)ave Palette"
    PRINT "(L)oad Palette"
    PRINT ""
    PRINT "1. Main Help"
    PRINT "2. This Page"
    PRINT "3. Cursor Edit Help"
  CASE 2
    CLS
    COLOR 15
    PRINT "* Cursor Edit Help *"
    PRINT "===================="
    COLOR 7
    PRINT "(C)hange Color"
    PRINT "(F1) Help"
    PRINT "(S)ave Cursor"
    PRINT "(L)oad Cursor"
    PRINT "(G)et color under cursor"
    PRINT "(H) Set HotSpot"
    PRINT "(R)eturn to Main"
    PRINT ""
    PRINT "1. Main Help"
    PRINT "2. Select Color Help"
    PRINT "3. This Page"
END SELECT
A$ = WaitKey$
SELECT CASE ASC(A$)
  CASE 49 TO 57
    Page = ASC(A$) - 49
    GOTO 51
END SELECT
SCREEN 13
PALETTE USING CPal
IF Pg = 0 OR Pg = 1 THEN CALL MakeEditScreen
IF Pg = 2 THEN
  CLS
  CALL MakeGrid(0, 0, 10, 11, 7)
  PUT (0, 111), Cursor, PSET
  CALL ShowCols
END IF
END SUB

SUB Load
SCREEN 0
WIDTH 80
COLOR 15
PRINT "* LOAD *"
PRINT "--------"
COLOR 7
INPUT "Filename: ", F$
IF LEN(F$) > 8 THEN F$ = LEFT$(A$, 8)
PRINT UCASE$(F$) + ".MON, right? No"
Load2 = False
DO
A$ = INKEY$
SELECT CASE A$
  CASE "Y", "y"
    LOCATE 4, LEN(A$) + 13
    PRINT "Yes"
    Load2 = True
  CASE "N", "n"
    LOCATE 4, LEN(A$) + 13
    PRINT "No"
    Load2 = False
END SELECT
LOOP UNTIL A$ = CHR$(Enter)
IF Load2 = True THEN
  OPEN PATH$ + F$ + ".MON" FOR RANDOM AS #1 LEN = 2
  GET #1, 1, MN
  GET #1, 2, Size
  FOR X = 1 TO CINT((S(Size).X * S(Size).Y + 4) / 2)
    GET #1, X + 2, Image(X)
  NEXT X
  CLOSE #1
  OPEN PATH$ + "MONATTR.DAT" FOR RANDOM AS #1 LEN = LEN(Ats)
  GET #1, MN, Ats
  CLOSE #1
END IF
SCREEN 13
XCoord = 0
YCoord = 0
PALETTE USING CPal
MakeEditScreen
END SUB

SUB MakeEditScreen
CLS
SELECT CASE Size
  CASE 1
    CALL MakeGrid(0, 0, 5, 40, 7)
    LINE (XCoord * 5, YCoord * 5)-STEP(5, 5), 15, B
    PUT (202, 158), Image
    FOR X = 0 TO S(1).X - 1
      FOR Y = 0 TO S(1).Y - 1
        LINE (X * 5 + 1, Y * 5 + 1)-STEP(3, 3), POINT(X + 202, Y + 158), BF
      NEXT Y
    NEXT X
    CALL Textfont("H)elp", 201, 0, 7, 0)
    IF NOT Locked THEN CALL Textfont("L)ock", 201, 8, 7, 0) ELSE CALL Textfont("(L) Unlock", 201, 8, 7, 0)
    CALL Textfont("C)olor", 201, 16, 7, 0)
    CALL Textfont("(F3) Size", 201, 24, 7, 0)
    CALL Textfont("(" + MStr$(XCoord + 1) + "," + MStr$(YCoord + 1) + ")   ", 201, 32, 7, 0)
    COLOR 7
    LOCATE 6, 27
    IF Ats.MonType THEN PRINT " Boss ";  ELSE PRINT "Normal";
  CASE 2 TO 5
    COLOR 15
    LOCATE 1, 1, 0
    PRINT ("Name: " + Ats.MonName)
    PUT (10, 10), Image
    LINE (9, 9)-STEP(S(Size).X + 1, S(Size).Y + 1), 15, B
    PUT (XCoord - Cursor(64) + 10, YCoord - Cursor(65) + 10), Cursor
    CALL Textfont("H)elp", 10, S(5).Y + 12, 7, 0)
    CALL Textfont("L)ock", 10, S(5).Y + 20, 7, 0)
    CALL Textfont("C)olor", 10, S(5).Y + 28, 7, 0)
    CALL Textfont("(F3) Size", 10, S(5).Y + 36, 7, 0)
    CALL Textfont("(" + MStr$(XCoord + 1) + "," + MStr$(YCoord + 1) + ")   ", 10, S(5).Y + 44, 7, 0)
    COLOR 7
    LOCATE 23, 2
    IF Ats.MonType THEN PRINT " Boss ";  ELSE PRINT "Normal";
  CASE 6
    CALL MakeGrid(0, 0, 10, 10, 7)
    LINE (XCoord * 10, YCoord * 10)-STEP(10, 10), 15, B
    FOR X = 0 TO S(6).X - 1
      FOR Y = 0 TO S(6).Y - 1
        LINE (X * 10 + 1, Y * 10 + 1)-STEP(8, 8), ASC(ZImage(X + 1, Y + 1)), BF
        PSET (X - 1, Y + 101), ASC(ZImage(X + 1, Y + 1))
      NEXT Y
    NEXT X
    CALL Textfont("H)elp", 0, 140, 7, 0)
    CALL Textfont("L)ock", 0, 150, 7, 0)
    CALL Textfont("C)olor", 0, 160, 7, 0)
    CALL Textfont("Z)oom Out", 0, 170, 7, 0)
    CALL Textfont("(" + MStr$(XCoord + 1) + "," + MStr$(YCoord + 1) + ")   ", 0, 180, 7, 0)
    COLOR 7
    LOCATE 25, 10
    IF Ats.MonType THEN PRINT " Boss ";  ELSE PRINT "Normal";
  CASE 7
    CALL MakeGrid(0, 0, 7, 20, 7)
    LINE (XCoord * 7, YCoord * 7)-STEP(7, 7), 15, B
    FOR X = 0 TO 19
      FOR Y = 0 TO 19
        LINE (X * 7 + 1, Y * 7 + 1)-STEP(5, 5), ASC(ZImage(X + 1, Y + 1)), BF
        PSET (X + 145, Y), ASC(ZImage(X + 1, Y + 1))
      NEXT Y
    NEXT X
    CALL Textfont("H)elp", 0, 141, 7, 0)
    CALL Textfont("L)ock", 0, 151, 7, 0)
    CALL Textfont("C)olor", 0, 161, 7, 0)
    CALL Textfont("Z)oom Out", 0, 171, 7, 0)
    CALL Textfont("(" + MStr$(XCoord + 1) + "," + MStr$(YCoord + 1) + ")   ", 0, 181, 7, 0)
    COLOR 7
    LOCATE 25, 10
    IF Ats.MonType THEN PRINT " Boss ";  ELSE PRINT "Normal";
END SELECT
CALL ShowCols
END SUB

SUB MakeGrid (X AS INTEGER, Y AS INTEGER, Interval AS INTEGER, Tot AS INTEGER, GrCol AS INTEGER)
FOR XY = 0 TO Tot
  LINE (XY * Interval + X, Y)-STEP(0, Tot * Interval), GrCol
  LINE (X, XY * Interval + Y)-STEP(Tot * Interval, 0), GrCol
NEXT XY
END SUB

SUB Move (X AS INTEGER, Y AS INTEGER)
SELECT CASE Size
  CASE 1
    LINE (XCoord * 5, YCoord * 5)-STEP(5, 5), 7, B
    LINE (X * 5, Y * 5)-STEP(5, 5), 15, B
    LINE (201, 32)-STEP(35, 7), 0, BF
    CALL Textfont("(" + MStr$(X + 1) + "," + MStr$(Y + 1) + ")   ", 201, 32, 7, 0)
  CASE 2 TO 5
    PUT (XCoord - Cursor(64) + 10, YCoord - Cursor(65) + 10), Cursor
    PUT (X - Cursor(64) + 10, Y - Cursor(65) + 10), Cursor
    LINE (10, S(5).Y + 44)-STEP(35, 7), 0, BF
    CALL Textfont("(" + MStr$(X + 1) + "," + MStr$(Y + 1) + ")   ", 10, S(5).Y + 44, 7, 0)
  CASE 6
    LINE (XCoord * 10, YCoord * 10)-STEP(10, 10), 7, B
    LINE (X * 10, Y * 10)-STEP(10, 10), 15, B
    LINE (0, 180)-STEP(35, 7), 0, BF
    CALL Textfont("(" + MStr$(X + ZX + 1) + "," + MStr$(Y + ZY + 1) + ")   ", 0, 180, 7, 0)
  CASE 7
    LINE (XCoord * 7, YCoord * 7)-STEP(7, 7), 7, B
    LINE (X * 7, Y * 7)-STEP(7, 7), 15, B
    LINE (0, 181)-STEP(35, 7), 0, BF
    CALL Textfont("(" + MStr$(X + ZX + 1) + "," + MStr$(Y + ZY + 1) + ")   ", 0, 181, 7, 0)
END SELECT
XCoord = X
YCoord = Y
END SUB

FUNCTION MStr$ (X AS INTEGER)
FOR Z = 5 TO 1 STEP -1
  IF (10 ^ Z > ABS(X)) THEN L = Z
NEXT Z
IF ABS(X) <> X THEN Neg = True ELSE Neg = False
IF L > 1 THEN
  DIM N(L) AS INTEGER
  DIM St(L) AS STRING * 1
  FOR Z = 1 TO L
    N(Z) = (INT(X / (10 ^ (Z - 1))) MOD 10)
    IF N(Z) = 0 THEN St(Z) = "0" ELSE St(Z) = CHR$(N(Z) + 48)
  NEXT Z
  SELECT CASE L
    CASE 1
      MStr$ = St(1)
    CASE 2
      MStr$ = St(2) + St(1)
    CASE 3
      MStr$ = St(3) + St(2) + St(1)
    CASE 4
      MStr$ = St(4) + St(3) + St(2) + St(1)
    CASE 5
      MStr$ = St(5) + St(4) + St(3) + St(2) + St(1)
  END SELECT
  ERASE St
  ERASE N
 ELSE
  IF X = 0 THEN MStr$ = "0" ELSE MStr$ = CHR$(X + 48)
END IF
END FUNCTION

SUB OpeningScreen
SCREEN 0
WIDTH 80
'monster editor
COLOR 1
PRINT " лллллллллллл ллллллллл лл      лл лллллллллл лллллллллллл лллллллллл ллллллл"
PRINT "лл   лл   лл лл     лл лллл    лл лл              лл      лл         лл   лл"
PRINT " лл   лл   лл лл     лл лл лл   лл лл              лл      лл         ллллллл"
PRINT "лл   лл   лл лл     лл лл  лл  лл лллллллллл      лл      лллллллллл лл  лл "
PRINT " лл   лл   лл лл     лл лл   лл лл         лл      лл      лл         лл  лл"
PRINT "лл        лл лл     лл лл    лллл         лл      лл      лл         лл    лл"
PRINT " лл        лл ллллллллл лл      лл лллллллллл      лл      лллллллллл лл    лл"
PRINT "                  "
PRINT "                  "
COLOR 7
PRINT "     лллллллллл ллллллл   ллллллллллл лллллллллллл лллллллллл лллллллл"
PRINT "      лл         лл    лл       лл          лл      лл      лл лл    лл"
PRINT "       лл         лл    лл       лл          лл      лл      лл лллллллл"
PRINT "        ллллллллл  лл    лл       лл          лл      лл      лл лл   лл"
PRINT "         лл         лл    лл       лл          лл      лл      лл лл   лл"
PRINT "          лл         лл    лл       лл          лл      лл      лл лл   лл"
PRINT "           лллллллллл лллллл    ллллллллллл      лл      лллллллллл лл   лл"
PRINT Ver$
A$ = WaitKey$
SCREEN 13
PALETTE USING CPal
END SUB

SUB PutCol
SELECT CASE Size
  CASE 1
    LINE (XCoord * 5 + 1, YCoord * 5 + 1)-STEP(3, 3), Col, BF
    PSET (XCoord + 202, YCoord + 158), Col
    GET (202, 158)-STEP(39, 39), Image
  CASE 2 TO 5
    PSET (XCoord + 10, YCoord + 10), Col
    GET (10, 10)-STEP(S(Size).X - 1, S(Size).Y - 1), Image
  CASE 6
    LINE (XCoord * 10 + 1, YCoord * 10 + 1)-STEP(8, 8), Col, BF
    PSET (XCoord, YCoord + 100), Col
    ZImage(XCoord + 1, YCoord + 1) = CHR$(Col)
  CASE 7
    LINE (XCoord * 7 + 1, YCoord * 7 + 1)-STEP(5, 5), Col, BF
    PSET (XCoord + 145, YCoord), Col
    ZImage(XCoord + 1, YCoord + 1) = CHR$(Col)
END SELECT
END SUB

SUB Save
SCREEN 0
WIDTH 80
COLOR 15
PRINT "* SAVE *"
PRINT "--------"
COLOR 7
PRINT UCASE$(Ats.MonName) + ".MON, right? No"
Save2 = False
DO
A$ = INKEY$
SELECT CASE A$
  CASE "Y", "y"
    LOCATE 3, LEN(Ats.MonName) + 13
    PRINT "Yes"
    Save2 = True
  CASE "N", "n"
    LOCATE 3, LEN(Ats.MonName) + 13
    PRINT "No"
    Save2 = False
END SELECT
LOOP UNTIL A$ = CHR$(Enter)
IF Save2 = True THEN
  OPEN PATH$ + "MONATTR.DAT" FOR RANDOM AS #1 LEN = LEN(Ats)
  IF MN < 2 THEN
    PUT #1, MN, Ats
   ELSE
    GET #1, 1, MNT
    MNT = MNT + 1
    MN = MNT + 2
    PUT #1, MN, Ats
    PUT #1, 1, MNT
  END IF
  CLOSE #1
  OPEN PATH$ + Ats.MonName + ".MON" FOR RANDOM AS #1 LEN = 2
  PUT #1, 1, MN
  PUT #1, 2, Size
  FOR X = 1 TO CINT((S(Size).X * S(Size).Y + 4) / 2)
    PUT #1, X + 2, Image(X)
  NEXT X
  CLOSE #1
END IF
SCREEN 13
PALETTE USING CPal
MakeEditScreen
END SUB

SUB SelectCol
FOR X = 0 TO 15
  FOR Y = 0 TO 15
    LINE (X * 3 + 212, Y * 3 + 61)-STEP(2, 2), (X * 16) + Y, BF
  NEXT Y
NEXT X
ColX = INT(Col / 16)
ColY = Col MOD 16
LINE (211, 60)-STEP(49, 49), 0, B
GET (ColX * 3 + 211, ColY * 3 + 60)-STEP(4, 4), ColBox
LINE (ColX * 3 + 211, ColY * 3 + 60)-STEP(4, 4), 15, B
DO
LINE (211, 110)-STEP(15, 7), 0, BF
CALL Textfont(MStr$(Col), 211, 110, 7, 0)
A$ = WaitKey$
SELECT CASE A$
  CASE CHR$(0) + CHR$(Left)
    IF ColX <> 0 THEN
      PUT (ColX * 3 + 211, ColY * 3 + 60), ColBox, PSET
      ColX = ColX - 1
      GET (ColX * 3 + 211, ColY * 3 + 60)-STEP(4, 4), ColBox
      LINE (ColX * 3 + 211, ColY * 3 + 60)-STEP(4, 4), 15, B
      Col = (ColX * 16) + ColY
    END IF
  CASE CHR$(0) + CHR$(Right)
    IF ColX <> 15 THEN
      PUT (ColX * 3 + 211, ColY * 3 + 60), ColBox, PSET
      ColX = ColX + 1
      GET (ColX * 3 + 211, ColY * 3 + 60)-STEP(4, 4), ColBox
      LINE (ColX * 3 + 211, ColY * 3 + 60)-STEP(4, 4), 15, B
      Col = (ColX * 16) + ColY
    END IF
  CASE CHR$(0) + CHR$(Up)
    IF ColY <> 0 THEN
      PUT (ColX * 3 + 211, ColY * 3 + 60), ColBox, PSET
      ColY = ColY - 1
      GET (ColX * 3 + 211, ColY * 3 + 60)-STEP(4, 4), ColBox
      LINE (ColX * 3 + 211, ColY * 3 + 60)-STEP(4, 4), 15, B
      Col = (ColX * 16) + ColY
    END IF
  CASE CHR$(0) + CHR$(Down)
    IF ColY <> 15 THEN
      PUT (ColX * 3 + 211, ColY * 3 + 60), ColBox, PSET
      ColY = ColY + 1
      GET (ColX * 3 + 211, ColY * 3 + 60)-STEP(4, 4), ColBox
      LINE (ColX * 3 + 211, ColY * 3 + 60)-STEP(4, 4), 15, B
      Col = (ColX * 16) + ColY
    END IF
  CASE CHR$(0) + CHR$(FK + 1)
    CALL Help(1)
  CASE "C", "c", "E", "e"
    IF Col > 15 THEN CALL ChangeCol
  CASE "R", "r"
    FOR C = 1 TO 255
      CPal(C) = -1
    NEXT C
    PALETTE USING CPal
  CASE "S", "s"
    CALL SLPal(0)
  CASE "L", "l"
    CALL SLPal(1)
END SELECT
LOOP UNTIL A$ = CHR$(ESC) OR A$ = " "
END SUB

SUB SetSize
SCREEN 0
WIDTH 80
PRINT "* Set Size *"
PRINT "------------"
PRINT "Current size: "
LOCATE 3, 15
SELECT CASE Size
  CASE 1
    PRINT "Pixie (" + MStr$(S(1).X) + "x" + MStr$(S(1).Y) + ")"
  CASE 2
    PRINT "Normal (" + MStr$(S(2).X) + "x" + MStr$(S(2).Y) + ")"
  CASE 3
    PRINT "Horizontal (" + MStr$(S(3).X) + "x" + MStr$(S(3).Y) + ")"
  CASE 4
    PRINT "Giant (" + MStr$(S(4).X) + "x" + MStr$(S(4).Y) + ")"
  CASE 5
    PRINT "BOSS! (" + MStr$(S(5).X) + "x" + MStr$(S(5).Y) + ")"
END SELECT
PRINT "Sizes are:"
PRINT "1. Pixie (" + MStr$(S(1).X) + "x" + MStr$(S(1).Y) + ")"
PRINT "2. Normal (" + MStr$(S(2).X) + "x" + MStr$(S(2).Y) + ")"
PRINT "3. Horizontal (" + MStr$(S(3).X) + "x" + MStr$(S(3).Y) + ")"
PRINT "4. Giant (" + MStr$(S(4).X) + "x" + MStr$(S(4).Y) + ")"
PRINT "5. BOSS! (" + MStr$(S(5).X) + "x" + MStr$(S(5).Y) + ")"
PRINT ""
12 INPUT "Size"; Size
IF Size < 1 OR Size > 5 THEN
  PRINT "BAD SIZE!!"
  GOTO 12
END IF
SCREEN 13
PALETTE USING CPal
XCoord = 0
YCoord = 0
IF Size = 1 THEN
  PUT (0, 0), Image
  GET (0, 0)-(S(1).X - 1, S(1).Y - 1), Image
END IF
MakeEditScreen
END SUB

SUB ShowCols
FOR X = 0 TO 15
  FOR Y = 0 TO 15
    LINE (X * 3 + 212, Y * 3 + 61)-STEP(2, 2), (X * 16) + Y, BF
  NEXT Y
NEXT X
ColX = INT(Col / 16)
ColY = Col MOD 16
GET (ColX * 3 + 211, ColY * 3 + 60)-STEP(4, 4), ColBox
LINE (ColX * 3 + 211, ColY * 3 + 60)-STEP(4, 4), 15, B
LINE (211, 110)-STEP(15, 7), 0, BF
CALL Textfont(MStr$(Col), 211, 110, 7, 0)
END SUB

SUB SLCurs (Mode AS INTEGER)
SCREEN 0
WIDTH 80
COLOR 15
IF Mode GOTO 152
PRINT "* Save Cursor *"
PRINT "---------------"
COLOR 7
INPUT "Filename (no EXT): ", FileN$
IF LEN(FileN$) > 8 THEN FileN$ = LEFT$(FileN$, 8)
FileN$ = UCASE$(FileN$): FileN$ = FileN$ + ".CUR"
PRINT FileN$ + ": right? No"
Save2 = False
DO
A$ = INKEY$
SELECT CASE A$
  CASE "Y", "y"
    LOCATE 4, LEN(FileN$) + 10
    PRINT "Yes"
    Save2 = True
  CASE "N", "n"
    LOCATE 4, LEN(FileN$) + 10
    PRINT "No "
    Save2 = False
END SELECT
LOOP UNTIL A$ = CHR$(Enter)
IF Save2 THEN
  FileN$ = PATH$ + FileN$
  OPEN FileN$ FOR RANDOM AS #1 LEN = 2
  FOR X = 1 TO 65
    PUT #1, X, Cursor(X)
  NEXT X
  CLOSE #1
  PRINT "File Saved!"
  A$ = WaitKey$
END IF
GOTO 153
152
PRINT "* Load Cursor *"
PRINT "---------------"
COLOR 7
INPUT "Filename (no EXT): ", FileN$
IF LEN(FileN$) > 8 THEN FileN$ = LEFT$(FileN$, 8)
FileN$ = UCASE$(FileN$): FileN$ = FileN$ + ".CUR"
PRINT FileN$ + ": right? No"
Load2 = False
DO
A$ = INKEY$
SELECT CASE A$
  CASE "Y", "y"
    LOCATE 4, LEN(FileN$) + 10
    PRINT "Yes"
    Load2 = True
  CASE "N", "n"
    LOCATE 4, LEN(FileN$) + 10
    PRINT "No "
    Load2 = False
END SELECT
LOOP UNTIL A$ = CHR$(Enter)
IF Load2 THEN
  FileN$ = PATH$ + FileN$
  OPEN FileN$ FOR RANDOM AS #1 LEN = 2
  FOR X = 1 TO 65
    GET #1, X, Cursor(X)
  NEXT X
  CLOSE #1
  PRINT "File Loaded!"
  A$ = WaitKey$
END IF
153
SCREEN 13
PALETTE USING CPal
END SUB

SUB SLPal (Mode AS INTEGER)
SCREEN 0
WIDTH 80
COLOR 15
IF Mode THEN GOTO 151
PRINT "* Save Palette *"
PRINT "----------------"
COLOR 7
INPUT "Enter the filename(no EXT): ", FileN$
IF LEN(FileN$) > 8 THEN FileN$ = LEFT$(FileN$, 8)
FileN$ = UCASE$(FileN$)
FileN$ = FileN$ + ".PAL"
PRINT FileN$ + ": right? No"
Save2 = False
DO
A$ = INKEY$
SELECT CASE A$
  CASE "Y", "y"
    LOCATE 4, LEN(FileN$) + 10
    PRINT "Yes"
    Save2 = True
  CASE "N", "n"
    LOCATE 4, LEN(FileN$) + 10
    PRINT "No "
    Save2 = False
END SELECT
LOOP UNTIL A$ = CHR$(Enter)
IF Save2 THEN
  FileN$ = PATH$ + FileN$
  OPEN FileN$ FOR RANDOM AS #1 LEN = 4
  FOR C = 1 TO 256
    PUT #1, C, CPal(C)
  NEXT C
  CLOSE #1
  PRINT "File Saved!"
  A$ = WaitKey$
END IF
GOTO 154
151 PRINT "* Load Palette *"
PRINT "----------------"
COLOR 7
INPUT "Enter the filename(no EXT): ", FileN$
IF LEN(FileN$) > 8 THEN FileN$ = LEFT$(FileN$, 8)
FileN$ = UCASE$(FileN$)
FileN$ = FileN$ + ".PAL"
PRINT FileN$ + ": right? No"
Load2 = False
DO
A$ = INKEY$
SELECT CASE A$
  CASE "Y", "y"
    LOCATE 4, LEN(FileN$) + 10
    PRINT "Yes"
    Load2 = True
  CASE "N", "n"
    LOCATE 4, LEN(FileN$) + 10
    PRINT "No "
    Load2 = False
END SELECT
LOOP UNTIL A$ = CHR$(Enter)
IF Load2 THEN
  FileN$ = PATH$ + FileN$
  OPEN FileN$ FOR RANDOM AS #1 LEN = 4
  FOR C = 1 TO 256
    GET #1, C, CPal(C)
  NEXT C
  CLOSE #1
  PRINT "File Loaded!"
  A$ = WaitKey$
END IF
154 SCREEN 13
PALETTE USING CPal
CALL MakeEditScreen
END SUB

SUB SpecEdit (Mode AS INTEGER)
IF Mode <> 1 THEN GOTO 2
TN$ = LTRIM$(Ats.Special1.SpellName)
TD$ = LTRIM$(Ats.Special1.SpellDesc)
CL2% = 3
DO
CLS
COLOR 15
PRINT "Special Attack 1"
PRINT "================"
COLOR 7
PRINT "Name: " + TN$
PRINT "Description: " + TD$
PRINT "Effect:"; Ats.Special1.Effect
PRINT "Effect Strength:"; Ats.Special1.EStrength
PRINT "Cost in Magic Points:"; Ats.Special1.MPCost
PRINT "Cost in Hit Points:"; Ats.Special1.HPCost
LOCATE CL2%, 1
A$ = WaitKey$
SELECT CASE CL2%
  CASE 3, 4
    SELECT CASE A$
      CASE CHR$(32) TO CHR$(126)
        IF CL2% = 3 THEN TN$ = TN$ + A$
        IF CL2% = 4 THEN TD$ = TD$ + A$
      CASE CHR$(Backspace), CHR$(0) + CHR$(Left)
        IF CL2% = 3 THEN
          IF LEN(TN$) > 1 THEN TN$ = LEFT$(TN$, LEN(TN$) - 1) ELSE TN$ = ""
        END IF
        IF CL2% = 4 THEN
          IF LEN(TD$) > 1 THEN TD$ = LEFT$(TD$, LEN(TD$) - 1) ELSE TD$ = ""
        END IF
      CASE CHR$(0) + CHR$(Up)
        IF CL2% = 4 THEN CL2% = CL2% - 1
      CASE CHR$(0) + CHR$(Down)
        CL2% = CL2% + 1
      CASE CHR$(0) + CHR$(Right)
        IF CL2% = 3 THEN TN$ = TN$ + " "
        IF CL2% = 4 THEN TD$ = TD$ + " "
      END SELECT
  CASE 5 TO 8
    SELECT CASE A$
      CASE "1" TO "9"
        IF CL2% = 5 THEN Ats.Special1.Effect = Ats.Special1.Effect * 10 + ASC(A$) - 48
        IF CL2% = 6 THEN Ats.Special1.EStrength = Ats.Special1.EStrength * 10 + ASC(A$) - 48
        IF CL2% = 7 THEN Ats.Special1.MPCost = Ats.Special1.MPCost * 10 + ASC(A$) - 48
        IF CL2% = 8 THEN Ats.Special1.HPCost = Ats.Special1.HPCost * 10 + ASC(A$) - 48
      CASE "0"
        IF CL2% = 5 THEN Ats.Special1.Effect = Ats.Special1.Effect * 10
        IF CL2% = 6 THEN Ats.Special1.EStrength = Ats.Special1.EStrength * 10
        IF CL2% = 7 THEN Ats.Special1.MPCost = Ats.Special1.MPCost * 10
        IF CL2% = 8 THEN Ats.Special1.HPCost = Ats.Special1.HPCost * 10
      CASE CHR$(Backspace), CHR$(0) + CHR$(Left)
        IF CL2% = 5 THEN Ats.Special1.Effect = Ats.Special1.Effect \ 10
        IF CL2% = 6 THEN Ats.Special1.EStrength = Ats.Special1.EStrength \ 10
        IF CL2% = 7 THEN Ats.Special1.MPCost = Ats.Special1.MPCost \ 10
        IF CL2% = 8 THEN Ats.Special1.HPCost = Ats.Special1.HPCost \ 10
      CASE CHR$(0) + CHR$(Up)
        CL2% = CL2% - 1
      CASE CHR$(0) + CHR$(Down)
        IF CL2% < 8 THEN CL2% = CL2% + 1
    END SELECT
END SELECT
LOOP UNTIL A$ = CHR$(ESC)
Ats.Special1.SpellName = LTRIM$(TN$)
Ats.Special1.SpellDesc = LTRIM$(TD$)
GOTO 3
2
CL2% = 3
TN$ = LTRIM$(Ats.Special2.SpellName)
TD$ = LTRIM$(Ats.Special2.SpellDesc)
DO
CLS
COLOR 15
PRINT "Special Attack 2"
PRINT "================"
COLOR 7
PRINT "Name: "; LTRIM$(TN$)
PRINT "Description: "; LTRIM$(TD$)
PRINT "Effect:"; Ats.Special2.Effect
PRINT "Effect Strength:"; Ats.Special2.EStrength
PRINT "Cost in Magic Points:"; Ats.Special2.MPCost
PRINT "Cost in Hit Points:"; Ats.Special2.HPCost
LOCATE CL2%, 1
A$ = WaitKey$
SELECT CASE CL2%
  CASE 3, 4
    SELECT CASE A$
      CASE CHR$(32) TO CHR$(126)
        IF CL2% = 3 THEN TN$ = TN$ + A$
        IF CL2% = 4 THEN TD$ = TD$ + A$
      CASE CHR$(Backspace), CHR$(0) + CHR$(Left)
        IF CL2% = 3 THEN
          IF LEN(TN$) > 1 THEN TN$ = LEFT$(TN$, LEN(TN$) - 1) ELSE TN$ = ""
        END IF
        IF CL2% = 4 THEN
          IF LEN(TD$) > 1 THEN TD$ = LEFT$(TD$, LEN(TD$) - 1) ELSE TD$ = ""
        END IF
      CASE CHR$(0) + CHR$(Up)
        IF CL2% = 4 THEN CL2% = CL2% - 1
      CASE CHR$(0) + CHR$(Down)
        CL2% = CL2% + 1
      CASE CHR$(0) + CHR$(Right)
        IF CL2% = 3 THEN TN$ = TN$ + " "
        IF CL2% = 4 THEN TD$ = TD$ + " "
      END SELECT
  CASE 5 TO 8
    SELECT CASE A$
      CASE "1" TO "9"
        IF CL2% = 5 THEN Ats.Special2.Effect = Ats.Special2.Effect * 10 + ASC(A$) - 48
        IF CL2% = 6 THEN Ats.Special2.EStrength = Ats.Special2.EStrength * 10 + ASC(A$) - 48
        IF CL2% = 7 THEN Ats.Special2.MPCost = Ats.Special2.MPCost * 10 + ASC(A$) - 48
        IF CL2% = 8 THEN Ats.Special2.HPCost = Ats.Special2.HPCost * 10 + ASC(A$) - 48
      CASE "0"
        IF CL2% = 5 THEN Ats.Special2.Effect = Ats.Special2.Effect * 10
        IF CL2% = 6 THEN Ats.Special2.EStrength = Ats.Special2.EStrength * 10
        IF CL2% = 7 THEN Ats.Special2.MPCost = Ats.Special2.MPCost * 10
        IF CL2% = 8 THEN Ats.Special2.HPCost = Ats.Special2.HPCost * 10
      CASE CHR$(Backspace), CHR$(0) + CHR$(Left)
        IF CL2% = 5 THEN Ats.Special2.Effect = Ats.Special2.Effect \ 10
        IF CL2% = 6 THEN Ats.Special2.EStrength = Ats.Special2.EStrength \ 10
        IF CL2% = 7 THEN Ats.Special2.MPCost = Ats.Special2.MPCost \ 10
        IF CL2% = 8 THEN Ats.Special2.HPCost = Ats.Special2.HPCost \ 10
      CASE CHR$(0) + CHR$(Up)
        CL2% = CL2% - 1
      CASE CHR$(0) + CHR$(Down)
        IF CL2% < 8 THEN CL2% = CL2% + 1
    END SELECT
END SELECT
LOOP UNTIL A$ = CHR$(ESC)
Ats.Special2.SpellName = TN$
Ats.Special2.SpellDesc = TD$
3 CLS
END SUB

'SYNTAX: CALL TEXTFONT ("MESSAGE", X, Y, COLOR,Sound[y/n])
'X AND Y are coordinate locations
'
SUB Textfont (S$, X AS INTEGER, Y AS INTEGER, C AS INTEGER, So AS INTEGER)

PSET (-1, Y + 4), C
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

IF So = 1 AND UCASE$(MID$(S$, T, 1)) <> " " THEN SOUND 55, .1
IF So = 1 AND UCASE$(MID$(S$, T, 1)) <> " " THEN FOR M = 1 TO (1000 * TD): NEXT M

NEXT T
END SUB

FUNCTION WaitKey$
DO
A$ = INKEY$
LOOP UNTIL A$ <> ""
WaitKey$ = A$
END FUNCTION

SUB ZMove (X AS INTEGER, Y AS INTEGER, Z)
IF Z = 1 THEN
  IF X < (S(Size).X - 9) AND Y < (S(Size).Y - 9) AND X > -1 AND Y > -1 THEN
    SELECT CASE Size
      CASE 1
        PUT (XCoord + 201, YCoord + 157), ZBox
        PUT (X + 201, Y + 157), ZBox
      CASE 2 TO 5
        PUT (XCoord + 9, YCoord + 9), ZBox
        PUT (X + 9, Y + 9), ZBox
    END SELECT
    XCoord = X
    YCoord = Y
  END IF
END IF
IF Z = 2 THEN
  IF X < (S(Size).X - 19) AND Y < (S(Size).Y - 19) AND X > -1 AND Y > -1 THEN
    SELECT CASE Size
      CASE 1
        PUT (XCoord + 201, YCoord + 157), ZBox
        PUT (X + 201, Y + 157), ZBox
      CASE 2 TO 5
        PUT (XCoord + 9, YCoord + 9), ZBox
        PUT (X + 9, Y + 9), ZBox
    END SELECT
    XCoord = X
    YCoord = Y
  END IF
END IF
END SUB

SUB ZoomIn
LOCATE 25, 1
COLOR 15
PRINT "Zoom to(0/1/2):      ";
Z = 0
DO
A$ = INKEY$
LOCATE 25, 17
SELECT CASE A$
  CASE "0"
    PRINT "     ";
    Z = 0
  CASE "1"
    PRINT "10x10";
    Z = 1
  CASE "2"
    PRINT "20x20";
    Z = 2
END SELECT
LOOP UNTIL A$ = CHR$(Enter) OR A$ = CHR$(ESC)
IF A$ = CHR$(ESC) THEN Z = 0
IF Z = 0 THEN GOTO 13
IF Size > 1 AND Size < 6 THEN PUT (XCoord - Cursor(64) + 10, YCoord - Cursor(65) + 10), Cursor
Zoomed = True
SELECT CASE Z
  CASE 1
    LINE (300, 180)-STEP(11, 11), 15, B
    GET (300, 180)-STEP(11, 11), ZBox
    LINE (300, 180)-STEP(11, 11), 0, B
  CASE 2
    LINE (290, 170)-STEP(21, 21), 15, B
    GET (290, 170)-STEP(21, 21), ZBox
    LINE (290, 170)-STEP(21, 21), 0, B
END SELECT
XCoord = 0
YCoord = 0
SELECT CASE Size
  CASE 1
    PUT (201, 157), ZBox
  CASE 2 TO 5
    PUT (9, 9), ZBox
END SELECT
DO
  A$ = WaitKey$
  SELECT CASE A$
    CASE CHR$(0) + CHR$(Left)
      CALL ZMove(XCoord - 1, YCoord, Z)
    CASE CHR$(0) + CHR$(Right)
      CALL ZMove(XCoord + 1, YCoord, Z)
    CASE CHR$(0) + CHR$(Up)
      CALL ZMove(XCoord, YCoord - 1, Z)
    CASE CHR$(0) + CHR$(Down)
      CALL ZMove(XCoord, YCoord + 1, Z)
  END SELECT
LOOP UNTIL A$ = CHR$(Enter) OR A$ = CHR$(ESC)
ZX = XCoord
ZY = YCoord
SELECT CASE Size
  CASE 1
    FOR X = 1 TO 20
      FOR Y = 1 TO 20
        ZImage(X, Y) = CHR$(POINT(X + 201 + ZX, Y + 157 + ZY))
      NEXT Y
    NEXT X
  CASE 2 TO 5
    FOR X = 1 TO 20
      FOR Y = 1 TO 20
        ZImage(X, Y) = CHR$(POINT(X + 9 + ZX, Y + 9 + ZY))
      NEXT Y
    NEXT X
END SELECT
XCoord = 0
YCoord = 0
OSize = Size
SELECT CASE Z
  CASE 1
    Size = 6
  CASE 2
    Size = 7
END SELECT
13 CLS
CALL MakeEditScreen
END SUB

SUB ZoomOut
Zoomed = False
S = (Size - 5) * 10
Size = OSize
CLS
XCoord = 0
YCoord = 0
CALL MakeEditScreen
SELECT CASE Size
  CASE 2 TO 5
    PUT (XCoord - Cursor(64) + 10, YCoord - Cursor(65) + 10), Cursor
    FOR X = 1 TO S
      FOR Y = 1 TO S
        PSET (X + 9 + ZX, Y + 9 + ZY), ASC(ZImage(X, Y))
      NEXT Y
    NEXT X
    GET (10, 10)-STEP(S(Size).X - 1, S(Size).Y - 1), Image
    PUT (XCoord - Cursor(64) + 10, YCoord - Cursor(65) + 10), Cursor
  CASE 1
    FOR X = 1 TO S
      FOR Y = 1 TO S
        PSET (X + 201 + ZX, Y + 157 + ZY), ASC(ZImage(X, Y))
      NEXT Y
    NEXT X
    GET (202, 158)-STEP(S(1).X - 1, S(1).Y - 1), Image
END SELECT
END SUB

