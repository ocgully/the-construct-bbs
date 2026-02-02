MapName$ = "WORLD"
MapDimX = 260
MapDimY = 198
PRINT
PRINT "Map Name: "; MapName$
PRINT "Map X Dimension: "; MapDimX
PRINT "Map Y Dimension: "; MapDimY
PRINT: PRINT "Is this correct? (Y/N): "
DO
  A$ = INKEY$
LOOP UNTIL UCASE$(A$) = "Y" OR UCASE$(A$) = "N"
IF UCASE$(A$) = "Y" THEN
  PRINT "Allocating Memory..."
  DIM SHARED Map(MapDimX, MapDimY, 2) AS INTEGER
  WorldX = INT(MapDimX / 2)
  WorldY = INT(MapDimY / 2)
  '^^ ^^^^^^ ^^^ Makes Array for screen by the new dimensions
  FOR X = 1 TO MapDimX
    FOR Y = 1 TO MapDimY
      Map(X, Y, 1) = 11
      Map(X, Y, 2) = 1
    NEXT Y
  NEXT X
  MenuDone = True
END IF


PATHNAME$ = MapPath$ + MapName$ + ".MMM"
OPEN PATHNAME$ FOR OUTPUT AS #1
    WRITE #1, MapName$  'Saves MapName On First Line
    WRITE #1, MapDimX   'Saves MapDimX on second Line
    WRITE #1, MapDimY   'Saves MapDimY on 3rd Line
    FOR X = 1 TO MapDimX
      FOR Y = 1 TO MapDimY
        WRITE #1, Map(X, Y, 1) 'Alternates Between the Icon # and the
        WRITE #1, Map(X, Y, 2) 'Attrib for each coordinate of the map
      NEXT Y
    NEXT X
    CLOSE #1