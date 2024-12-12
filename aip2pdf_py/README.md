# AIP downloader

Program to extract approach charts into pdf documents from the German 
AIP-VFR.

The pdfs are formatted in a way that most of them can be directly
printed as an A5-foldable approach chart. This is true for the common
three-piece apprach charts for small airports. 

For IFR airports and perhaps some VFR fields the pdf is currently 
unusable. (Working on it...) In this case you can still create your own documents from 
the downloaded images using LibreOffice, OpenOffice, MS-Word or whatever 
program works for you.

## how it works:

The German AIP-VFR is organized in a tree-structure, resembling the chapters
of the AIP.

The AIP website is quite regularly organized and easy to parse. All
relevant tags are decorated with distinct css and other attributes and 
thus simple to address.

There are "folder-links", which point to a sub-page, and there are 
"document-links", which point to html documents containing the final
data. 

This data is always a PNG image with the chart or text. 

The program iterates the tree, creates the respective folders in 
the file system and, for the documents, downloads the image files. 
Due to a restriction in the next step, the PNG files are converted 
to JPEG before saving them.

Then, it assembles the images into a PDF document per folder. 
For most small airports, this is usable as a neatly foldable A5 approach chart.
For some (ifr and e.g. EDHE) it will currently be crap.  


The AIP website uses the concept or "permalinks", i.e. known links which do 
not change between AIP cycles. These links are addressing small "stub" 
pages which redirect into the "real" tree (with monthly changing adresses.)
The redirect mechanism might be a bit unusual, it is handled be a "reload" instruction. 

This redirect is recognized and handled by the program too, so you can use
any permalink as entry point for the scan.