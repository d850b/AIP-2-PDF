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

