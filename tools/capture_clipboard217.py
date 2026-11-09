"""Capture native copy from the same public release in Chrome and Firefox.

No synthetic clipboard provider or HTML rewrite participates in these captures.
The second fixture measures authored colors independently of GitHub's theme.
"""
import hashlib
import json
from pathlib import Path
import subprocess

from playwright.sync_api import sync_playwright

OUT = Path('clipboard217-evidence')
URL = 'https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.38'
CONTROL = '''<!doctype html><html><head><meta charset="utf-8"><style>
body { color: var(--fg); background: var(--bg); font-family: sans-serif; }
@media (prefers-color-scheme: light) { body { --fg:#1f2328; --bg:#ffffff; } }
@media (prefers-color-scheme: dark) { body { --fg:#f0f6fc; --bg:#0d1117; } }
</style></head><body><article id="control"><h2>Clipboard control</h2>
<p>Ordinary <strong>bold</strong> and <em>italic</em> text.
<span style="color:#ff0000">Intentional red</span>;
<span style="background-color:#ffff00;color:#000000">Intentional highlight</span>.
<a href="https://example.org/">A link</a>.</p>
<p style="color:#f0f6fc;background:#0d1117">Intentional light on dark panel</p>
<ul><li>First item</li><li>Second item</li></ul></article></body></html>'''


def capture(page, selector, stem):
    node = page.locator(selector).first
    node.wait_for(state='visible')
    info = node.evaluate('''el => {
      const r=document.createRange(); r.selectNodeContents(el);
      const s=window.getSelection(); s.removeAllRanges(); s.addRange(r);
      const ancestors=[]; for(let n=el;n;n=n.parentElement) {
        const c=getComputedStyle(n); ancestors.push({tag:n.tagName,
          class:n.className,color:c.color,background:c.backgroundColor,
          style:n.getAttribute('style')});
      }
      return {text:s.toString(),html:el.outerHTML,ancestors,
        scheme:matchMedia('(prefers-color-scheme: dark)').matches,
        root:document.documentElement.outerHTML.slice(0,700)};
    }''')
    assert info['text'].strip(), stem
    page.keyboard.press('Control+c')
    # Clipboard ownership remains with the running browser while xclip reads.
    types = subprocess.check_output(['xclip','-selection','clipboard','-o','-t','TARGETS'],timeout=15)
    html = subprocess.check_output(['xclip','-selection','clipboard','-o','-t','text/html'],timeout=15)
    text = subprocess.check_output(['xclip','-selection','clipboard','-o','-t','UTF8_STRING'],timeout=15)
    (OUT / (stem+'.html')).write_bytes(html)
    (OUT / (stem+'.txt')).write_bytes(text)
    (OUT / (stem+'-targets.txt')).write_bytes(types)
    info['html_sha256'] = hashlib.sha256(html).hexdigest()
    info['html_bytes'] = len(html)
    (OUT / (stem+'.json')).write_text(json.dumps(info,ensure_ascii=False,indent=2))
    page.screenshot(path=str(OUT / (stem+'.png')),full_page=False)
    print(stem, len(html), info['ancestors'],flush=True)


def main():
    OUT.mkdir(exist_ok=True)
    versions = {}
    with sync_playwright() as p:
        for name, kind, kwargs in [('chrome',p.chromium,{'channel':'chrome'}),('firefox',p.firefox,{})]:
            browser = kind.launch(headless=False,**kwargs)
            versions[name] = browser.version
            for theme in ['light','dark']:
                context = browser.new_context(color_scheme=theme,viewport={'width':1500,'height':1300})
                page = context.new_page()
                page.goto(URL,wait_until='networkidle',timeout=90000)
                page.locator('main .markdown-body').first.wait_for()
                assert 'Recovery in the start view' in page.locator('main .markdown-body').first.inner_text()
                capture(page,'main .markdown-body',f'{name}-{theme}-release')
                page.set_content(CONTROL,wait_until='load')
                capture(page,'#control',f'{name}-{theme}-authored')
                context.close()
            browser.close()
    (OUT/'versions.json').write_text(json.dumps({'browsers':versions,'url':URL},indent=2))
    print('BROWSER217_CAPTURE_OK',versions,flush=True)


if __name__ == '__main__':
    main()
