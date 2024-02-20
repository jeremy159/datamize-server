use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit0},
    combinator::{map_res, opt},
    error::Error,
    number::complete::double,
    sequence::{preceded, separated_pair},
    IResult,
};

fn parse_number_with_space(input: &str) -> IResult<&str, f64> {
    map_res(
        separated_pair(digit0::<&str, nom::error::Error<&str>>, char(' '), double),
        |(left, right)| {
            let mut input = left.to_owned();
            input.push_str(right.to_string().as_str());
            double::<&str, nom::error::Error<&str>>(input.as_str())
                .map(|res| res.1)
                .map_err(|err| err.map(|err: Error<_>| Error::new(err.input.to_string(), err.code)))
        },
    )(input)
}

fn parse_number_with_comma(input: &str) -> IResult<&str, f64> {
    map_res(
        separated_pair(digit0::<&str, nom::error::Error<&str>>, char(','), double),
        |(left, right)| {
            let mut input = left.to_owned();
            input.push_str(right.to_string().as_str());
            double::<&str, nom::error::Error<&str>>(input.as_str())
                .map(|res| res.1)
                .map_err(|err| err.map(|err: Error<_>| Error::new(err.input.to_string(), err.code)))
        },
    )(input)
}

fn float_parser(input: &str) -> IResult<&str, f64> {
    alt((parse_number_with_space, parse_number_with_comma, double))(input)
}

/// Parses inpute string as a balance.
/// Returns the amount in milliunits format.
pub fn parse_balance(input: &str) -> anyhow::Result<i64> {
    let (_, balance) = preceded(opt(tag("$")), float_parser)(input)
        .map_err(|err| err.map(|err: Error<_>| Error::new(err.input.to_string(), err.code)))?;
    Ok((balance * 1000_f64).ceil() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        /// Optionnal leading dollar sign with optionnal commas
        /// ldscs = Leading Dollar Sign Comma Separated
        fn parse_valid_amounts_ldscs(a in r#"(\$)?(([0-9]+)|([0-9]{1,3})(\,[0-9]{3})*)(\.[0-9]{1,2})?"#) {
            parse_balance(&a).unwrap();
        }

        #[test]
        /// Optionnal leading dollar sign with optionnal spaces
        /// ldsss = Leading Dollar Sign Space Separated
        fn parse_valid_amounts_ldsss(a in r#"(\$)?(([0-9]+)|([0-9]{1,3})(\s[0-9]{3})*)(\.[0-9]{1,2})?"#) {
            parse_balance(&a).unwrap();
        }

        #[test]
        /// Optionnal Ending dollar sign with optionnal commas
        /// edscs = Ending Dollar Sign Comma Separated
        fn parse_valid_amounts_edscs(a in r#"(([0-9]+)|([0-9]{1,3})(\,[0-9]{3})*)(\.[0-9]{1,2})?(\$)?"#) {
            parse_balance(&a).unwrap();
        }

        #[test]
        /// Optionnal Ending dollar sign with optionnal spaces
        /// edsss = Ending Dollar Sign Space Separated
        fn parse_valid_amounts_edsss(a in r#"(([0-9]+)|([0-9]{1,3})(\s[0-9]{3})*)(\.[0-9]{1,2})?(\$)?"#) {
            parse_balance(&a).unwrap();
        }
    }
}
