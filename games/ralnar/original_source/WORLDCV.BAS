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
PRINT "THIS FILE CONVERTS 'WORLD.MMM' INTO A SMALLER VERSION OF ITSELF!"
PRINT "BE CAREFUL RUNNING THIS PROGRAM!!!!!!"
INPUT "Which file do you want to convert? ", Nm$

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

OPEN PATH$ + N$ FOR OUTPUT AS #1
 WRITE #1, MapName$  'Loads MapName From First Line
 WRITE #1, 233   'Loads MapDimX From second Line
   PRINT "MapDimX:"; 233
 WRITE #1, 178   'Loads MapDimY From 3rd Line
   PRINT "MapDimY:"; 178
 WRITE #1, Enemies   'Loads if enemies will be on current map
   PRINT "Enemies:"; Enemies
 
 FOR X = 1 TO MapDimX
   FOR Y = 1 TO MapDimY
    IF (X < 234) AND (Y < 179) THEN
     WRITE #1, Map(X, Y, 1) 'Alternates Between the Icon # and the
     WRITE #1, Map(X, Y, 2) 'Attrib for each coordinate of the map
    END IF
   NEXT Y
 NEXT X
CLOSE #1
