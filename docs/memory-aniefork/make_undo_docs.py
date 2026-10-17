"""Synthetic documents for measuring what an undo step costs (our own text, names and numbers,
nothing from Word). Each has the same 74 pages of text as text-100p and adds one kind of thing a
document from a real office carries in quantity:

  undo-plain.docx      the text alone
  undo-styles.docx     600 styles (400 paragraph, 200 character), 376 latent-style entries and
                       60 list definitions of 9 levels each
  undo-comments.docx   300 comments
  undo-parts.docx      20 sections, each with its own header and footer, and 300 footnotes
  undo-revisions.docx  300 tracked insertions
  undo-all.docx        all of the above

Usage: python3 -I make_undo_docs.py <out dir>
"""
import os, random, sys, zipfile

out = sys.argv[1]
random.seed(11)
WORDS = ("garden river lantern quiet harbour meadow copper window orchard pebble signal "
         "thunder velvet compass ember harvest willow beacon canvas marble journey").split()
NS = ('xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" '
      'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"')
PARAGRAPHS = 650


def sentence(n):
    return " ".join(random.choice(WORDS) for _ in range(n)).capitalize() + "."


def run(text):
    return f'<w:r><w:t xml:space="preserve">{text}</w:t></w:r>'


def styles_xml(many):
    s = [f'<w:styles {NS}><w:docDefaults><w:rPrDefault><w:rPr><w:sz w:val="22"/></w:rPr></w:rPrDefault></w:docDefaults>']
    if many:
        s.append('<w:latentStyles w:count="376">')
        s += [f'<w:lsdException w:name="Latent Style {i}" w:uiPriority="{i % 99}" w:qFormat="1"/>' for i in range(376)]
        s.append("</w:latentStyles>")
    s.append('<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style>')
    if many:
        for i in range(400):
            s.append(f'<w:style w:type="paragraph" w:styleId="Body{i}"><w:name w:val="Body Variant {i}"/><w:basedOn w:val="Normal"/>'
                     f'<w:qFormat/><w:pPr><w:spacing w:before="{60 + i % 12 * 20}" w:after="{40 + i % 7 * 20}" w:line="276" w:lineRule="auto"/>'
                     f'<w:ind w:left="{i % 9 * 120}" w:right="{i % 5 * 60}"/><w:jc w:val="{("left", "both", "center")[i % 3]}"/></w:pPr>'
                     f'<w:rPr><w:rFonts w:ascii="Font {i % 23}" w:hAnsi="Font {i % 23}" w:cs="Font {i % 17}"/><w:color w:val="{i * 2654435 % 0xFFFFFF:06X}"/>'
                     f'<w:sz w:val="{18 + i % 10 * 2}"/><w:lang w:val="en-GB"/></w:rPr></w:style>')
        for i in range(200):
            s.append(f'<w:style w:type="character" w:styleId="Mark{i}"><w:name w:val="Marking {i}"/>'
                     f'<w:rPr><w:b/><w:i w:val="{i % 2}"/><w:color w:val="{i * 40503 % 0xFFFFFF:06X}"/><w:highlight w:val="yellow"/></w:rPr></w:style>')
    s.append("</w:styles>")
    return "".join(s)


def numbering_xml():
    s = [f"<w:numbering {NS}>"]
    for a in range(60):
        s.append(f'<w:abstractNum w:abstractNumId="{a}"><w:multiLevelType w:val="multilevel"/>')
        for lvl in range(9):
            fmt = ("decimal", "lowerLetter", "lowerRoman", "bullet")[(a + lvl) % 4]
            text = "•" if fmt == "bullet" else "".join(f"%{k + 1}." for k in range(lvl + 1))
            s.append(f'<w:lvl w:ilvl="{lvl}"><w:start w:val="1"/><w:numFmt w:val="{fmt}"/><w:lvlText w:val="{text}"/><w:lvlJc w:val="left"/>'
                     f'<w:pPr><w:ind w:left="{360 * (lvl + 1)}" w:hanging="360"/></w:pPr><w:rPr><w:rFonts w:ascii="Font {lvl}"/></w:rPr></w:lvl>')
        s.append("</w:abstractNum>")
    s += [f'<w:num w:numId="{n + 1}"><w:abstractNumId w:val="{n % 60}"/></w:num>' for n in range(100)]
    s.append("</w:numbering>")
    return "".join(s)


