'PLAY - a device I/O statement that plays music
'
'Syntax
'  PLAY commandstring
'    ş commandstring is a stringexpression that contains music commands:
'ÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄSet Octaves and Play TonesÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄ
' On  Sets current octave (n = 0-6)    ³  < or >  Up or down one octave
' Nn  Plays note n (n = 0-84, 0 is a   ³  A-G  Plays A, B, ..., G in current
'     rest)                            ³      octave (+ = sharp, - = flat)
'ÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄSet Tone Duration and TempoÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄ
' Ln  Sets length of a note (L1 is     ³  MS  Each note plays 3/4 of length
'     whole note, L4 is quarter note,  ³  MN  Each note plays 7/8 of length
'     etc.)  n = 1-64                  ³  ML  Each note plays full length
' Tn  Sets number of quarter notes per ³  Pn  Pause for the duration of
'     minute (n = 32-255, 120 default  ³      n quarternotes (n = 1-64)
'ÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄSet OperationÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄ
' MF  Plays music in foreground        ³  MB  Plays music in background
'ÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄÄ

PLAY "<< MB T180 L4; F# F# G# L8 A L3 MN F#.. ML"
PLAY "MB L4 G# L8 A L3 MN E.. ML L4 G# L8 A L3 MN F#..."
PLAY "MB ML L8 <B> C# E"
PLAY "MB L4; F# F# G# L8 A L3 MN F#.. ML"
PLAY "MB L4 G# L8 A L3 MN E.. ML L4 G# A A "
PLAY "MB L16 G# A L8 G# L4 F# L8 E.. P8 "

PLAY "MB >MS T120 L16 C C <G> C  D C <G> C D# C <G> C D C <G> C"
PLAY "MB C C <G#> C D C <G#> C D# C <G#> C D C <G#> C"

PLAY "MB D# F <A#> D; F F <A#> D; G F <A#> D; F F D# D"
PLAY "MB L8 MS C... L6 C. ML<"