import spacy
import re
import textwrap
import argparse

if __name__ == "__main__":
    
    parser = argparse.ArgumentParser()
    parser.add_argument('file', help='the text file to read from')
    parser.add_argument('lang', help='the language of the text')
    args = parser.parse_args()

    packages = {
        "english": "en_core_web_sm",
        "spanish": "es_core_news_sm",
        "french":  "fr_core_news_sm"
    }
    nlp = spacy.load(packages[args.lang])

    file_path = args.file

    replace_subs = [
        ("\[[^\[\]]+\]: ", ""),
        ("\(SOUNDBITE ARCHIVO\)", ""),
        ("\s+", " "),
        ("…", "."),
    ]

    punctuation_marks = ["-", "»", "“", "”", "¡", "&"]

    print("Reading from {}.".format(file_path))
    text = ""
    with open(file_path, "rt") as file:
        text = file.read()

    for (old, new) in replace_subs:
        text = re.sub(old, new, text)

    text = text.strip()
    doc = nlp(text)

    outfile_name = args.file.split('.')[0]
    outfile_path = "{}.json".format(outfile_name)
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
                outfile.write("""        "lemma":"{}",\n""".format(token.lemma_.lower()))
                outfile.write("""        "pos":"{}"\n""".format(pos))
                outfile.write("""      {}""".format("}"))
            outfile.write("""\n""")
            outfile.write("""    ]""")
        outfile.write("""\n""")
        outfile.write("""  ]\n""")
        outfile.write("""}""")
    print("Results written into {}.".format(outfile_path))