def docx(name, styles=False, comments=False, parts=False, revisions=False):
    files, rels, overrides = {}, [], []
    body, comment_xml, footnote_xml = [], [], []
    sections = 20 if parts else 1
    per_section = PARAGRAPHS // sections
    n_comments = n_notes = n_revs = 0
    for i in range(PARAGRAPHS):
        ppr = f'<w:pPr><w:pStyle w:val="Body{i % 400}"/></w:pPr>' if styles else ""
        text = " ".join(sentence(random.randint(8, 18)) for _ in range(3))
        inner = run(text)
        if comments and i % 2 == 0 and n_comments < 300:
            c = n_comments
            n_comments += 1
            inner = (f'<w:commentRangeStart w:id="{c}"/>{inner}<w:commentRangeEnd w:id="{c}"/>'
                     f'<w:r><w:commentReference w:id="{c}"/></w:r>')
            comment_xml.append(f'<w:comment w:id="{c}" w:author="Reviewer {c % 12}" w:initials="R{c % 12}" w:date="2026-0{1 + c % 9}-1{c % 9}T10:00:00Z">'
                               f'<w:p>{run(sentence(25))}</w:p></w:comment>')
        if parts and i % 2 == 1 and n_notes < 300:
            n_notes += 1
            inner += f'<w:r><w:footnoteReference w:id="{n_notes + 1}"/></w:r>'
            footnote_xml.append(f'<w:footnote w:id="{n_notes + 1}"><w:p>{run(sentence(15))}</w:p></w:footnote>')
        if revisions and i % 2 == 0 and n_revs < 300:
            inner += f'<w:ins w:id="{1000 + n_revs}" w:author="Editor {n_revs % 9}" w:date="2026-02-0{1 + n_revs % 9}T09:30:00Z">{run(" " + sentence(6))}</w:ins>'
            n_revs += 1
        if parts and i % per_section == per_section - 1 and i // per_section < sections - 1:
            k = i // per_section
            ppr = (ppr[:-8] if ppr else "<w:pPr>") + (f'<w:sectPr><w:headerReference w:type="default" r:id="rIdH{k}"/>'
                                                      f'<w:footerReference w:type="default" r:id="rIdF{k}"/></w:sectPr></w:pPr>')
        body.append(f"<w:p>{ppr}{inner}</w:p>")
    last = sections - 1
    final = (f'<w:sectPr><w:headerReference w:type="default" r:id="rIdH{last}"/><w:footerReference w:type="default" r:id="rIdF{last}"/>'
             if parts else "<w:sectPr>") + '<w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr>'
    files["word/document.xml"] = f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document {NS}><w:body>{"".join(body)}{final}</w:body></w:document>'
    ct = "application/vnd.openxmlformats-officedocument.wordprocessingml"
    files["word/styles.xml"] = styles_xml(styles)
    rels.append(("rIdStyles", "styles", "styles.xml"))
    overrides.append(("/word/styles.xml", f"{ct}.styles+xml"))
    if styles:
        files["word/numbering.xml"] = numbering_xml()
        rels.append(("rIdNumbering", "numbering", "numbering.xml"))
        overrides.append(("/word/numbering.xml", f"{ct}.numbering+xml"))
    if comments:
        files["word/comments.xml"] = f'<w:comments {NS}>{"".join(comment_xml)}</w:comments>'
        rels.append(("rIdComments", "comments", "comments.xml"))
        overrides.append(("/word/comments.xml", f"{ct}.comments+xml"))
    if parts:
        sep = '<w:footnote w:type="separator" w:id="0"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type="continuationSeparator" w:id="1"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>'
        files["word/footnotes.xml"] = f'<w:footnotes {NS}>{sep}{"".join(footnote_xml)}</w:footnotes>'
        rels.append(("rIdFootnotes", "footnotes", "footnotes.xml"))
        overrides.append(("/word/footnotes.xml", f"{ct}.footnotes+xml"))
        for k in range(sections):
            files[f"word/header{k}.xml"] = f'<w:hdr {NS}><w:p>{run(f"Section {k + 1} header: " + sentence(6))}</w:p></w:hdr>'
            files[f"word/footer{k}.xml"] = f'<w:ftr {NS}><w:p>{run(f"Section {k + 1} footer: " + sentence(6))}</w:p></w:ftr>'
            rels += [(f"rIdH{k}", "header", f"header{k}.xml"), (f"rIdF{k}", "footer", f"footer{k}.xml")]
            overrides += [(f"/word/header{k}.xml", f"{ct}.header+xml"), (f"/word/footer{k}.xml", f"{ct}.footer+xml")]
    rel_base = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/"
    files["word/_rels/document.xml.rels"] = ('<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
                                             + "".join(f'<Relationship Id="{i}" Type="{rel_base}{t}" Target="{target}"/>' for i, t, target in rels) + "</Relationships>")
    files["_rels/.rels"] = ('<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
                            f'<Relationship Id="rId1" Type="{rel_base}officeDocument" Target="word/document.xml"/></Relationships>')
    files["[Content_Types].xml"] = ('<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
                                    '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/>'
                                    f'<Override PartName="/word/document.xml" ContentType="{ct}.document.main+xml"/>'
                                    + "".join(f'<Override PartName="{p}" ContentType="{c}"/>' for p, c in overrides) + "</Types>")
    path = os.path.join(out, name)
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        for n, data in files.items():
            z.writestr(n, data)
    print(path, os.path.getsize(path) // 1024, "KB")


docx("undo-plain.docx")
docx("undo-styles.docx", styles=True)
docx("undo-comments.docx", comments=True)
docx("undo-parts.docx", parts=True)
docx("undo-revisions.docx", revisions=True)
docx("undo-all.docx", styles=True, comments=True, parts=True, revisions=True)
