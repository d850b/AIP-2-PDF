""" some (possibly stupid) helpers """
import os
import requests
from bs4 import BeautifulSoup, Tag


def check_for_refresh_redirect(soup: BeautifulSoup):
    """ there are other ways do request a redirect... This site uses a form of 'refresh' 
    which is not handled by BeautifulSoup.. Do it manually. Tedious...
    returns (refresh_time, refresh__url) or None if no tefresh tag is 
    if no refresh_url is found, it is an empty string.
    """
    refresh_time = -1
    refresh_url = ""
    # do we have a meta tag with http-equiv=Refresh ? 
    refresh_meta = soup.find("meta", attrs={"http-equiv":"Refresh"})
    if refresh_meta is None :
        return None
    # yup. Get the "content" attribute
    ct = refresh_meta.attrs.get('content')
    if ct is None :
        raise Exception("missing content attribute")

    refresh_partno = 0
    for refresh_part in ct.split(';'):
        if refresh_partno == 0 : 
            refresh_time = int(refresh_part)
        elif refresh_partno == 1:
            url_parts = refresh_part.split("=")
            refresh_url = url_parts[1]
        else:
            raise Exception(f"too many parts in content attribute: {ct}")
        refresh_partno += 1 
    return (refresh_time, refresh_url)

def sanitize_for_path(s: str):
    """ make str useable as directore/file name.
      a little radical, but better save than sorry.. """
    return "".join((x if x.isalnum() or x == ' ' else '_' for x in s))


def iterable_to_pairs ( x ):
    """ return pairs of elements in iterable x. 
    I know, there are itertools, but i can't wrap my head around this right now"""
    it = x.__iter__()
    while True:
        elem1 = None
        elem2 = None
        try:
            elem1 = it.__next__()
            try:
                elem2 = it.__next__()
                yield (elem1, elem2)
            except StopIteration:
                yield (elem1,)
        except StopIteration:
            return

def files_with_extension(path : str, extension: str):
    """ search directory for files with extension. Search is cas insensitive. Give extension with the dot, i.e. '.jpg'
    -> iterator(os.DireEntry) 
    yes, there is the glob module, but i want to be case 
    insensitive for the extension."""
    extension = extension.upper()
    with os.scandir( path ) as it:
        for entry in it:
            if entry.is_file() : 
                entry_parts = os.path.splitext(entry.path)
                if entry_parts[1] and entry_parts[1].upper() == extension:
                    yield entry
