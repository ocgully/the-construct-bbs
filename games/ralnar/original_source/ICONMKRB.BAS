DECLARE SUB UPDATEIconPIC (XCorr!, YCorr!)
DECLARE SUB UPDATEIconBOX (XCorr!, YCorr!)
DECLARE SUB Attributes ()
DECLARE SUB PrintAttrib ()
DECLARE SUB ClearIconPic (Initial!)
DECLARE SUB LoadIcon ()
DECLARE SUB SaveIcon ()
DECLARE SUB IconHelp ()
DECLARE SUB Textfont (S$, X!, Y!, C!, S!)
DECLARE SUB SelectCol ()
DECLARE SUB MAKEPICBOX ()
DECLARE SUB MAKEGRID ()
COMMON SHARED OXCorr, XCorr, OYCorr, YCorr, CHANGE.CORR, Attrib$, IconAttribute, IconDone


'ICON MAKER
'By Brian Gulliver
'20 x 20
SCREEN 13

CONST XDIM = 20
CONST YDIM = 20
CONST PATH$ = "C:\BG_RPG\BASIC\PICS\"  'Where Icons are located
CONST Land = 1
CONST Water = 2
CONST Slow = 3
CONST Damaging = 4 'Damaging is slow as well
CONST Impassible = 5
CONST Town = 6
CONST Cave = 7
CONST Dock = 8
CONST AnyPass = 9
CONST True = 1
CONST False = 0
IconAttribute = 1

DIM SHARED Icon(XDIM, YDIM)
DIM SHARED Col AS INTEGER
KLOCK = False
IconDone = False
'NAME$ = ""
CALL Textfont("Lock OFF", 210, 170, 7, 0)
CALL ClearIconPic(0) 'Initializes Everything
CALL PrintAttrib
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
                    CALL IconHelp
                      ELSEIF UCASE$(A$) = "S" THEN
                        CALL SaveIcon
                          ELSEIF UCASE$(A$) = "O" THEN
                            CALL LoadIcon
                              ELSEIF UCASE$(A$) = "G" THEN
                                Col = Icon(XCorr, YCorr)
                                CALL SelectCol
                                 ELSEIF UCASE$(A$) = "A" THEN
                                   CALL Attributes
                                  ELSEIF UCASE$(A$) = "E" THEN
                                    CALL ClearIconPic(1)
                                      ELSEIF UCASE$(A$) = "L" THEN
                                        IF KLOCK = 1 THEN KLOCK = 0 ELSE KLOCK = 1
                                        LINE (205, 165)-(280, 175), 0, BF
                                        IF KLOCK = 1 THEN
                                              CALL Textfont("Lock ON", 210, 170, 7, 0)
                                           ELSE CALL Textfont("Lock OFF", 210, 170, 7, 0)
                                        END IF

                                    A$ = " "
                                  
                                  ELSEIF UCASE$(A$) = "`" THEN
                                    Icon(XCorr, YCorr) = -1
                                    CALL UPDATEIconPIC(XCorr, YCorr)
                                    CALL UPDATEIconBOX(XCorr, YCorr)
END IF
IF (OXCorr <> XCorr) OR (OYCorr <> YCorr) THEN CHANGE.CORR = 1 ELSE CHANGE.CORR = 0
LOCATE 20, 26: PRINT "("; XCorr; ","; YCorr; ")    "

IF (A$ = " ") THEN
    Icon(XCorr, YCorr) = Col
    CALL UPDATEIconPIC(XCorr, YCorr)
    CALL UPDATEIconBOX(XCorr, YCorr)
END IF

IF (CHANGE.CORR = 1) AND (KLOCK = 1) THEN
   Icon(XCorr, YCorr) = Col
   CALL UPDATEIconPIC(XCorr, YCorr)
   CALL UPDATEIconBOX(XCorr, YCorr)
END IF


LOOP UNTIL UCASE$(A$) = "Q" OR IconDone = True

               


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
CALL MAKEGRID
CALL Textfont("(C)hange color", 10, 188, 15, 0)
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)

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
  PATHNAME$ = PATH$ + Name$
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
  IF Icon(X, Y) <> -1 THEN
  LINE (SHIFTX + 1 + ((X - 1) * 9), SHIFTY + 1 + ((Y - 1) * 9))-(SHIFTX + 8 + ((X - 1) * 9), 1 + SHIFTY + 7 + ((Y - 1) * 9)), Icon(X, Y), BF
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
   IF Icon(X, Y) <> -1 THEN
      PSET (SHIFTX + X, SHIFTY + Y), Icon(X, Y)
        ELSE
          PSET (SHIFTX + X, SHIFTY + Y), 0
   END IF
  NEXT X
NEXT Y
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
PATHNAME$ = PATH$ + Name$
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
CALL Textfont("(Q)uit", 10, 195, 15, 0)
CALL Textfont("(H)elp", 120, 188, 15, 0)

END SUB

SUB UPDATEIconBOX (XCorr, YCorr)
SHIFTX = 290
SHIFTY = 170
   IF Icon(XCorr, YCorr) <> -1 THEN
      PSET (SHIFTX + XCorr, SHIFTY + YCorr), Icon(XCorr, YCorr)
        ELSE
          PSET (SHIFTX + XCorr, SHIFTY + YCorr), 0
   END IF

END SUB

SUB UPDATEIconPIC (XCorr, YCorr)
SHIFTY = 1
SHIFTX = 15
  IF Icon(XCorr, YCorr) <> -1 THEN
  LINE (SHIFTX + 1 + ((XCorr - 1) * 9), SHIFTY + 1 + ((YCorr - 1) * 9))-(SHIFTX + 8 + ((XCorr - 1) * 9), 1 + SHIFTY + 7 + ((YCorr - 1) * 9)), Icon(XCorr, YCorr), BF
  ELSE
  LINE (SHIFTX + 1 + ((XCorr - 1) * 9), SHIFTY + 1 + ((YCorr - 1) * 9))-(SHIFTX + 8 + ((XCorr - 1) * 9), 1 + SHIFTY + 7 + ((YCorr - 1) * 9)), 0, BF
  PSET (SHIFTX + 1 + ((XCorr - 1) * 9 + 4), SHIFTY + 1 + ((YCorr - 1) * 9) + 4), 7
  END IF

END SUB