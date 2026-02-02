SCREEN 13: CLS
DIM ICON(20, 20) AS INTEGER
DIM ICON2(20, 20) AS INTEGER
OPEN "C:\BG_RPG\BASIC\PICS\cair1.Pic" FOR INPUT AS #1
OPEN "C:\BG_RPG\BASIC\PICS\cair3.Pic" FOR INPUT AS #3
OPEN "C:\BG_RPG\BASIC\PICS\cair2.Pic" FOR INPUT AS #2
'OPEN "C:\BG_RPG\BASIC\PICS\cair3.Pic" FOR INPUT AS #3

X = 0
y = 1

'COLOR 7
DO WHILE NOT EOF(1)
     X = X + 1
       IF X = 21 THEN
          X = 1
          y = y + 1
       END IF
     INPUT #1,ICON(y, X)
LOOP

X = 0: y = 1
DO WHILE NOT EOF(2)
     X = X + 1
       IF X = 21 THEN
          X = 1
          y = y + 1
       END IF
     INPUT #2,ICON2(y, X)
LOOP

X = 0: y = 1
DO WHILE NOT EOF(3)
     X = X + 1
       IF X = 21 THEN
          X = 1
          y = y + 1
       END IF
     INPUT #3,ICON2(y, X)
LOOP


FOR T = 1 TO 20

FOR X = 1 TO 20
  FOR y = 1 TO 20
    PSET (y, X), ICON(y, X)
  NEXT y
NEXT X
GOSUB PAUS:
FOR X = 1 TO 20
  FOR y = 1 TO 20
    PSET (y, X), ICON2(y, X)
  NEXT y
NEXT X
GOSUB PAUS:
NEXT T


END
GET (1, 1)-(20, 20), ICON
CLS
FOR X = 1 TO 20 STEP 20
  FOR y = 1 TO 20 STEP 20
    PUT (X, y), ICON, OR
  NEXT y
NEXT X

END
PAUS:
  FOR M = 1 TO 25000: NEXT M
RETURN