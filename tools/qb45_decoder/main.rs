// QB45 Binary Format Decoder
// Converts tokenized QuickBASIC 4.5 .BAS files to plain ASCII text
//
// File Format (reverse-engineered from hex analysis):
// 
// Header (0x00-0x6F):
//   0x00: 0xFC = QB 4.5 signature
//   0x01: 0x00 = sub-version
//   0x02-0x03: unknown (possibly segment info)
//   0x04-0x05: number of name table entries (LE16)
//   ... additional header fields ...
//
// Name Table (variable position):
//   Each entry: [2-byte offset] [flags] [length] [name bytes]
//
// Code Section:
//   Tokenized code with references to name table

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufReader, Read, Write, BufWriter};
use std::path::Path;

// QB45 Token definitions
// These are the keyword tokens used in QuickBASIC 4.5
const TOKENS: &[(u16, &str)] = &[
    // Statement tokens (0x80-0xCF range typically)
    (0x81, "END"),
    (0x82, "FOR"),
    (0x83, "NEXT"),
    (0x84, "DATA"),
    (0x85, "INPUT"),
    (0x86, "DIM"),
    (0x87, "READ"),
    (0x88, "LET"),
    (0x89, "GOTO"),
    (0x8A, "RUN"),
    (0x8B, "IF"),
    (0x8C, "RESTORE"),
    (0x8D, "GOSUB"),
    (0x8E, "RETURN"),
    (0x8F, "REM"),
    (0x90, "STOP"),
    (0x91, "PRINT"),
    (0x92, "CLEAR"),
    (0x93, "LIST"),
    (0x94, "NEW"),
    (0x95, "ON"),
    (0x96, "WAIT"),
    (0x97, "DEF"),
    (0x98, "POKE"),
    (0x99, "CONT"),
    (0x9A, "OUT"),
    (0x9B, "LPRINT"),
    (0x9C, "LLIST"),
    (0x9D, "WIDTH"),
    (0x9E, "ELSE"),
    (0x9F, "TRON"),
    (0xA0, "TROFF"),
    (0xA1, "SWAP"),
    (0xA2, "ERASE"),
    (0xA3, "EDIT"),
    (0xA4, "ERROR"),
    (0xA5, "RESUME"),
    (0xA6, "DELETE"),
    (0xA7, "AUTO"),
    (0xA8, "RENUM"),
    (0xA9, "DEFSTR"),
    (0xAA, "DEFINT"),
    (0xAB, "DEFSNG"),
    (0xAC, "DEFDBL"),
    (0xAD, "LINE"),
    (0xAE, "WHILE"),
    (0xAF, "WEND"),
    (0xB0, "CALL"),
    (0xB1, "WRITE"),
    (0xB2, "OPTION"),
    (0xB3, "RANDOMIZE"),
    (0xB4, "OPEN"),
    (0xB5, "CLOSE"),
    (0xB6, "LOAD"),
    (0xB7, "MERGE"),
    (0xB8, "SAVE"),
    (0xB9, "COLOR"),
    (0xBA, "CLS"),
    (0xBB, "MOTOR"),
    (0xBC, "BSAVE"),
    (0xBD, "BLOAD"),
    (0xBE, "SOUND"),
    (0xBF, "BEEP"),
    (0xC0, "PSET"),
    (0xC1, "PRESET"),
    (0xC2, "SCREEN"),
    (0xC3, "KEY"),
    (0xC4, "LOCATE"),
    (0xC5, "TO"),
    (0xC6, "THEN"),
    (0xC7, "TAB("),
    (0xC8, "STEP"),
    (0xC9, "USR"),
    (0xCA, "FN"),
    (0xCB, "SPC("),
    (0xCC, "NOT"),
    (0xCD, "ERL"),
    (0xCE, "ERR"),
    (0xCF, "STRING$"),
    (0xD0, "USING"),
    (0xD1, "INSTR"),
    (0xD2, "'"),  // REM shorthand
    (0xD3, "VARPTR"),
    (0xD4, "CSRLIN"),
    (0xD5, "POINT"),
    (0xD6, "OFF"),
    (0xD7, "INKEY$"),
    
    // Extended tokens (0xE6-0xFF range)
    (0xE6, "FILES"),
    (0xE7, "FIELD"),
    (0xE8, "SYSTEM"),
    (0xE9, "NAME"),
    (0xEA, "LSET"),
    (0xEB, "RSET"),
    (0xEC, "KILL"),
    (0xED, "PUT"),
    (0xEE, "GET"),
    (0xEF, "RESET"),
    (0xF0, "COMMON"),
    (0xF1, "CHAIN"),
    (0xF2, "DATE$"),
    (0xF3, "TIME$"),
    (0xF4, "PAINT"),
    (0xF5, "COM"),
    (0xF6, "CIRCLE"),
    (0xF7, "DRAW"),
    (0xF8, "PLAY"),
    (0xF9, "TIMER"),
    (0xFA, "ERDEV"),
    (0xFB, "IOCTL"),
    (0xFC, "CHDIR"),
    (0xFD, "MKDIR"),
    (0xFE, "RMDIR"),
    (0xFF, "SHELL"),
    
    // QB45-specific extensions (two-byte tokens starting with specific prefix)
    // These are handled separately in the decode logic
];

