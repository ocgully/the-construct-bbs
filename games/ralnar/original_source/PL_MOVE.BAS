DECLARE SUB SetScr ()
DECLARE SUB UpdatePic ()
DECLARE SUB CheckBoundry (Dir)
'$DYNAMIC
CONST Land = 1
CONST Water = 2
CONST Slow = 3
CONST Damaging = 4 'Damaging is slow as well
CONST Impassible = 5
CONST Town = 6
CONST Cave = 7
CONST Dock = 8
CONST AnyPass = 9
CONST Walk = 1
CONST Ship = 2
CONST Air = 3
CONST Right = 1
CONST Left = 2
CONST Up = 3
CONST Down = 4
CONST True = 1
CONST False = 0
CONST ScrPath$ = "C:\BG_RPG\SCREENS\"

COMMON SHARED PlayerLocX, PlayerLocY, CurrentDir, Mode, ExitScr
COMMON SHARED OPLX, OPLY, WorldY, WorldX

ExitScr = 0
DIM SHARED Pic(200, 180)
DIM SHARED BlockAttributes(10, 9)

SCREEN 13

OPLX = 8
OPLY = 8
PlayerLocX = 8
PlayerLocY = 8
CurrentDir = Right
Mode = Walk
WorldX = 19
WorldY = 15


CALL SetScr
CALL UpdatePic
DO
A$ = INKEY$
SELECT CASE A$
   
       CASE IS = CHR$(0) + "H"    'UP
         CALL CheckBoundry(Up)
         IF (CurrentDir <> Up) AND (ValidDir = True) THEN
             LOCATE 1, 25: PRINT "Change to Up   "
             CurrentDir = Up
         END IF
         CALL UpdatePic

      CASE IS = CHR$(0) + "P"    'DOWN
         CALL CheckBoundry(Down)
         IF (CurrentDir <> Down) AND (ValidDir = True) THEN
             LOCATE 1, 25: PRINT "Change to Down "
             CurrentDir = Down
         END IF
         CALL UpdatePic

      CASE IS = CHR$(0) + "K"    'LEFT
         CALL CheckBoundry(Left)
         IF (CurrentDir <> Left) AND (ValidDir = True) THEN
             LOCATE 1, 25: PRINT "Change to Left "
             CurrentDir = Left
         END IF
         CALL UpdatePic

      CASE IS = CHR$(0) + "M"    'RIGHT
         CALL CheckBoundry(Right)
         IF (CurrentDir <> Right) AND (ValidDir = True) THEN
             LOCATE 1, 25: PRINT "Change to Right"
             CurrentDir = Right
         END IF
         CALL UpdatePic
        
END SELECT

LOCATE 2, 26: PRINT "X: "; PlayerLocX; " Y: "; PlayerLocY
LOCATE 3, 30: PRINT BlockAttributes(PlayerLocX, PlayerLocY)
LOOP UNTIL UCASE$(A$) = "Q"


DEFSNG A-Z
'This Sub Checks to make sure the player can move to that location
'depending on the current mode of transportation(Walking, Ship, Air...)
'And where the player wants to move.
'
SUB CheckBoundry (Dir)
SHARED ValidDir

Valid = 0
ExitScr = 0
SELECT CASE Dir
   
      CASE IS = Up
          IF PlayerLocY - 1 > 0 THEN
          SELECT CASE Mode
              CASE IS = Walk
                  IF (BlockAttributes(PlayerLocX, PlayerLocY - 1) = Land) OR (BlockAttributes(PlayerLocX, PlayerLocY - 1) = AnyPass) THEN
                     OPLY = PlayerLocY: OPLX = PlayerLocX
                     PlayerLocY = PlayerLocY - 1
                     Valid = 1
                     ELSE Valid = 0
                  END IF

              CASE IS = Ship
                  IF BlockAttributes(PlayerLocX, PlayerLocY - 1) = Water THEN
                     OPLY = PlayerLocY: OPLX = PlayerLocX
                     PlayerLocY = PlayerLocY - 1
                     Valid = 1
                     ELSE Valid = 0
                  END IF
                  
              CASE IS = Air
                  OPLY = PlayerLocY: OPLX = PlayerLocX
                  PlayerLocY = PlayerLocY - 1
                  Valid = 1
          END SELECT 'Mode

          ELSE ExitScr = Up
          END IF

      CASE IS = Down
          IF PlayerLocY + 1 < 10 THEN
          SELECT CASE Mode
              CASE IS = Walk
                  IF BlockAttributes(PlayerLocX, PlayerLocY + 1) = Land OR (BlockAttributes(PlayerLocX, PlayerLocY + 1) = AnyPass) THEN
                     OPLY = PlayerLocY: OPLX = PlayerLocX
                     PlayerLocY = PlayerLocY + 1
                     Valid = 1
                     ELSE Valid = 0
                  END IF

              CASE IS = Ship
                  IF BlockAttributes(PlayerLocX, PlayerLocY + 1) = Water THEN
                     OPLY = PlayerLocY: OPLX = PlayerLocX
                     PlayerLocY = PlayerLocY + 1
                     Valid = 1
                     ELSE Valid = 0
                  END IF
                 
              CASE IS = Air
                  OPLY = PlayerLocY: OPLX = PlayerLocX
                  PlayerLocY = PlayerLocY + 1
                  Valid = 1
          END SELECT 'Mode

          ELSE ExitScr = Down
          END IF
     
      CASE IS = Right
          IF PlayerLocX + 1 < 11 THEN
          SELECT CASE Mode
              CASE IS = Walk
                  IF BlockAttributes(PlayerLocX + 1, PlayerLocY) = Land OR (BlockAttributes(PlayerLocX + 1, PlayerLocY) = AnyPass) THEN
                     OPLX = PlayerLocX: OPLY = PlayerLocY
                     PlayerLocX = PlayerLocX + 1
                     Valid = 1
                     ELSE Valid = 0
                  END IF

              CASE IS = Ship
                  IF BlockAttributes(PlayerLocX + 1, PlayerLocY) = Water THEN
                     OPLX = PlayerLocX
                     OPLY = PlayerLocY
                     PlayerLocX = PlayerLocX + 1
                     Valid = 1
                     ELSE Valid = 0
                  END IF
                
              CASE IS = Air
                  OPLX = PlayerLocX: OPLY = PlayerLocY
                  PlayerLocX = PlayerLocX + 1
                  Valid = 1
          END SELECT 'Mode

          ELSE ExitScr = Right
          END IF

      CASE IS = Left
          IF PlayerLocX - 1 > 0 THEN
          SELECT CASE Mode
              CASE IS = Walk
                  IF BlockAttributes(PlayerLocX - 1, PlayerLocY) = Land OR (BlockAttributes(PlayerLocX - 1, PlayerLocY) = AnyPass) THEN
                     OPLX = PlayerLocX: OPLY = PlayerLocY
                     PlayerLocX = PlayerLocX - 1
                     Valid = 1
                     ELSE Valid = 0
                  END IF

              CASE IS = Ship
                  IF BlockAttributes(PlayerLocX - 1, PlayerLocY) = Water THEN
                     OPLX = PlayerLocX: OPLY = PlayerLocY
                     PlayerLocX = PlayerLocX - 1
                     Valid = 1
                     ELSE Valid = 0
                  END IF
               
              CASE IS = Air
                  OPLX = PlayerLocX: OPLY = PlayerLocY
                  PlayerLocX = PlayerLocX - 1
                   Valid = 1
          END SELECT 'Mode

          ELSE ExitScr = Left
          END IF

