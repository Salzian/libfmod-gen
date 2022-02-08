use crate::models::{Error, ErrorStringMapping};
use crate::repr::JsonConverter;
use pest::{error, Parser};

#[derive(Parser)]
#[grammar = "./grammars/fmod_errors.pest"]
struct FmodErrorsParser;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Header {
    pub mapping: ErrorStringMapping,
}

pub fn parse(source: &str) -> Result<Header, Error> {
    let declarations = FmodErrorsParser::parse(Rule::api, source)?
        .next()
        .ok_or(Error::FileMalformed)?;

    let arrays = vec![String::from("errors")];
    let converter = JsonConverter::new(arrays);

    let mut header = Header::default();
    for declaration in declarations.into_inner() {
        match declaration.as_rule() {
            Rule::ErrorStringMapping => header.mapping = converter.convert(declaration)?,
            _ => continue,
        }
    }

    Ok(header)
}

impl From<error::Error<Rule>> for Error {
    fn from(error: error::Error<Rule>) -> Self {
        Self::Pest(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::fmod_errors::{parse, Header};
    use crate::models::{ErrorString, ErrorStringMapping};

    #[test]
    fn test_should_ignore_gnuc() {
        let source = r#"
            #ifdef __GNUC__ 
            static const char *FMOD_ErrorString(FMOD_RESULT errcode) __attribute__((unused));
            #endif
        "#;
        assert_eq!(parse(source), Ok(Header::default()))
    }

    #[test]
    fn test_should_parse_error_string_mapping() {
        let source = r#"
            static const char *FMOD_ErrorString(FMOD_RESULT errcode)
            {
                switch (errcode)
                {
                    case FMOD_OK:                            return "No errors.";
                    case FMOD_ERR_TOOMANYSAMPLES:            return "The length provided exceeds the allowable limit.";
                    default :                                return "Unknown error.";
                };
            }
        "#;
        assert_eq!(
            parse(source),
            Ok(Header {
                mapping: ErrorStringMapping {
                    errors: vec![
                        ErrorString {
                            name: "FMOD_OK".into(),
                            string: "No errors.".into()
                        },
                        ErrorString {
                            name: "FMOD_ERR_TOOMANYSAMPLES".into(),
                            string: "The length provided exceeds the allowable limit.".into()
                        }
                    ]
                }
            })
        )
    }
}
