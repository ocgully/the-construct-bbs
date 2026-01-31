#!/usr/bin/env python3
"""
QB45 Binary Symbol Extractor
Extracts readable symbol names from tokenized QuickBASIC 4.5 .BAS files

Usage:
    python qb45_symbols.py <file.BAS>
    python qb45_symbols.py *.BAS > all_symbols.txt
"""

import sys
import os
import re

def extract_symbols(data):
    """Extract readable symbol names from QB45 binary data."""
    symbols = {
        'subs': [],      # SUB names (flag 0x40 prefix)
        'functions': [], # FUNCTION names  
        'types': [],     # TYPE names (flag 0x08 prefix)
        'variables': [], # Variable names
        'strings': [],   # String literals
    }
    
    pos = 0
    while pos < len(data) - 4:
        # Look for name table entries
        # Format: [flags] [length] [name]
        # Flags: 0x40 = SUB/FUNCTION, 0x08 = TYPE, 0x00 = variable
        
        flags = data[pos]
        length = data[pos + 1]
        
        # Check for valid name entry
        if length > 0 and length < 64 and pos + 2 + length <= len(data):
            name_bytes = data[pos + 2:pos + 2 + length]
            
            # Check if it's readable ASCII
            if all(32 <= b < 127 for b in name_bytes):
                try:
                    name = name_bytes.decode('ascii')
                    
                    # Categorize based on flags and patterns
                    if flags == 0x40:
                        if name not in symbols['subs']:
                            symbols['subs'].append(name)
                    elif flags == 0x08:
                        if name not in symbols['types']:
                            symbols['types'].append(name)
                    elif flags == 0x04:
                        if name not in symbols['subs']:
                            symbols['subs'].append(name)
                    else:
                        # Likely a variable or other identifier
                        if name not in symbols['variables'] and len(name) > 1:
                            # Filter out obvious non-identifiers
                            if re.match(r'^[A-Za-z_][A-Za-z0-9_$.%&#!]*$', name):
                                symbols['variables'].append(name)
                    
                    pos += 2 + length
                    continue
                except:
                    pass
        
        # Also look for @ prefix pattern (another SUB marker)
        if data[pos] == 0x40 and pos + 1 < len(data):
            length = data[pos + 1]
            if length > 0 and length < 64 and pos + 2 + length <= len(data):
                name_bytes = data[pos + 2:pos + 2 + length]
                if all(32 <= b < 127 for b in name_bytes):
                    try:
                        name = name_bytes.decode('ascii')
                        if name not in symbols['subs'] and re.match(r'^[A-Za-z_][A-Za-z0-9_]*$', name):
                            symbols['subs'].append(name)
                        pos += 2 + length
                        continue
                    except:
                        pass
        
        pos += 1
    
    # Also extract string literals (often longer sequences)
    pos = 0
    while pos < len(data) - 5:
        length = data[pos]
        if 5 < length < 200 and pos + 1 + length <= len(data):
            str_bytes = data[pos + 1:pos + 1 + length]
            # Check if mostly printable with spaces and common punctuation
            printable_count = sum(1 for b in str_bytes if 32 <= b < 127)
            if printable_count > length * 0.9:
                try:
                    s = str_bytes.decode('ascii', errors='replace')
                    # Must contain at least some letters and look like text
                    if re.search(r'[A-Za-z]{2,}', s) and s not in symbols['strings']:
                        # Filter out things that look like binary/code
                        if not re.search(r'[\x00-\x1f]', s):
                            symbols['strings'].append(s)
                except:
                    pass
        pos += 1
    
    return symbols

def analyze_file(filepath):
    """Analyze a single QB45 binary file."""
    with open(filepath, 'rb') as f:
        data = f.read()
    
    # Check for QB45 signature
    if len(data) < 2 or data[0] != 0xFC:
        return None, "Not a QB45 binary format file"
    
    return extract_symbols(data), None

def main():
    if len(sys.argv) < 2:
        print("QB45 Binary Symbol Extractor")
        print("Usage: python qb45_symbols.py <file.BAS> [file2.BAS ...]")
        print()
        print("Extracts SUB/FUNCTION names, TYPE definitions, and variables")
        print("from tokenized QuickBASIC 4.5 binary .BAS files.")
        sys.exit(1)
    
    for filepath in sys.argv[1:]:
        if not os.path.exists(filepath):
            print(f"File not found: {filepath}", file=sys.stderr)
            continue
        
        print(f"=== {os.path.basename(filepath)} ===")
        
        symbols, error = analyze_file(filepath)
        
        if error:
            print(f"  Error: {error}")
            continue
        
        if symbols['subs']:
            print(f"  SUBs/FUNCTIONs ({len(symbols['subs'])}):")
            for name in sorted(symbols['subs']):
                print(f"    {name}")
        
        if symbols['types']:
            print(f"  TYPEs ({len(symbols['types'])}):")
            for name in sorted(symbols['types']):
                print(f"    {name}")
        
        if symbols['variables']:
            # Only show interesting variables (longer names, likely meaningful)
            interesting = [v for v in symbols['variables'] if len(v) > 3]
            if interesting:
                print(f"  Variables ({len(interesting)} shown):")
                for name in sorted(interesting)[:30]:  # Limit output
                    print(f"    {name}")
                if len(interesting) > 30:
                    print(f"    ... and {len(interesting) - 30} more")
        
        if symbols['strings']:
            # Show a few interesting strings
            interesting = [s for s in symbols['strings'] if len(s) > 10][:10]
            if interesting:
                print(f"  String literals (sample):")
                for s in interesting:
                    print(f'    "{s[:50]}{"..." if len(s) > 50 else ""}"')
        
        print()

if __name__ == '__main__':
    main()
