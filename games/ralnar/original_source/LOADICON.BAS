'$DYNAMIC
SCREEN 13
OPTION BASE 1
DIM SHARED WORLD(200, 200) AS INTEGER
DIM SHARED Attrib(100, 100) AS INTEGER
DIM SHARED Icon(100, 20, 20)  AS INTEGER
DIM SHARED TempIcon(20, 20) AS INTEGER
T = 0
OPEN "C:\BG_RPG\BASIC\PICS\PicFiles.txt" FOR INPUT AS #1
  WHILE NOT (EOF(1))
     T = T + 1
     INPUT #1,IconName$
     OPEN "C:\BG_RPG\BASIC\PICS\" + IconName$ FOR INPUT AS #2
        FOR X = 1 TO 20
           FOR Y = 1 TO 20
               INPUT #2,Temp
               PSET (X, Y), Temp
           NEXT Y
        NEXT X
     CLOSE #2
     GET (1, 1)-(20, 20), TempIcon
     FOR X = 1 TO 20
       FOR Y = 1 TO 20
         Icon(T, X, Y) = TempIcon(X, Y)
       NEXT Y
     NEXT X
   WEND
   


FOR X = 1 TO 200 STEP 20
  FOR Y = 1 TO 180 STEP 20
    FOR XX = 1 TO 20
      FOR YY = 1 TO 20
        TempIcon(XX, YY) = Icon(14, XX, YY)
      NEXT YY
    NEXT XX
    PUT (X, Y), TempIcon, PSET
  NEXT Y
NEXT X