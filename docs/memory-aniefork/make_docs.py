"""Synthetic test documents for memory measurements (our own text and pictures, nothing from Word).

Usage: python -I make_docs.py <out dir>. Writes text-100p.docx (74 pages), pics-100p.docx (89 pages,
20 PNG pictures of 2400 x 1800, about 92 MB) and four one-line documents for font fallback.
"""
import io, os, random, struct, sys, zipfile, zlib

out = sys.argv[1]
random.seed(7)
WORDS = ("garden river lantern quiet harbour meadow copper window orchard pebble signal "
         "thunder velvet compass ember harvest willow beacon canvas marble journey").split()

def sentence(n):
    w = [random.choice(WORDS) for _ in range(n)]
    return " ".join(w).capitalize() + "."

def para(text, style=None):
    ppr = f'<w:pPr><w:pStyle w:val="{style}"/></w:pPr>' if style else ""
    return f'<w:p>{ppr}<w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p>'

def png(w, h, seed):
    rnd = random.Random(seed)
    rows = []
    for y in range(h):
        row = bytearray([0])
        for x in range(w):
            row += bytes(((x * 255 // w) ^ rnd.randrange(32), (y * 255 // h), (seed * 37) & 255))
        rows.append(bytes(row))
    raw = b"".join(rows)
    def chunk(t, d):
        c = struct.pack(">I", len(d)) + t + d
        return c + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 6)) + chunk(b"IEND", b""))

def picture(rid, n, cx_in, cy_in):
    cx, cy = int(cx_in * 914400), int(cy_in * 914400)
    return (f'<w:p><w:r><w:drawing><wp:inline><wp:extent cx="{cx}" cy="{cy}"/><wp:docPr id="{n}" name="Picture {n}"/>'
            f'<a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic>'
            f'<pic:nvPicPr><pic:cNvPr id="{n}" name="p{n}.png"/><pic:cNvPicPr/></pic:nvPicPr>'
            f'<pic:blipFill><a:blip r:embed="{rid}"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>'
            f'<pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"/></pic:spPr>'
            f'</pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>')

def docx(path, paragraphs, pictures, pic_px):
    body, rels, media = [], [], []
    every = max(1, paragraphs // max(1, pictures)) if pictures else 0
    pic_no = 0
    for i in range(paragraphs):
        if i % 40 == 0:
            body.append(para(f"Section {i // 40 + 1}: " + sentence(4).rstrip("."), "Heading1"))
        body.append(para(" ".join(sentence(random.randint(8, 18)) for _ in range(3))))
        if pictures and i % every == every - 1 and pic_no < pictures:
            pic_no += 1
            rid = f"rIdImg{pic_no}"
            rels.append(f'<Relationship Id="{rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/p{pic_no}.png"/>')
            media.append((f"word/media/p{pic_no}.png", png(pic_px[0], pic_px[1], pic_no)))
            body.append(picture(rid, pic_no, 6.0, 6.0 * pic_px[1] / pic_px[0]))
    ns = ('xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" '
          'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" '
          'xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" '
          'xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" '
          'xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"')
    document = (f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document {ns}><w:body>'
                + "".join(body) + '<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr></w:body></w:document>')
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml", '<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Default Extension="png" ContentType="image/png"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>')
        z.writestr("_rels/.rels", '<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>')
        z.writestr("word/_rels/document.xml.rels", '<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' + "".join(rels) + "</Relationships>")
        z.writestr("word/document.xml", document)
        for name, data in media:
            z.writestr(name, data)
    print(path, os.path.getsize(path) // 1024, "KB,", paragraphs, "paragraphs,", len(media), "pictures")

docx(os.path.join(out, "text-100p.docx"), 650, 0, None)
docx(os.path.join(out, "pics-100p.docx"), 650, 20, (2400, 1800))

def one_line(name, text):
    ns = 'xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"'
    with zipfile.ZipFile(os.path.join(out, name), "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml", '<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>')
        z.writestr("_rels/.rels", '<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>')
        z.writestr("word/document.xml", f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document {ns}><w:body>{para(text)}</w:body></w:document>')

one_line("latin.docx", "Hello world, a plain line of text.")
one_line("cjk.docx", "Hello world 你好.")
one_line("emoji.docx", "Hello world 😀.")
one_line("cjk-emoji.docx", "Hello world 你好 😀 สวัสดี नमस्ते مرحبا.")
print("one-line documents written")
