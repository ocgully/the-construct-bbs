DECLARE SUB ClearPic (Initial!)
DECLARE SUB Load ()
DECLARE SUB Save ()
DECLARE SUB Help ()
DECLARE SUB Textfont (S$, X!, Y!, C!, S!)
DECLARE SUB SelectCol ()
DECLARE SUB MAKEPICBOX ()
DECLARE SUB MAKEGRID ()
COMMON SHARED OXCorr, XCorr, OYCorr, YCorr, CHANGE.CORR

'ICON MAKER
'By Brian Gulliver
'10 x 16
SCREEN 13

CONST XDIM = 20
CONST YDIM = 20
CONST PATH$ = "C:\BG_RPG\BASIC\PICS\"

DIM SHARED ICON(XDIM, YDIM)
DIM SHARED Col AS INTEGER
'NAME$ = ""
CALL ClearPic(0) 'Initializes Everything

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
                  ELSEIF UCASE$(A$) = "H" THEN
                    CALL Help
                      ELSEIF UCASE$(A$) = "S" THEN
                        CALL Save
                          ELSEIF UCASE$(A$) = "L" THEN
                            CALL Load
                              ELSEIF UCASE$(A$) = "E" THEN
                                CALL ClearPic(1)
                                  ELSEIF UCASE$(A$) = "`" THEN
                                    ICON(XCorr, YCorr) = -1
                                    CALL MAKEGRID
                                    CALL MAKEPICBOX
                     
END IF
IF (OXCorr <> XCorr) OR (OYCorr <> YCorr) THEN CHANGE.CORR = 1 ELSE CHANGE.CORR = 0
LOCATE 20, 26: PRINT "("; XCorr; ","; YCorr; ")    "
IF A$ = " " THEN
    ICON(XCorr, YCorr) = Col
    CALL MAKEGRID
    CALL MAKEPICBOX
END IF

LOOP UNTIL UCASE$(A$) = "Q"

               


DEFSNG A-Z
SUB ClearPic (Initial)
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
   IF YesNo$ = "Y" THEN Initial = 0
END IF
IF Initial = 0 THEN

FOR X = 1 TO XDIM        ' This sets all the dimensions to clear
  FOR Y = 1 TO YDIM      '
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

END SUB

SUB Help
LINE (1, 183)-(200, 200), 0, BF
LINE (15, 1)-(195, 181), 7, B
LINE (16, 2)-(194, 180), 0, BF
CALL Textfont("* Help *", 83, 20, 7, 0)
CALL Textfont("Just press the keys", 50, 50, 7, 0)
CALL Textfont("and draw a picture!", 50, 60, 7, 0)

DO
LOOP UNTIL INKEY$ <> ""
CALL MAKEGRID
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)

END SUB

SUB Load
LINE (1, 183)-(200, 200), 0, BF
LINE (15, 1)-(195, 181), 7, B
LINE (16, 2)-(194, 180), 0, BF
CALL Textfont("* LOAD *", 83, 20, 7, 0)
CALL Textfont("Enter the file name:", 25, 50, 7, 0)
X = 125
NAME$ = ""
DO
   S$ = INKEY$
   IF (X < 165) AND (S$ <> "") AND (S$ <> CHR$(13)) AND (((S$ > "/") AND (S$ < ":")) OR ((S$ > "@") AND (S$ < "[")) OR ((S$ > CHR$(96)) AND (S$ < "{"))) THEN
      X = X + 5
      CALL Textfont(S$, X, 50, 7, 0)
      NAME$ = NAME$ + S$
   END IF
   IF (S$ = CHR$(8)) AND (NAME$ <> "") THEN
      CALL Textfont("~", X, 50, 0, 0)
      X = X - 5
      NAME$ = MID$(NAME$, 1, LEN(NAME$) - 1)
   END IF
  
LOOP UNTIL S$ = CHR$(13)
IF NAME$ <> "" THEN
  NAME$ = NAME$ + ".PIC"

  CALL Textfont("Loading file: " + NAME$, 25, 60, 7, 0)
  CALL Textfont("Is this Correct? (Y/N): ", 25, 80, 7, 0)

  DO
  YesNo$ = INKEY$
   IF UCASE$(YesNo$) = "Y" THEN
     CALL Textfont("Y", 140, 80, 7, 1)
       ELSEIF UCASE$(YesNo$) = "N" THEN
         CALL Textfont("N", 140, 80, 7, 1)
   END IF
  LOOP UNTIL (UCASE$(YesNo$) = "Y") OR (UCASE$(YesNo$) = "N")
  PATHNAME$ = PATH$ + NAME$
  IF UCASE$(YesNo$) = "Y" THEN
    OPEN PATHNAME$ FOR INPUT AS #1
    FOR X = 1 TO XDIM
      FOR Y = 1 TO YDIM
        INPUT #1,ICON(X, Y)
      NEXT Y
    NEXT X
  CLOSE #1
  END IF


