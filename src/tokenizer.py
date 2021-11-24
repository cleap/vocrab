import spacy
import re
import textwrap

nlp = spacy.load("es_core_news_sm")

file_path = "data/emos-vs-punks.txt"

replace_subs = [
    ("\[[^\[\]]+\]: ", ""),
    ("\(SOUNDBITE ARCHIVO\)", ""),
    ("…", "."),
    ("\s+", " ")
]

punctuation_marks = ["»", "“", "¡", "&"]

text = ""
with open(file_path, "rt") as file:
    text = file.read()

for (old, new) in replace_subs:
    text = re.sub(old, new, text)

text = text.strip()
doc = nlp(text)

outfile_path = "data/emos-vs-punks.json"
with open(outfile_path, "w") as outfile:
    outfile.write("""{\n""")
    outfile.write("""  "sentences": [\n""")
    outfile.write("""    [\n""");
    for sentence_i, sentence in enumerate(doc.sents):
        if sentence_i > 0:
            outfile.write(""",\n    [\n""")
        tok_end=","
        for token_i, token in enumerate(sentence):
            if token_i > 0:
                outfile.write(""",\n""")
            pos = "PUNCT" if token.text in punctuation_marks else token.pos_
            outfile.write("""      {}\n""".format("{"))
            outfile.write("""        "text":"{}",\n""".format(token.text))
            outfile.write("""        "lemma":"{}",\n""".format(token.lemma_))
            outfile.write("""        "pos":"{}"\n""".format(pos))
            outfile.write("""      {}""".format("}"))
        outfile.write("""\n""")
        outfile.write("""    ]""")
    outfile.write("""\n""")
    outfile.write("""  ]\n""")
    outfile.write("""}""")
