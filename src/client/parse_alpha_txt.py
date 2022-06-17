from string import ascii_lowercase
LETTERS = {letter: str(index) for index, letter in enumerate(ascii_lowercase, start=1)} 

def alph_pos(text):
    text = text.lower()
    numbers = [LETTERS[character] for character in text if character in LETTERS]
    return numbers

big = {}

with open("words_alpha.txt", "r") as f:
    lines = f.readlines()
    for bx in lines:
        x = bx.replace("\n", "")
        if len(x) > 1:
            for i in range(1, len(x)-1):
                print("1"+x[0:i])
                big[x[0:i]] = "false"
    for x in lines:
        print("2" + x)
        big[x.replace("\n", "")] = "true"
    f.close()

print(len(big))

text = "use phf::{phf_map};\npub static DICT: &'static phf::Map<&str, bool> = &phf_map! {"
for i, key in enumerate(big):
    print("3"+key+" : "+big[key])
    text = "{t}\"{k}\"=>{v},".format(t = text,k = key , v = big[key])
text = text + "};"
text = text.replace(",}", "}")

with open("dict.rs", "w") as g:
    g.write(text)
    g.close()
