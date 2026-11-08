"""Translate AppStream prose and list entries using Leaf's gettext catalogs."""
import copy
import xml.etree.ElementTree as ET

LANG = '{http://www.w3.org/XML/1998/namespace}lang'


def text_nodes(root):
    nodes = list(root.findall('summary'))
    for description in root.findall('description'):
        nodes.extend(node for node in description.iter() if node.tag in ('p', 'li'))
    return [node for node in nodes if LANG not in node.attrib]


def translate(root, catalogs):
    # Do not translate structural <ul>/<ol> nodes. AppStream localizes each
    # paragraph/list item; release prose is already bilingual in news.json.
    parents = {node: parent for parent in root.iter() for node in parent}
    for node in text_nodes(root):
        source = node.text or ''
        parent = parents[node]
        position = list(parent).index(node) + 1
        for language, catalog in catalogs.items():
            text = catalog.gettext(source)
            if language != 'en' and text != source:
                localized = copy.deepcopy(node)
                localized.set(LANG, language)
                localized.text = text
                parent.insert(position, localized)
                position += 1