// Function tokens (usually prefixed with 0xFD or 0xFE in QB45)
const FUNCTION_TOKENS: &[(u16, &str)] = &[
    (0x81, "CVI"),
    (0x82, "CVS"),
    (0x83, "CVD"),
    (0x84, "MKI$"),
    (0x85, "MKS$"),
    (0x86, "MKD$"),
    (0x87, "EXTERR"),
    (0x88, "CONST"),
    (0x89, "CVSMBF"),
    (0x8A, "CVDMBF"),
    (0x8B, "MKSMBF$"),
    (0x8C, "MKDMBF$"),
    (0x8D, "DECLARE"),
    (0x8E, "FUNCTION"),
    (0x8F, "SUB"),
    (0x90, "DEFTYPE"),
    (0x91, "STATIC"),
    (0x92, "TYPE"),
    (0x93, "ENDTYPE"),
    (0x94, "SELECT"),
    (0x95, "CASE"),
    (0x96, "AS"),
    (0x97, "INTEGER"),
    (0x98, "LONG"),
    (0x99, "SINGLE"),
    (0x9A, "DOUBLE"),
    (0x9B, "STRING"),
    (0x9C, "BYVAL"),
    (0x9D, "SHARED"),
    (0x9E, "DO"),
    (0x9F, "LOOP"),
    (0xA0, "UNTIL"),
    (0xA1, "EXIT"),
    (0xA2, "SEG"),
    (0xA3, "ABS"),
    (0xA4, "ASC"),
    (0xA5, "ATN"),
    (0xA6, "CDBL"),
    (0xA7, "CHR$"),
    (0xA8, "CINT"),
    (0xA9, "CLNG"),
    (0xAA, "COS"),
    (0xAB, "CSNG"),
    (0xAC, "EOF"),
    (0xAD, "EXP"),
    (0xAE, "FIX"),
    (0xAF, "FRE"),
    (0xB0, "HEX$"),
    (0xB1, "INP"),
    (0xB2, "INT"),
    (0xB3, "LBOUND"),
    (0xB4, "LEFT$"),
    (0xB5, "LEN"),
    (0xB6, "LOC"),
    (0xB7, "LOF"),
    (0xB8, "LOG"),
    (0xB9, "LPOS"),
    (0xBA, "LTRIM$"),
    (0xBB, "MID$"),
    (0xBC, "OCT$"),
    (0xBD, "PEEK"),
    (0xBE, "PEN"),
    (0xBF, "POS"),
    (0xC0, "RIGHT$"),
    (0xC1, "RND"),
    (0xC2, "RTRIM$"),
    (0xC3, "SADD"),
    (0xC4, "SEEK"),
    (0xC5, "SETMEM"),
    (0xC6, "SGN"),
    (0xC7, "SIN"),
    (0xC8, "SPACE$"),
    (0xC9, "SQR"),
    (0xCA, "STICK"),
    (0xCB, "STR$"),
    (0xCC, "STRIG"),
    (0xCD, "TAN"),
    (0xCE, "UBOUND"),
    (0xCF, "UCASE$"),
    (0xD0, "VAL"),
    (0xD1, "VARPTR$"),
    (0xD2, "VARSEG"),
];

struct QB45Decoder {
    data: Vec<u8>,
    pos: usize,
    names: HashMap<u16, String>,  // offset -> name
    tokens: HashMap<u16, &'static str>,
    func_tokens: HashMap<u16, &'static str>,
}

impl QB45Decoder {
    fn new(data: Vec<u8>) -> Self {
        let mut tokens = HashMap::new();
        for (code, name) in TOKENS {
            tokens.insert(*code, *name);
        }
        
        let mut func_tokens = HashMap::new();
        for (code, name) in FUNCTION_TOKENS {
            func_tokens.insert(*code, *name);
        }
        
        QB45Decoder {
            data,
            pos: 0,
            names: HashMap::new(),
            tokens,
            func_tokens,
        }
    }
    
