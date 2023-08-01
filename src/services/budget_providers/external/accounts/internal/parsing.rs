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

    #[test]
    fn test_parse_balance() {
        let tests = [
            ("$1.00", 1000),
            ("$0.00", 0),
            ("$0.01", 10),
            ("$0.10", 100),
            ("$0.99", 990),
            ("$1.99", 1990),
            ("1.00$", 1000),
            ("0.00$", 0),
            ("0.01$", 10),
            ("0.10$", 100),
            ("0.99$", 990),
            ("1.99$", 1990),
            ("$2.00", 2000),
            ("$2.01", 2010),
            ("$2.99", 2990),
            ("$3124.27", 3124270),
            ("200.27", 200270),
            ("654.2$", 654200),
            ("8 428", 8428000),
            ("8 428$", 8428000),
            ("7 524.5", 7524500),
            ("5,847.56", 5847560),
        ];

        for (input, expected) in tests {
            assert_eq!(parse_balance(input).unwrap(), expected);
        }
    }
}