END SELECT

SELECT CASE ExitScr
     CASE IS = Left
        WorldX = WorldX - 1
        PlayerLocX = 10
        OPLX = 10

     CASE IS = Right
       WorldX = WorldX + 1
       PlayerLocX = 1
       OPLX = 1

     CASE IS = Up
       WorldY = WorldY - 1
       PlayerLocY = 9
       OPLY = 9

     CASE IS = Down
       WorldY = WorldY + 1
       PlayerLocY = 1
       OPLY = 1

END SELECT

IF ExitScr <> 0 THEN CALL SetScr


IF (Valid = 0) AND (ExitScr = 0) THEN BEEP
ValidDir = Valid
END SUB

SUB SetScr
'This Sub Loads and Draws the Screen depending on the location
'And Also Loads the Screens Attributes
'
'
LINE (1, 1)-(200, 180), 0, BF
X$ = STR$(WorldX)
X$ = MID$(X$, 2, LEN(X$) - 1)
Y$ = STR$(WorldY)
Y$ = MID$(Y$, 2, LEN(Y$) - 1)

ScrName$ = "MX" + X$ + "Y" + Y$ + ".SCR"


OPEN ScrPath$ + ScrName$ FOR INPUT AS #1
FOR X = 1 TO 200
  FOR Y = 1 TO 180
    INPUT #1,Pic(X, Y)
    PSET (X, Y), Pic(X, Y)
  NEXT Y
NEXT X
  
FOR X = 1 TO 10
  FOR Y = 1 TO 9
    INPUT #1,BlockAttributes(X, Y)
  NEXT Y
NEXT X


CLOSE #1

END SUB

SUB UpdatePic
'This Sub Updates the Icon moving from location to location
'
'
'
'

FOR X = ((OPLX - 1) * 20) + 1 TO ((OPLX - 1) * 20) + 20
  FOR Y = ((OPLY - 1) * 20) + 1 TO ((OPLY - 1) * 20) + 20
    PSET (X, Y), Pic(X, Y)
  NEXT Y
NEXT X

FOR X = ((PlayerLocX - 1) * 20) + 1 TO ((PlayerLocX - 1) * 20) + 20
  FOR Y = ((PlayerLocY - 1) * 20) + 1 TO ((PlayerLocY - 1) * 20) + 20
    PSET (X, Y), 0
  NEXT Y
NEXT X

SELECT CASE CurrentDir 'Draws Arrow to show direction

      CASE IS = Right
         PSET ((PlayerLocX - 1) * 20 + 9, (PlayerLocY - 1) * 20 + 10), 15
         DRAW "r3hfg"
     
      CASE IS = Left
         PSET ((PlayerLocX - 1) * 20 + 11, (PlayerLocY - 1) * 20 + 10), 15
         DRAW "l3egf"
     
      CASE IS = Up
         PSET ((PlayerLocX - 1) * 20 + 10, (PlayerLocY - 1) * 20 + 11), 15
         DRAW "u3gef"

      CASE IS = Down
         PSET ((PlayerLocX - 1) * 20 + 10, (PlayerLocY - 1) * 20 + 9), 15
         DRAW "d3egh"



END SELECT
'FOR X = PlayerLocX - 9 TO PlayerLocX + 10
'  FOR Y = PlayerLocY - 9 TO PlayerLocY + 10
'    PSET (X, Y), 0
'  NEXT Y
'NEXT X
'PSET (PlayerLocX, PlayerLocY), 15
END SUB