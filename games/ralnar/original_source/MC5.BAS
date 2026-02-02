'$DYNAMIC
CONST PATH$ = "C:\BG_RPG\Maps\"  'Screen Path
CONST NPATH$ = "C:\BG_RPG\Maps\" '.NMF Path
CONST MMIPath$ = "C:\BG_RPG\MMI\"
CONST MaxIcons = 410
CONST MaxIconsPerScreen = 350
CONST True = 1
CONST False = 0
CLS
DEFINT A-Z
DIM Icons(MaxIcons) AS STRING
OPEN MMIPath$ + "MMIFILES.TXT" FOR INPUT AS #1
  WHILE NOT EOF(1)
    TotalIcons = TotalIcons + 1
    INPUT #1,Icons(TotalIcons)   'Loads a List of the MMI filenames
  WEND
CLOSE #1
DIM NINums(TotalIcons) AS INTEGER
DIM Map(1, 1, 1) AS INTEGER
DIM MapIcons(203, MaxIconsPerScreen) AS INTEGER
1
CLS
INPUT "Which file do you want to convert? ", Nm$
INPUT "What do you want the half see size to be (in # of Pixels): ", HSize
IF (HSize > 20) OR (HSize < 0) THEN HSize = 10
INPUT "What do you want the full see size to be (in # of Pixels): ", FSize
IF (FSize > 20) OR (FSize < 0) THEN FSize = 20

N$ = Nm$ + ".MMM"
PRINT "Converting File: "; N$
OPEN PATH$ + N$ FOR INPUT AS #1
 INPUT #1,MapName$   'Loads MapName From First Line
 INPUT #1,MapDimX    'Loads MapDimX From second Line
   PRINT "MapDimX:"; MapDimX
 INPUT #1,MapDimY    'Loads MapDimY From 3rd Line
   PRINT "MapDimY:"; MapDimY
 INPUT #1,Enemies    'Loads if enemies will be on current map
   PRINT "Enemies:"; Enemies
 REDIM Map(MapDimX, MapDimY, 2) AS INTEGER
 FOR X = 1 TO MapDimX
   FOR Y = 1 TO MapDimY
     INPUT #1,Map(X, Y, 1)  'Alternates Between the Icon # and the
     INPUT #1,Map(X, Y, 2)  'Attrib for each coordinate of the map
   NEXT Y
 NEXT X
CLOSE #1
TMI = 0
FOR X = 1 TO MapDimX
  FOR Y = 1 TO MapDimY
    LOCATE 10, 40: PRINT "(" + STR$(X) + "," + STR$(Y) + ")" + "    "
    IF Map(X, Y, 1) <> 0 THEN
       NumFound = False: D = 0
       WHILE (D <= TMI) AND (NumFound = False)
         D = D + 1
         IF Map(X, Y, 1) = NINums(D) THEN NumFound = True
       WEND
       IF NumFound = False THEN
          TMI = TMI + 1
          NINums(TMI) = Map(X, Y, 1)
          Map(X, Y, 1) = TMI
          LOCATE 10, 1: PRINT "NEW ICON #"; TMI
        ELSE
          Map(X, Y, 1) = D
       END IF
    END IF
  NEXT Y
NEXT X
FOR D = 1 TO TMI
OPEN MMIPath$ + Icons(NINums(D)) FOR INPUT AS #1
   FOR T = 1 TO 203
     INPUT #1,Num
     MapIcons(T, D) = Num
   NEXT T
CLOSE #1
NEXT D


Nm$ = Nm$ + ".NMF"
PRINT "File Loaded." + STR$(TMI) + "  Saving file " + Nm$
INPUT "Press ENTER To Continue..", Con$
CLS
PRINT "Now Making new file..."
OPEN NPATH$ + Nm$ FOR RANDOM AS #1 LEN = 2
  PUT #1, 1, MapDimX
  PRINT "MapDimX:"; MapDimX
  PUT #1, 2, MapDimY
  PRINT "MapDimY:"; MapDimY
  PUT #1, 3, Enemies
  PRINT "Enemies:"; Enemies
  PUT #1, 4, HSize
  PRINT "HalfSeeSize:"; HSize
  PUT #1, 5, FSize
  PRINT "FullSeeSize:"; FSize
  PUT #1, 6, TMI
  PRINT "TMI: "; TMI
  FOR X = 1 TO MapDimX
    FOR Y = 1 TO MapDimY
      PUT #1, , Map(X, Y, 1)
      PUT #1, , Map(X, Y, 2)
    NEXT Y
  NEXT X
  FOR Y = 1 TO TMI
    FOR X = 1 TO 203
      PUT #1, , MapIcons(X, Y)
    NEXT X
  NEXT Y
CLOSE #1
INPUT "Convert Another"; YN$
IF UCASE$(LEFT$(YN$, 1)) = "Y" THEN GOTO 1