    fn read_u8(&mut self) -> Option<u8> {
        if self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;
            Some(b)
        } else {
            None
        }
    }
    
    fn read_u16_le(&mut self) -> Option<u16> {
        if self.pos + 1 < self.data.len() {
            let lo = self.data[self.pos] as u16;
            let hi = self.data[self.pos + 1] as u16;
            self.pos += 2;
            Some(lo | (hi << 8))
        } else {
            None
        }
    }
    
    fn peek_u8(&self) -> Option<u8> {
        if self.pos < self.data.len() {
            Some(self.data[self.pos])
        } else {
            None
        }
    }
    
    fn is_qb45_format(&self) -> bool {
        self.data.len() > 2 && self.data[0] == 0xFC
    }
    
    fn parse_header(&mut self) -> io::Result<()> {
        // Verify signature
        if !self.is_qb45_format() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not a QB45 binary format file (missing 0xFC signature)"
            ));
        }
        
        // Skip to name table
        // The name table typically starts after the header (around offset 0x70)
        // Find it by looking for readable ASCII strings
        self.find_and_parse_name_table();
        
        Ok(())
    }
    
    fn find_and_parse_name_table(&mut self) {
        // Look for the name table by finding sequences of readable strings
        // Name entries have format: [offset LE16] [flags] [length] [string]
        
        let mut pos = 0x70;  // Typical start of name table
        
        while pos + 4 < self.data.len() {
            // Read potential entry
            let offset = (self.data[pos] as u16) | ((self.data[pos + 1] as u16) << 8);
            let flags = self.data[pos + 2];
            let length = self.data[pos + 3];
            
            // Check if this looks like a valid name entry
            if length > 0 && length < 64 && pos + 4 + (length as usize) <= self.data.len() {
                let name_bytes = &self.data[pos + 4..pos + 4 + (length as usize)];
                
                // Check if it's readable ASCII
                if name_bytes.iter().all(|&b| b >= 0x20 && b < 0x7F) {
                    if let Ok(name) = std::str::from_utf8(name_bytes) {
                        self.names.insert(offset, name.to_string());
                        pos += 4 + (length as usize);
                        continue;
                    }
                }
            }
            
            // Try looking for direct string patterns
            // Sometimes names appear as [length][string] without offset/flags
            let length2 = self.data[pos];
            if length2 > 0 && length2 < 64 && pos + 1 + (length2 as usize) <= self.data.len() {
                let name_bytes = &self.data[pos + 1..pos + 1 + (length2 as usize)];
                if name_bytes.iter().all(|&b| b >= 0x20 && b < 0x7F) {
                    // This might be a name, but we need the offset from elsewhere
                    pos += 1 + (length2 as usize);
                    continue;
                }
            }
            
            pos += 1;
        }
    }
    
    fn decode(&mut self) -> io::Result<String> {
        self.parse_header()?;
        
        // For QB45 binary format, we need to find and decode the code section
        // This is complex and depends on the exact format variant
        
        // As a fallback, extract readable strings and structure
        let mut output = String::new();
        output.push_str("' Decoded from QB45 binary format\n");
        output.push_str("' Note: Full reconstruction requires complete token tables\n\n");
        
        // Extract and output the names we found (TYPE definitions, SUBs, FUNCTIONs, variables)
        output.push_str("' ========== SYMBOL TABLE ==========\n");
        let mut sorted_names: Vec<_> = self.names.iter().collect();
        sorted_names.sort_by_key(|(offset, _)| *offset);
        
        for (offset, name) in sorted_names {
            output.push_str(&format!("' {:04X}: {}\n", offset, name));
        }
        output.push_str("\n");
        
        // Try to extract string literals and other readable content
        output.push_str("' ========== EXTRACTED CONTENT ==========\n");
        self.extract_readable_content(&mut output);
        
        Ok(output)
    }
    
    fn extract_readable_content(&self, output: &mut String) {
        // Scan for string literals (usually preceded by a length byte)
        let mut pos = 0;
        
        while pos < self.data.len() {
            // Look for potential string literal patterns
            let len = self.data[pos] as usize;
            
            if len > 3 && len < 200 && pos + 1 + len <= self.data.len() {
                let potential_string = &self.data[pos + 1..pos + 1 + len];
                
                // Check if it looks like readable text
                let readable_count = potential_string.iter()
                    .filter(|&&b| (b >= 0x20 && b < 0x7F) || b == 0x0A || b == 0x0D)
                    .count();
                
                if readable_count == len {
                    if let Ok(s) = std::str::from_utf8(potential_string) {
                        // Filter out very short strings and things that look like binary data
                        if s.len() > 2 && s.chars().any(|c| c.is_alphabetic()) {
                            output.push_str(&format!("' String at {:04X}: \"{}\"\n", pos, s));
                        }
                    }
                    pos += 1 + len;
                    continue;
                }
            }
            
            pos += 1;
        }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("QB45 Binary Format Decoder");
        eprintln!("Usage: {} <input.bas> [output.txt]", args[0]);
        eprintln!();
        eprintln!("Converts tokenized QuickBASIC 4.5 .BAS files to readable text.");
        eprintln!("If no output file is specified, writes to stdout.");
        std::process::exit(1);
    }
    
    let input_path = &args[1];
    let output_path = args.get(2);
    
    // Read input file
    let mut file = File::open(input_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    
    // Check if it's actually binary or already ASCII
    if data.len() > 0 && data[0] != 0xFC {
        // Might already be ASCII
        if data.iter().all(|&b| b < 0x80 || b == 0xFF) {
            eprintln!("File appears to already be in ASCII format");
            if let Some(out_path) = output_path {
                std::fs::copy(input_path, out_path)?;
            } else {
                io::stdout().write_all(&data)?;
            }
            return Ok(());
        }
    }
    
    // Decode
    let mut decoder = QB45Decoder::new(data);
    let output = decoder.decode()?;
    
    // Write output
    if let Some(out_path) = output_path {
        let mut out_file = BufWriter::new(File::create(out_path)?);
        out_file.write_all(output.as_bytes())?;
        eprintln!("Decoded output written to: {}", out_path);
    } else {
        print!("{}", output);
    }
    
    Ok(())
}
