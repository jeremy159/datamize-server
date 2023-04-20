use nom::{
    bytes::complete::tag, combinator::opt, error::Error, number::complete::double,
    sequence::preceded, IResult,
};

fn float_parser(input: &str) -> IResult<&str, f64> {
    double(input)
}

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
        assert_eq!(parse_balance("$1.00").unwrap(), 1000);
        assert_eq!(parse_balance("$0.00").unwrap(), 0);
        assert_eq!(parse_balance("$0.01").unwrap(), 10);
        assert_eq!(parse_balance("$0.10").unwrap(), 100);
        assert_eq!(parse_balance("$0.99").unwrap(), 990);
        assert_eq!(parse_balance("$1.99").unwrap(), 1990);
        assert_eq!(parse_balance("$2.00").unwrap(), 2000);
        assert_eq!(parse_balance("$2.01").unwrap(), 2010);
        assert_eq!(parse_balance("$2.99").unwrap(), 2990);
        assert_eq!(parse_balance("$3124.27").unwrap(), 3124270);
        assert_eq!(parse_balance("200.27").unwrap(), 200270);
        assert_eq!(parse_balance("654.2$").unwrap(), 654200);
        // assert_eq!(parse_balance("7 524.5").unwrap(), 7524500); FIXME: To make it pass
        // assert_eq!(parse_balance("5,847.56").unwrap(), 5847560); FIXME: To make it pass
    }
}
