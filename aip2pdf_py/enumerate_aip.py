import requests
from bs4 import BeautifulSoup, Tag
import urllib.parse
import os 
import base64
import pyx


from enumerate_helpers import check_for_refresh_redirect, sanitize_for_path, convert_to_jpeg_inline, files_with_extension, iterable_to_pairs

indent_count = 0

# fake user agent, with the original user agent header of python requests the dfs won't answer! They think they are clever or what???
user_agent_header = {"User-Agent" : "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0"}

def get_soup_resolve_redirects(url : str) :
    """ get BeautifulSoup object from url. Resolve redirects before return.
    returns (url, soup), i.e the final url along with the soup. 
    (well, this is not completely true, it does so for the kind of redirects used in the AIP.
    Must be enhanced when needed.)
    """
    response = requests.get(url, allow_redirects=False, headers=user_agent_header)
    while True:
        soup = BeautifulSoup(response.text, 'html.parser')
        refresh = check_for_refresh_redirect(soup)
        if refresh and refresh[1] :
            absrefresh = urllib.parse.urljoin(aip_root, refresh[1])
            print(f"redirected from {url} to {absrefresh}")
            url = absrefresh
            response = requests.get(absrefresh, allow_redirects=True, headers=user_agent_header)
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

def get_bytes_from_aip_img(tag : Tag) -> bytes :
    """ get a bytes object from an aip image. (inline, base64 encoded png)"""
    if tag.name != "img" : raise Exception("expected <img>")
    src = tag['src']
    scrparts1 = src.split(';')
    if scrparts1[0] != 'data:image/png':
        raise Exception(f"unexpected src type")
    scrparts2 = scrparts1[1].split(',')
    if scrparts2[0] != 'base64':
        raise Exception(f"unexpected src coding")
    return base64.b64decode(scrparts2[1])

def assemble_aip_images_to_pdf(image_folder : str):
    """ collect jpg images in image_folder to pdf. 
     Place 2 images per page side by side, page in landscape.
     If there is a single last image, put it full size in portrait on
     the last page
     Approach carts come first, than the text pages. 
     For the usual 3 piece AIP-VFR entry (2 app, 1 text), this results
     in a nicely foldable approach chart, especially if printed 2 sided. 
     For larger airports, the pdf is likely crap. For now, you have to 
     assemble the jpgs manually than.
    """
    pages = []

    img_files = list(files_with_extension(image_folder, '.jpg'))
    if len(img_files) == 0:
        # no pdf for empty (e.g. intermediate) folders
        return

    def DirEntryName(entry):
        return entry.name

    # order files so that the carts come first.(its either ED or ET, normally... )
    reordered = sorted( [f for f in img_files if f.name.startswith('ED')], key=DirEntryName )
    reordered += sorted( [f for f in img_files if f.name.startswith('ET')], key=DirEntryName )
    reordered += sorted( [f for f in img_files if f.name.startswith('AD 2')], key=DirEntryName)

    for jpg_file_entries_pair in iterable_to_pairs( reordered ):
        twopage = len(jpg_file_entries_pair) > 1
        horz_offs = 0.2 if twopage else 0.0
        canvas = pyx.canvas.canvas()
        # add first image. 
        bitmap_image = pyx.bitmap.jpegimage(jpg_file_entries_pair[0].path)
        bitmap = pyx.bitmap.bitmap(0, 1, bitmap_image, height= 1.0, compressmode = None)
        canvas.insert(bitmap )
        if twopage :
            # add 2nd bitmap, left of the middle. 
            bitmap_image = pyx.bitmap.jpegimage(jpg_file_entries_pair[1].path)
            bitmap = pyx.bitmap.bitmap(0.5 + horz_offs, 1, bitmap_image, height= 1.0, compressmode = None)
            canvas.insert(bitmap )
        if twopage:
            # add canvas with 2 images in landscape direction
            page = pyx.document.page(canvas, paperformat=pyx.document.paperformat.A4, rotated = 1, fittosize=1, margin = 0)
        else:
            # add canvas with single image in 'portrait' direction.
            page = pyx.document.page(canvas, paperformat=pyx.document.paperformat.A4, rotated = 0, fittosize=1, margin = 0)
        pages.append(page)
    document = pyx.document.document(pages=pages)
    document.writePDFfile(os.path.join(image_folder, os.path.split(image_folder)[1] + ".pdf"))


def download_document_item(url, target_folder, document_name, document_rel_url):
    abs_url = urllib.parse.urljoin(url, document_rel_url)
    file_name = os.path.join( target_folder, sanitize_for_path(document_name) + ".jpg")
    # the 'marker' file shall contain the relative url. 
    # This should change when the document is changed, so download can be skipped for unchanged items 
    marker_file_name = os.path.join( target_folder, sanitize_for_path(document_name) + ".marker")
    # check marker file: 
    if os.path.exists(marker_file_name) : 
        with open(marker_file_name, "r") as f : 
            prev_rurl = f.read().strip()
        if prev_rurl == document_rel_url:
            print(f"{' ' * indent_count}<DOC>{document_name} unchanged, skip")
            return
    print(f"{' ' * indent_count}<DOC>{document_name} is changed, download")

    try:
        response = requests.get(abs_url, allow_redirects=True, headers=user_agent_header)

        # extract the image into the target folder, converted to jpeg.
        # it is in an <img> tag, base64 encoded. 
        document_soup = BeautifulSoup(response.text, 'html.parser')
        img_tag = document_soup.find('img', class_ = 'pageImage', id = 'imgAIP')
        if img_tag :
            png_bytes = get_bytes_from_aip_img(img_tag)
            # convert to jpg, as pyx library can only handle jpg.
            jpg_bytes = convert_to_jpeg_inline(png_bytes)
            with open(file_name, "bw") as f:
                f.write(jpg_bytes)
            # write document_rel_url into marker file.
            with open(marker_file_name, "w") as f:
                f.write(document_rel_url)
        else:
            raise Exception(f"img tag not found")
    except BaseException as be : 
        be.add_note(f"when reading document data from {abs_url}")
        raise be



def recurse_aip(url : str, target_folder: str):
    try:
        global indent_count
        url, soup = get_soup_resolve_redirects(url)
        indent_count += 1

        os.makedirs(target_folder, exist_ok=True)

        # are there any documents in the current folder? -> download and store them
        documents = get_decode_aip_document_items(soup)
        for document_name, document_rel_url in documents:
            download_document_item(url, target_folder, document_name, document_rel_url)
        # build pdf from image files in folder.
        assemble_aip_images_to_pdf(target_folder)

        # are there any folders in the current folder? -> recurse.
        folders = get_decode_aip_folder_items(soup)
        for folder_name, folder_rel_url in folders:
            print(f"{' ' * indent_count}<FOL>{folder_name}")
            abs_url = urllib.parse.urljoin(url, folder_rel_url)
            folder_path = os.path.join(target_folder, sanitize_for_path( folder_name))
            recurse_aip(abs_url, folder_path)
        indent_count -= 1
    except BaseException as be:
        be.add_note(f"when processing {url}")
        raise be



if __name__ == '__main__':
    # this is the 'permalink' of the root page. 
    #aip_root = 'https://aip.dfs.de/BasicVFR/pages/C00001.html'

    # use another entry point for testing: 
    #aip_root = 'https://aip.dfs.de/BasicVFR/pages/C00064.html'
    #aip_root ='https://aip.dfs.de/BasicVFR/pages/C0005E.html'

    # root of the "AD" section.
    aip_root =  'https://aip.dfs.de/BasicVFR/pages/C0004A.html'

    recurse_aip(aip_root, "./downloads/aip_tracking")