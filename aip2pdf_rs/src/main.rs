
use reqwest::{Client, Url};
use scraper::{Html, Selector};

mod helpers;
use helpers::{Aip2PdfError, ErrorType};


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

 

#[tokio::main]
async fn main() -> Result<(), ErrorType> {

    let (_url, document) = get_document_resolve_redirects ( Url::parse(AIP_ROOT)?).await?;
    println!("{}", document.html());

    Ok(())
}


