'$DYNAMIC
DECLARE SUB NewIcon ()
TYPE ICONTYPE
  Number AS INTEGER
  Attrib AS INTEGER
END TYPE
COMMON SHARED ScreenX, ScreenY, NewIcons, IconAttrib
OPTION BASE 1
DIM Pic(10, 9)AS ICONTYPE
DIM Scr(200, 180) AS INTEGER
DIM ScrAttrib(10, 9)AS INTEGER
CONST True = 1
CONST False = 0
DIM SHARED TempIcon(20, 20)
ScreenName$ = "Test.Scr"
ScreenPath$ = "C:\BG_RPG\BASIC\SCREENS\" 'Should already be a constant
PicPath$ = "C:\BG_RPG\BASIC\PICS\"
NewIcons = 0

SCREEN 13
LOCATE 23, 1: PRINT "Loading..."
OPEN ScreenPath$ + ScreenName$ FOR INPUT AS #1
FOR X = 1 TO 200
  FOR Y = 1 TO 180
    INPUT #1,Scr(X, Y)
  NEXT Y
NEXT X
'FOR X = 1 TO 10
'  FOR Y = 1 TO 9
'    INPUT #1, ScrAttrib(X, Y)
'  NEXT Y
'NEXT X
CLS
CLOSE #1
END
FOR ScreenX = 1 TO 10
  FOR ScreenY = 1 TO 9
    IconFound = False
    T = 0
    FOR X = 1 TO 20
      FOR Y = 1 TO 20
        TempIcon(X, Y) = Scr((ScreenX - 1) * 20 + X, (ScreenY - 1) * 20 + Y)
        PSET ((ScreenX - 1) * 20 + X, (ScreenY - 1) * 20 + Y), TempIcon(X, Y)
      NEXT Y
    NEXT X
    OPEN PicPath$ + "Icons.Txt" FOR INPUT AS #2
    DO
      Valid = True
      INPUT #2,IconName$
      T = T + 1
      LOCATE 3, 30: PRINT T
      OPEN PicPath$ + IconName$ FOR INPUT AS #3
      FOR X = 1 TO 20
        FOR Y = 1 TO 20
          INPUT #3,TempIconNum
          IF TempIconNum <> TempIcon(X, Y) THEN Valid = False
        NEXT Y
      NEXT X
      INPUT #3,IconAttrib
      CLOSE #3
      IF Valid = True THEN
        Pic(ScreenX, ScreenY).Number = T
        Pic(ScreenX, ScreenY).Attrib = IconAttrib
        IF Valid = True THEN IconFound = True
      END IF
    LOOP UNTIL EOF(2) OR (IconFound = True)
    CLOSE #2
    IF IconFound = False THEN CALL NewIcon
  NEXT ScreenY
NEXT ScreenX
CLOSE #1

        
      


DEFSNG A-Z
SUB NewIcon
BEEP
FOR X = 1 TO 20
  FOR Y = 1 TO 20
    PSET (280 + X, 170 + Y), TempIcon(X, Y)
  NEXT Y
NEXT X

NewIcons = NewIcons + 1
LOCATE 1, 25: PRINT "NEW ICON # "; NewIcons
LOCATE 23, 1: INPUT "Enter New Name: ", Name$
LOCATE 23, 1: PRINT "                               "
FOR X = 1 TO 20
  FOR Y = 1 TO 20
    PSET (280 + X, 170 + Y), 0
  NEXT Y
NEXT X

OPEN PicPath$ + "Icons.Txt" FOR APPEND AS #2
  WRITE #2, Name$
CLOSE #2

OPEN PicPath$ + Name$ FOR OUTPUT AS #2
  FOR X = 1 TO 20
    FOR Y = 1 TO 20
      WRITE #2, TempIcon(X, Y)
    NEXT Y
  NEXT X
  WRITE #2, IconAttrib
CLOSE #2

END SUB