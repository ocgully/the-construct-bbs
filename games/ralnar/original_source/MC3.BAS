'$DYNAMIC
CONST PATH$ = "E:\QB\SCREENS\"  'Screen Path
CONST NPATH$ = "E:\QB\SCREENS\" '.NMF Path
CONST MMIPATH$ = "E:\QB\MMIS\"
CONST MaxIcons = 310
CONST MaxIconsPerScreen = 150
CLS
DEFINT A-Z
DIM Icons(MaxIcons) AS STRING
OPEN MMIPATH$ + "MMIFILES.TXT" FOR INPUT AS #1
  WHILE NOT EOF(1)
    TotalIcons = TotalIcons + 1
    INPUT #1,Icons(TotalIcons)   'Loads a List of the MMI filenames
  WEND
CLOSE #1
DIM NINums(TotalIcons) AS INTEGER
DIM Map(1, 1, 1) AS INTEGER
DIM MapIcons(203, MaxIconsPerScreen) AS INTEGER
1 INPUT "Which file do you want to convert? ", Nm$
N$ = Nm$ + ".MMM"
OPEN PATH$ + N$ FOR INPUT AS #1
 INPUT #1,MapName$   'Loads MapName From First Line
 INPUT #1,MapDimX    'Loads MapDimX From second Line
 INPUT #1,MapDimY    'Loads MapDimY From 3rd Line
 INPUT #1,Enemies    'Loads if enemies will be on current map
 REDIM Map(MapDimX, MapDimY, 2) AS INTEGER
 FOR X = 1 TO MapDimX
   FOR Y = 1 TO MapDimY
     INPUT #1,Map(X, Y, 1)  'Alternates Between the Icon # and the
     INPUT #1,Map(X, Y, 2)  'Attrib for each coordinate of the map
   NEXT Y
 NEXT X
CLOSE #1
TotalMapIcons = 0
FOR X = 1 TO MapDimX
  FOR Y = 1 TO MapDimY
    IF Map(X, Y, 1) <> 0 THEN
      IF NINums(Map(X, Y, 1)) = 0 THEN
        TotalMapIcons = TotalMapIcons + 1
        NINums(Map(X, Y, 1)) = TotalMapIcons
        OPEN MMIPATH$ + Icons(Map(X, Y, 1)) FOR INPUT AS #1
        FOR I = 1 TO 203
          INPUT #1,X%
          MapIcons(I, TotalMapIcons) = X%
        NEXT I
        CLOSE #1
        Map(X, Y, 1) = TotalMapIcons
       ELSE
        Map(X, Y, 1) = NINums(Map(X, Y, 1))
      END IF
    END IF
  NEXT Y
NEXT X
Nm$ = Nm$ + ".NMF"
PRINT "File Loaded.  Saving file " + Nm$
OPEN NPATH$ + Nm$ FOR RANDOM AS #1 LEN = 2
  PUT #1, 1, MapDimX
  PUT #1, 2, MapDimY
  PUT #1, 3, Enemies
  PUT #1, 4, TotalMapIcons
  FOR X = 1 TO MapDimX
    FOR Y = 1 TO MapDimY
      PUT #1, , Map(X, Y, 1)
      PUT #1, , Map(X, Y, 2)
    NEXT Y
  NEXT X
  FOR Y = 1 TO TotalMapIcons
    FOR X = 1 TO 203
      PUT #1, , MapIcons(X, Y)
    NEXT X
  NEXT Y
CLOSE #1
INPUT "Convert Another"; YN$
IF UCASE$(LEFT$(YN$, 1)) = "Y" THEN GOTO 1
