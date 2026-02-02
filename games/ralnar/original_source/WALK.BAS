DECLARE SUB Textfont (S$, X!, Y!, C!, S!)
DECLARE SUB PrintAttrib (IM AS INTEGER)
DECLARE SUB DispScreen ()
CONST False = 0
CONST True = 1
CLS
NWorldX = WorldX: NWorldY = WorldY
Trigger = False
CurrentDir = Down
Mode = 3
CALL DispScreen
GOSUB ShowArrow
GOSUB PrintMode
CALL PrintAttrib(False)
DO
LINE (5 * 20, 4 * 20)-STEP(21, 21), 15, B
B$ = INKEY$
SELECT CASE B$

     
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


CLS
CALL DispScreen

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



DEFSNG A-Z
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

SUB Walk
CLS
NWorldX = WorldX: NWorldY = WorldY
Trigger = False
CurrentDir = Down
Mode = 3
CALL DispScreen
GOSUB ShowArrow
GOSUB PrintMode
CALL PrintAttrib(False)
DO
LINE (5 * 20, 4 * 20)-STEP(21, 21), 15, B
B$ = INKEY$
SELECT CASE B$

    
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


CLS
CALL DispScreen

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