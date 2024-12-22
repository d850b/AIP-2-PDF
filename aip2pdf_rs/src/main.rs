
use reqwest::{Client, Url};
use scraper::{selectable::Selectable, Html, Selector, ElementRef};

mod helpers;
use helpers::{Aip2PdfError, ErrorType};
use tokio::select;


//const aip_root : &str = "https://aip.dfs.de/BasicVFR/pages/C00001.html";
const AIP_ROOT : &str = "https://aip.dfs.de/BasicVFR/pages/C0005E.html";


async fn get_document_resolve_redirects(url: reqwest::Url) -> Result<(reqwest::Url, Html), ErrorType>{
    let client = Client::new();
    let mut abs_get_url = url.clone();
    let mut response = client.get(abs_get_url).send().await?;
    // limit number or refresh-redirects to 5, in practice it should be only 1.
    for _ in 0..5 {
        // url to be returned, might be different from url used in get due to redirects?
        // anyway, abs_get_url is consumed by get. Was this the reason?
        let final_url = response.url().clone();
        let document = Html::parse_document(&response.text().await?);
        // check for refresh-redirect.
        if let Some( (_refresh_time, refresh_url)) = check_for_refresh_redirects( &document)?{
            // follow refresh url, but it might be relative, so use join.
            abs_get_url = url.join(refresh_url)?;
            response = client.get(abs_get_url).send().await?;
        } else {
            // no refresh found, return current document.
            return  Ok((final_url, document));
        }
    }
    Err ( Aip2PdfError::boxed("more than 5 redirects"))
}


fn check_for_refresh_redirects(document : & Html) -> Result<Option<(i32, &str)>, ErrorType>{
    // selects meta element with refresh information in it.
    let select_meta_refresh = Selector::parse(r#"meta[http-equiv="refresh"]"#)?;

    if let Some(element) = document.select(&select_meta_refresh).next() {
        println!("{:?}", element);
        if let Some(content) = element.value().attr("content"){
            let split1 : Vec<&str> = content.split(";").collect();
            if split1.len() == 2 {
                let refresh_time : i32 = split1[0].parse()?;
                let split2 : Vec<&str> = split1[1].split("=").collect();
                if split2.len() == 2 {
                    Ok( Some((refresh_time, split2[1])))
                } else {
                    Err(Aip2PdfError::boxed("too many parts in split2"))
                }
            } else {
                Err(Aip2PdfError::boxed("too many parts in split1"))
            }
        }
        else {
            Err(Aip2PdfError::boxed("no content attribute"))
        }
    } else {
        Ok(None)
    }
}

// The original pyhton function:
//
// def get_decode_aip_folder_items(tag : Tag):
//     items = tag.find_all("li", class_ = "folder-item")
//     for item in items:
//         folder_link_tag = item.find("a", class_ = "folder-link")
//         folder_rel_url = folder_link_tag.attrs.get("href")
//         folder_name_tag = item.find("span", class_= "folder-name", lang = "en")
//         folder_name = folder_name_tag.text
//         yield (folder_name, folder_rel_url)


fn get_decode_aip_folder_items__test_selection<'a, S : Selectable<'a>>(selectable : S) -> Result<(), ErrorType>{
    // Hm. Generating the selectors anew for every call looks like overhead, right?
    let select_folder_item = Selector::parse(r#"li[class="folder-item"]"#)?;
    let select_folder_link = Selector::parse(r#"a[class="folder-link"]"#)?;
    let select_folder_name = Selector::parse(r#"span[class="folder-name"][lang="en"]"#)?;
    for folder_item_element in selectable.select(&select_folder_item){
        //println!("{:?}", folder_item_element.html());
        if let Some(folder_link_element) = folder_item_element.select(&select_folder_link).next(){
            //println!("{:?}", folder_link_element.html());
            if let Some(href) = folder_link_element.value().attr("href"){
                println!("href = {}", href);
            }
            if let Some(folder_name_element) = folder_link_element.select(&select_folder_name).next(){
                if let Some(folder_name)  =  folder_name_element.text().map(|n| n).next(){
                    println!("name = {}", folder_name)
                }
            }
        }
    }
    Ok(())
}

/// how the heck do i learn where to place all those lifetime parameters... 
fn get_decode_aip_folder_items__test_iterator<'a, S : Selectable<'a> + 'a>(selectable : S, selectors : &'a AllSelectors) -> Result< Box<dyn Iterator<Item = ElementRef<'a>> + 'a>, ErrorType>{
    let it = selectable.select(&selectors.select_folder_item);

    Ok( Box::new(it))

}

/// how the heck do i learn where to place all those lifetime parameters... 
fn get_decode_aip_folder_items<'a, S : Selectable<'a> + 'a>(selectable : S, selectors : &'a AllSelectors) -> Result< impl Iterator<Item = (String, String)> + 'a, ErrorType>{
    let it = selectable.select(&selectors.select_folder_item);
    let it2 = it.map(|folder_item_element| {
        let mut href_str = String::new();
        let mut name_str = String::new();
        if let Some(folder_link_element) = folder_item_element.select(&selectors.select_folder_link).next(){
            if let Some(href) = folder_link_element.value().attr("href"){
                href_str = href.into();
            }
            if let Some(folder_name_element) = folder_link_element.select(&selectors.select_folder_name).next(){
                if let Some(folder_name)  =  folder_name_element.text().next(){
                    name_str = folder_name.into();
                }
            }
        }
        (href_str, name_str)
    });
    Ok( it2)
}





/// Keep all selectors. 
/// First: for effiency, they can be reused and are probably expansive to make
/// Second: to satisfy some lifetime expectations. (Rust is complicated... ) 
struct AllSelectors{
    pub select_folder_item : Selector,
    pub select_folder_link  : Selector,
    pub select_folder_name  : Selector,
}

impl AllSelectors {
    fn new() -> Result<Self, ErrorType> {
        Ok(
            Self { 
                select_folder_item : Selector::parse(r#"li[class="folder-item"]"#)?,
                select_folder_link :Selector::parse(r#"a[class="folder-link"]"#)?, 
                select_folder_name : Selector::parse(r#"span[class="folder-name"][lang="en"]"#)?
            }
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), ErrorType> {

    let (_url, document) = get_document_resolve_redirects ( Url::parse(AIP_ROOT)?).await?;

    let selectors = AllSelectors::new()?;

    //println!("{}", document.html());
    //get_decode_aip_folder_items__test_selection( &document)?;

    // for x  in get_decode_aip_folder_items__test_iterator(&document, &selectors)? {
    //     println!("{:?}", x)
    // }

    for x  in get_decode_aip_folder_items(&document, &selectors)? {
        println!("{:?}", x)
    }

    Ok(())
}

