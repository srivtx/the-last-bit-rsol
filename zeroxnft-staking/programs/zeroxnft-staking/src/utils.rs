use anchor_lang::prelude::*;
use mpl_core::types::Attribute;

use crate::constants::{STAKED_COUNT_KEY, STAKED_KEY, STAKED_TIME_KEY};
use crate::error::StakingError;

pub fn parse_i64_attr(value: &str) -> Result<i64> {
    value
        .parse::<i64>()
        .map_err(|_| StakingError::InvalidTimestamp.into())
}

pub fn is_staked(staked_value: &str) -> bool {
    staked_value != "0"
}

pub fn accrue_elapsed(staked_value: &str, now: i64) -> Result<i64> {
    let start = parse_i64_attr(staked_value)?;
    now.checked_sub(start).ok_or(StakingError::Underflow.into())
}

pub fn build_stake_start_attributes(existing: &[Attribute], now: i64) -> Result<Vec<Attribute>> {
    let mut list: Vec<Attribute> = Vec::new();
    let mut has_staked = false;
    let mut has_staked_time = false;

    for attribute in existing {
        if attribute.key == STAKED_KEY {
            require!(!is_staked(&attribute.value), StakingError::AlreadyStaked);
            list.push(Attribute {
                key: STAKED_KEY.to_string(),
                value: now.to_string(),
            });
            has_staked = true;
        } else {
            if attribute.key == STAKED_TIME_KEY {
                has_staked_time = true;
            }
            list.push(attribute.clone());
        }
    }

    if existing.is_empty() {
        list.push(Attribute {
            key: STAKED_KEY.to_string(),
            value: now.to_string(),
        });
        list.push(Attribute {
            key: STAKED_TIME_KEY.to_string(),
            value: "0".to_string(),
        });
        return Ok(list);
    }

    if !has_staked {
        list.push(Attribute {
            key: STAKED_KEY.to_string(),
            value: now.to_string(),
        });
    }
    if !has_staked_time {
        list.push(Attribute {
            key: STAKED_TIME_KEY.to_string(),
            value: "0".to_string(),
        });
    }
    Ok(list)
}

pub fn build_unstake_attributes(existing: &[Attribute], now: i64) -> Result<Vec<Attribute>> {
    let mut attribute_list: Vec<Attribute> = Vec::new();
    let mut staked_time: i64 = 0;
    let mut saw_staked = false;

    for attribute in existing.iter() {
        match attribute.key.as_str() {
            STAKED_KEY => {
                require!(is_staked(&attribute.value), StakingError::NotStaked);
                let elapsed = accrue_elapsed(&attribute.value, now)?;
                staked_time = staked_time
                    .checked_add(elapsed)
                    .ok_or(StakingError::Overflow)?;
                attribute_list.push(Attribute {
                    key: STAKED_KEY.to_string(),
                    value: "0".to_string(),
                });
                saw_staked = true;
            }
            STAKED_TIME_KEY => {
                staked_time = staked_time
                    .checked_add(parse_i64_attr(&attribute.value)?)
                    .ok_or(StakingError::Overflow)?;
            }
            _ => attribute_list.push(attribute.clone()),
        }
    }

    require!(saw_staked, StakingError::StakingNotInitialized);

    if attribute_list.iter().any(|a| a.key == STAKED_TIME_KEY) {
        for attr in &mut attribute_list {
            if attr.key == STAKED_TIME_KEY {
                attr.value = staked_time.to_string();
            }
        }
    } else {
        attribute_list.push(Attribute {
            key: STAKED_TIME_KEY.to_string(),
            value: staked_time.to_string(),
        });
    }

    Ok(attribute_list)
}

pub fn build_claim_attributes(existing: &[Attribute], now: i64) -> Result<(Vec<Attribute>, i64)> {
    let mut attribute_list: Vec<Attribute> = Vec::new();
    let mut saw_staked = false;
    let mut staked_time_total: i64 = 0;
    let mut elapsed: i64 = 0;

    for attribute in existing.iter() {
        match attribute.key.as_str() {
            STAKED_KEY => {
                require!(is_staked(&attribute.value), StakingError::NotStaked);
                elapsed = accrue_elapsed(&attribute.value, now)?;
                attribute_list.push(Attribute {
                    key: STAKED_KEY.to_string(),
                    value: now.to_string(),
                });
                saw_staked = true;
            }
            STAKED_TIME_KEY => {
                staked_time_total = parse_i64_attr(&attribute.value)?;
                attribute_list.push(attribute.clone());
            }
            _ => attribute_list.push(attribute.clone()),
        }
    }

    require!(saw_staked, StakingError::StakingNotInitialized);
    require!(elapsed > 0, StakingError::NothingToClaim);

    let new_total = staked_time_total
        .checked_add(elapsed)
        .ok_or(StakingError::Overflow)?;

    if let Some(attr) = attribute_list
        .iter_mut()
        .find(|a| a.key == STAKED_TIME_KEY)
    {
        attr.value = new_total.to_string();
    } else {
        attribute_list.push(Attribute {
            key: STAKED_TIME_KEY.to_string(),
            value: new_total.to_string(),
        });
    }

    Ok((attribute_list, elapsed))
}

pub fn build_collection_count_attributes(existing: &[Attribute], delta: i64) -> Result<Vec<Attribute>> {
    let mut attribute_list: Vec<Attribute> = Vec::new();
    let mut updated = false;

    for attribute in existing {
        if attribute.key == STAKED_COUNT_KEY {
            let current = parse_i64_attr(&attribute.value)?;
            let next = current.checked_add(delta).ok_or(if delta < 0 {
                StakingError::Underflow
            } else {
                StakingError::Overflow
            })?;
            require!(next >= 0, StakingError::Underflow);
            attribute_list.push(Attribute {
                key: STAKED_COUNT_KEY.to_string(),
                value: next.to_string(),
            });
            updated = true;
        } else {
            attribute_list.push(attribute.clone());
        }
    }

    require!(updated, StakingError::StakedCountMissing);
    Ok(attribute_list)
}
