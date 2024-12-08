# AIP downloader

This simple program iterates over the German AIP-VFR website and
places its contents into a folder tree, resembling the chapters
of the AIP.

The AIP website is quite regularly organized and easy to parse. All
relevant tags are decorated with distinct css and other attributes and thus simple to address.

The website uses the concept or "permalinks", i.e. non links which do 
not change between AIP cycles. These links are addressing small "stub" pages which redirect into the "real" tree (with monthly changing adresses.)
The redirect mechanism might be a bit unusual, it is handled be a "reload" instruction. 

This redirect must be (IMHO) 'manually' handled. This happens in "enumerate_helpers.check_for_refresh_redirect()"  