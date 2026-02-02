'$DYNAMIC
DECLARE SUB NewIcon ()
TYPE ICONTYPE
  Number AS INTEGER
  Attrib AS INTEGER
END TYPE
COMMON SHARED ScreenX, ScreenY, NewIcons, IconAttrib, TotalIcons
OPTION BASE 1
DIM Pic(10, 9)AS ICONTYPE
DIM Scr(200, 180) AS INTEGER
DIM ScrAttrib(10, 9) AS INTEGER
DIM SHARED Icons(100) AS STRING
CONST True = 1
CONST False = 0
DIM SHARED TempIcon(20, 20) AS INTEGER

ScreenName$ = "Test.Scr"
CONST ScreenPath$ = "C:\BG_RPG\BASIC\SCREENS\" 'Should already be a constant
CONST PicPath$ = "C:\BG_RPG\BASIC\PICS\"
NewIcons = 0

SCREEN 13
LOCATE 23, 1: PRINT "Loading..."
OPEN ScreenPath$ + ScreenName$ FOR INPUT AS #1
FOR X = 1 TO 200
  FOR Y = 1 TO 180
    INPUT #1,Scr(X, Y)
  NEXT Y
NEXT X

FOR X = 1 TO 10
  FOR Y = 1 TO 9
    INPUT #1,ScrAttrib(X, Y)
  NEXT Y
NEXT X
CLS
CLOSE #1

OPEN PicPath$ + "Icons.Txt" FOR INPUT AS #1
T = 0
WHILE NOT EOF(1)
  T = T + 1
  INPUT #1,Icons(T)
WEND
TotalIcons = T
CLOSE #1
LOCATE 5, 30: PRINT TotalIcons
'END
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
    DO
      T = T + 1
      Valid = True
      LOCATE 3, 30: PRINT T
      LOCATE 5, 30: PRINT TotalIcons
      OPEN PicPath$ + Icons(T) FOR INPUT AS #2
      FOR X = 1 TO 20
        FOR Y = 1 TO 20
          INPUT #2,TempIconNum
          IF TempIconNum <> TempIcon(X, Y) THEN Valid = False
        NEXT Y
      NEXT X
      INPUT #2,IconAttrib
      CLOSE #2
      IF Valid = True THEN
        Pic(ScreenX, ScreenY).Number = T
        Pic(ScreenX, ScreenY).Attrib = IconAttrib
        IF Valid = True THEN IconFound = True
      END IF
    LOOP UNTIL (T = TotalIcons) OR (IconFound = True)
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
DO
 Valid = True
 LOCATE 23, 1: INPUT "Enter New Name (8 Char): ", Name$
 LOCATE 23, 1: PRINT "                               "
 Name$ = Name$ + ".PIC"
 IF LEN(Name$) <= 12 THEN
    FOR T = 1 TO TotalIcons
      IF UCASE$(Name$) = UCASE$(Icons(T)) THEN Valid = False
    NEXT T
 ELSE IF LEN(Name$) > 12 THEN Valid = False
 END IF
 IF Valid = False THEN
   LOCATE 23, 1: INPUT "Filename already exists...", Con$
   LOCATE 23, 1: PRINT "                               "
 END IF
LOOP UNTIL Valid = True
TotalIcons = TotalIcons + 1
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

Icons(TotalIcons) = Name$

END SUB