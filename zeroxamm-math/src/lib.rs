pub fn get_amount_out(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
) -> Option<u64> {
    if (amount_in == 0 || reserve_in == 0 || reserve_out == 0) {
        return None;
    }
    
    let amount_in = amount_in as u128;
    let reserve_in = reserve_in as u128;
    let reserve_out = reserve_out as u128;

    let numerator = amount_in.checked_mul(reserve_out)?;
    let denominator = reserve_in.checked_add(amount_in)?;
    let out = numerator.checked_div(denominator)?;

    Some(out as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]

    fn hundred_in_thousand_pool_gives_ninety(){
        let out = get_amount_out(100 , 1000 , 1000).unwrap() ; 
        assert_eq!(out , 90) ; 
    }

    #[test]
    fn ten_in_hundred_pool_gives_nine(){
        let out = get_amount_out(10 , 100 , 100).unwrap() ; 
        assert_eq!(out , 9) ; 
    }
}
