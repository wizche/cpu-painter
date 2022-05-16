import re
import os
import tempfile
import shlex 
import sys
import multiprocessing as mp

size = 10
#https://fontstruct.com/fontstructions/show/1404190/cg-pixel-4x5-1
font = "CG-pixel-4x5-mono-Regular"
tempdir = tempfile.mkdtemp()
currwd = os.getcwd()
os.chdir(tempdir)
byte_count = int((size-1) / 8)+1
content = ""

def zeros():
    return f"0x{'00' * byte_count}, "

def extract(c):
    if c < 0x21 or c > 0x7e:
        match = f"[{(zeros() * (size))[:-2]}],\n"
    else:
        escaped = chr(c)
        if escaped == '\\':
            escaped = "\\\\"
        escaped = shlex.quote(escaped)
        cmd = f"convert -scale {size}x{size}\! -font {font} -pointsize 72 label:{escaped} file{c}.xbm"
        ret = os.system(f"cd {tempdir} && {cmd}")
        if ret != 0:
            print(f"Failed to generate char '{chr(c)}': {ret}")
            print(cmd)
            return None
        with open(f"file{c}.xbm", "r") as f:
            s = f.read()
            matches = re.findall(r"\{(.*?)\}", s, re.MULTILINE | re.DOTALL)
            if len(matches) < 1:
                print(f"Failed to find generated code for char '{chr(c)}': {s}")
                print(cmd)
                return None
            match = matches[0]
            match = match.replace('\n', '').strip()
            match = match.replace('   ', ' ').strip()
            if match[-1:] == ",":
                match = match[:-1]
            if byte_count > 0:
                tokens = match.split(", ")    
                final_tokens = []
                for i in range(0, len(tokens), byte_count):
                    token = tokens[i+1] + tokens[i].replace("0x", "")
                    final_tokens.append(token)
                final_tokens = final_tokens[:]
                match = ", ".join(final_tokens)

            match = f"[{match}],"

        match += f"   // U+{c:04x} ({chr(c) if c > 0x20 else ''})\n" 
    return (c, match)

pool = mp.Pool(mp.cpu_count())
results = pool.map(extract, [c for c in range(0x0, 0x80)])
content = ""
for c in results:
    content += f"{c[1]}"

os.chdir(currwd)
#print(results)
pool.close()
template = f"pub const font_bitmap: [[u16; {size}]; 128] = [\n{content}];"
with open("src/font.rs", "w+") as f:
    f.write(template)
print(template)