WHILE INKEY$ = "": WEND
END IF
CALL MAKEGRID
CALL MAKEPICBOX
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)

END SUB

SUB MAKEGRID
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
  IF ICON(X, Y) <> -1 THEN
  LINE (SHIFTX + 1 + ((X - 1) * 9), SHIFTY + 1 + ((Y - 1) * 9))-(SHIFTX + 8 + ((X - 1) * 9), 1 + SHIFTY + 7 + ((Y - 1) * 9)), ICON(X, Y), BF
  ELSE
  LINE (SHIFTX + 1 + ((X - 1) * 9), SHIFTY + 1 + ((Y - 1) * 9))-(SHIFTX + 8 + ((X - 1) * 9), 1 + SHIFTY + 7 + ((Y - 1) * 9)), 0, BF
  PSET (SHIFTX + 1 + ((X - 1) * 9 + 4), SHIFTY + 1 + ((Y - 1) * 9) + 4), 7
  END IF
 NEXT X
NEXT Y
END SUB

SUB MAKEPICBOX
SHIFTX = 290
SHIFTY = 170
FOR Y = 1 TO YDIM
  FOR X = 1 TO XDIM
   IF ICON(X, Y) <> -1 THEN
      PSET (SHIFTX + X, SHIFTY + Y), ICON(X, Y)
        ELSE
          PSET (SHIFTX + X, SHIFTY + Y), 0
   END IF
  NEXT X
NEXT Y
END SUB

SUB Save
LINE (1, 183)-(200, 200), 0, BF
LINE (15, 1)-(195, 181), 7, B
LINE (16, 2)-(194, 180), 0, BF
CALL Textfont("* SAVE *", 83, 20, 7, 0)
CALL Textfont("Enter the file name:", 25, 50, 7, 0)
X = 125
NAME$ = ""
DO
   S$ = INKEY$
   IF (X < 165) AND (S$ <> "") AND (S$ <> CHR$(13)) AND (((S$ > "/") AND (S$ < ":")) OR ((S$ > "@") AND (S$ < "[")) OR ((S$ > CHR$(96)) AND (S$ < "{"))) THEN
      X = X + 5
      CALL Textfont(S$, X, 50, 7, 0)
      NAME$ = NAME$ + S$
   END IF
   IF (S$ = CHR$(8)) AND (NAME$ <> "") THEN
      CALL Textfont("~", X, 50, 0, 0)
      X = X - 5
      NAME$ = MID$(NAME$, 1, LEN(NAME$) - 1)
   END IF
   
LOOP UNTIL S$ = CHR$(13)
IF NAME$ <> "" THEN
  NAME$ = NAME$ + ".PIC"

  CALL Textfont("Saving file as: " + NAME$, 25, 60, 7, 0)
  CALL Textfont("Is this Correct? (Y/N): ", 25, 80, 7, 0)

  DO
  YesNo$ = INKEY$
   IF UCASE$(YesNo$) = "Y" THEN
     CALL Textfont("Y", 140, 80, 7, 1)
       ELSEIF UCASE$(YesNo$) = "N" THEN
         CALL Textfont("N", 140, 80, 7, 1)
   END IF
  LOOP UNTIL (UCASE$(YesNo$) = "Y") OR (UCASE$(YesNo$) = "N")
PATHNAME$ = PATH$ + NAME$
  IF UCASE$(YesNo$) = "Y" THEN
    OPEN PATHNAME$ FOR OUTPUT AS #1
    FOR X = 1 TO XDIM
      FOR Y = 1 TO YDIM
        WRITE #1, ICON(X, Y)
      NEXT Y
    NEXT X
   
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
     LINE (300, 60 - (10 * Col))-(305, 60 - (10 * Col) + 5), 0, BF
       ELSE
         LINE (300, Y * 10)-(305, Y * 10 + 5), T, BF
  END IF
NEXT T
IF B$ = CHR$(0) + "P" AND Col <> 176 THEN
   Col = Col + 1
     ELSE IF B$ = CHR$(0) + "H" AND Col <> 0 THEN Col = Col - 1
END IF
LINE (244, 14)-(276, 36), 15, B
LINE (245, 15)-(275, 35), Col, BF
LOCATE 1, 31: PRINT Col



LOOP UNTIL UCASE$(B$) = "Q" OR UCASE$(B$) = " " OR UCASE$(B$) = "H"
IF UCASE$(B$) = "Q" THEN
   END
     ELSEIF UCASE$(B$) = "H" THEN
       CALL Help
END IF

PSET (285, 72): DRAW "c0;r5h2f2g2"
LINE (1, 183)-(200, 200), 0, BF
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)

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