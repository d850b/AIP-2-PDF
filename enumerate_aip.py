import requests
from bs4 import BeautifulSoup, Tag
import urllib.parse
import os 
import base64

from enumerate_helpers import check_for_refresh_redirect


# this is the 'permalink' of the root page. 
#aip_root = 'https://aip.dfs.de/BasicVFR/pages/C00001.html'

# use another entry point for for testing: 
aip_root = 'https://aip.dfs.de/BasicVFR/pages/C00064.html'


def get_soup_resolve_redirects(url : str):
    response = requests.get(url, allow_redirects=True)
    while True:
        soup = BeautifulSoup(response.text, 'html.parser')
        refresh = check_for_refresh_redirect(soup)
        if refresh and refresh[1] :
            absrefresh = urllib.parse.urljoin(aip_root, refresh[1])
            print(f"redirected from {url} to {absrefresh}")
            url = absrefresh
            response = requests.get(absrefresh, allow_redirects=True)
        else:
            return url, soup
          
def get_decode_aip_folder_items(tag : Tag):
    items = tag.find_all("li", class_ = "folder-item")
    for item in items:
        folder_link_tag = item.find("a", class_ = "folder-link")
        folder_rel_url = folder_link_tag.attrs.get("href")
        folder_name_tag = item.find("span", class_= "folder-name", lang = "en")
        folder_name = folder_name_tag.text
        yield (folder_name, folder_rel_url)

def get_decode_aip_document_items(tag : Tag):
    items = tag.find_all("li", class_ = "document-item")
    for item in items:
        document_link_tag = item.find("a", class_ = "document-link")
        document_rel_url = document_link_tag.attrs.get("href")
        document_name_tag = item.find("span", class_= "document-name", lang = "en")
        document_name = document_name_tag.text
        yield (document_name, document_rel_url)

def sanitize_for_path(s: str):
    """ a little radical, but better save than sorry.. """
    return "".join((x if x.isalnum() or x == ' ' else '_' for x in s))


indent_count = 0

def recurse_aip(url : str, target_folder: str):
    global indent_count
    url, soup = get_soup_resolve_redirects(url)
    indent_count += 1

    os.makedirs(target_folder, exist_ok=False)

    # are there any documents in the current folder? -> download and store them
    documents = get_decode_aip_document_items(soup)
    for document_name, document_rel_url in documents:
        print(f"{' ' * indent_count}<DOC>{document_name}")
        abs_url = urllib.parse.urljoin(url, document_rel_url)
        response = requests.get(abs_url, allow_redirects=True)

        # FIRST save the complete html document into target folder.
        file_name = os.path.join( target_folder, sanitize_for_path(document_name))
        with open(file_name  + ".html", "w") as f:
            f.write(response.text)

        # SECOND: extract the png image into the target folder.
        # it is in an <img> tag, base64 encoded. 
        document_soup = BeautifulSoup(response.text, 'html.parser')
        img_tag = document_soup.find('img', class_ = 'pageImage', id = 'imgAIP')
        if img_tag :
            src = img_tag['src']
            scrparts1 = src.split(';')
            if scrparts1[0] != 'data:image/png':
                raise Exception(f"unexpected src type in {abs_url}")
            scrparts2 = scrparts1[1].split(',')
            if scrparts2[0] != 'base64':
                raise Exception(f"unexpected src coding in {abs_url}")
            with open(file_name  + ".PNG", "bw") as f:
                f.write(base64.b64decode(scrparts2[1]))
        else:
            raise Exception(f"img tag not found in {abs_url}")

    # are there any folders in the current folder? -> recurse.
    folders = get_decode_aip_folder_items(soup)
    for folder_name, folder_rel_url in folders:
        print(f"{' ' * indent_count}<FOL>{folder_name}")
        abs_url = urllib.parse.urljoin(url, folder_rel_url)
        folder_path = os.path.join(target_folder, sanitize_for_path( folder_name))
        recurse_aip(abs_url, folder_path)
    indent_count -= 1



recurse_aip(aip_root, "./downloads/aip_1")