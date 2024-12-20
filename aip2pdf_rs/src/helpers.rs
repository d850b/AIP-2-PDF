
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

// impl From<Aip2PdfError> for Box< std::error::Error>{
//     fn from(error : Aip2PdfError) -> Self {
//         Box::new(error)
//     }
// }

// impl Into<Box<dyn std::error::Error>> for Aip2PdfError {
//     fn into(self) -> Box<dyn std::error::Error> {
//         Box::new( self)
//     }
// }