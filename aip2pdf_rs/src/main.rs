
use reqwest::{Client, Url};
use scraper::{selectable::Selectable, Html, Selector, ElementRef};

mod helpers;
use helpers::{Aip2PdfError, ErrorType};
use tokio::select;


const AIP_ROOT : &str = "https://aip.dfs.de/BasicVFR/pages/C00001.html";
//const AIP_ROOT : &str = "https://aip.dfs.de/BasicVFR/pages/C0005E.html";


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
        //println!("{:?}", element);
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

fn get_decode_aip_document_items<'a, S : Selectable<'a> + 'a>(selectable : S, selectors : &'a AllSelectors) -> Result< impl Iterator<Item = (String, String)> + 'a, ErrorType>{
    let it = selectable.select(&selectors.select_document_item);
    let it2 = it.map(|document_item_element| {
        let mut href_str = String::new();
        let mut name_str = String::new();
        if let Some(document_link_element) = document_item_element.select(&selectors.select_document_link).next(){
            if let Some(href) = document_link_element.value().attr("href"){
                href_str = href.into();
            }
            if let Some(folder_name_element) = document_link_element.select(&selectors.select_document_name).next(){
                if let Some(folder_name)  =  folder_name_element.text().next(){
                    name_str = folder_name.into();
                }
            }
        }
        (href_str, name_str)
    });
    Ok( it2)
}

async fn recurse_aip(selectors: &AllSelectors, url : Url, target_folder : &str, recurse_level : i32) -> Result<(), ErrorType> {

    let (final_url, document) = get_document_resolve_redirects( url ).await?;

    for (href, name) in get_decode_aip_document_items(&document, &selectors)?{
        let spacer = " ".repeat(recurse_level as usize);
        println!("D{}{}", spacer, name);

    }

    for (href, name)  in get_decode_aip_folder_items(&document, &selectors)? {
        let recurse_url = final_url.join(&href)?;
        let spacer = " ".repeat(recurse_level as usize);
        //println!("{}{:?} {:?}, {:?}", spacer, href, name, recurse_url);
        println!("F{}{}", spacer, name);
        // some magic to allow to recurse async... 
        Box::pin(recurse_aip(selectors, recurse_url, target_folder, recurse_level + 1)).await?;
        //recurse_aip(selectors, recurse_url, target_folder, recurse_level + 1).await?;
    }

    Ok(())
}



#[tokio::main]
async fn main() -> Result<(), ErrorType> {
    // initialize selectors
    let selectors = AllSelectors::new()?;

    //let (_url, document) = get_document_resolve_redirects ( Url::parse(AIP_ROOT)?).await?;


    //println!("{}", document.html());
    //get_decode_aip_folder_items__test_selection( &document)?;

    // for x  in get_decode_aip_folder_items__test_iterator(&document, &selectors)? {
    //     println!("{:?}", x)
    // }

    // for x  in get_decode_aip_folder_items(&document, &selectors)? {
    //     println!("{:?}", x)
    // }

    recurse_aip(&selectors, Url::parse(AIP_ROOT)?, "", 0).await?;

    Ok(())
}





/// Keep all selectors. 
/// First: for effiency, they can be reused and are probably expansive to make
/// Second: to satisfy some lifetime expectations. (Rust is complicated... ) 
struct AllSelectors{
    pub select_folder_item : Selector,
    pub select_folder_link  : Selector,
    pub select_folder_name  : Selector,

    pub select_document_item : Selector,
    pub select_document_link : Selector,
    pub select_document_name : Selector,
}

impl AllSelectors {
    fn new() -> Result<Self, ErrorType> {
        Ok(
            Self { 
                select_folder_item : Selector::parse(r#"li[class="folder-item"]"#)?,
                select_folder_link : Selector::parse(r#"a[class="folder-link"]"#)?, 
                select_folder_name : Selector::parse(r#"span[class="folder-name"][lang="en"]"#)?,

                select_document_item : Selector::parse(r#"li[class="document-item"]"#)?,
                select_document_link : Selector::parse(r#"a[class="document-link"]"#)?,
                select_document_name : Selector::parse(r#"span[class="document-name"][lang="en"]"#)?
            }
        )
    }
}
