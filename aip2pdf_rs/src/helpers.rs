
/// Convenience shorthand for the commonly used error type in Result, used througout this crate
pub type ErrorType = Box<dyn std::error::Error>;


#[derive(Debug)]
pub struct Aip2PdfError{
    message : String,
}

impl Aip2PdfError {
    pub fn new(message : &str) -> Self {
        Self { 
            message : String::from(message),
        }
    }

    /// convenience: returns Aip2PdfError in Box<dyn std::error::Error>. 
    /// (Is this really the way to do this? I doubt it. But 
    /// i cannot implement "From" or "Into" for this... )
    pub fn boxed(message : &str) -> Box<dyn std::error::Error>{
        Box::new(Self::new(message))
    }
}

impl std::fmt::Display for Aip2PdfError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Aip2PdfError {    
}


// def sanitize_for_path(s: str):
//     """ make str useable as directore/file name.
//       a little radical, but better save than sorry.. """
//     return "".join((x if x.isalnum() or x == ' ' else '_' for x in s))

pub fn sanitize_for_path(s : & str) -> String {
    s.chars().map(|c| if c.is_alphanumeric() & (c != ' ') {c} else {'_'} ).collect()
}