use super::{BlockMembership, Schicht, ShiftMinutes, ShiftRelation, WindowMinutes};
use chrono::{Duration, NaiveDateTime};
use std::collections::{BTreeMap, BTreeSet};

pub fn minutes(shifts: &[Schicht]) -> Vec<ShiftMinutes> {
    let start = shifts.iter().map(|s| s.start).min();
    let end = shifts.iter().map(|s| s.ende).max();
    let limit = start
        .zip(end)
        .map_or(0, |(a, b)| (b - a).num_minutes() * 43 / 168);
    shifts
        .iter()
        .map(|s| ShiftMinutes {
            id: s.index,
            minutes: (s.ende - s.start).num_minutes(),
            average_limit: limit,
        })
        .collect()
}

pub fn relations(shifts: &[Schicht]) -> Vec<ShiftRelation> {
    let mut ordered: Vec<_> = shifts.iter().collect();
    ordered.sort_by_key(|s| (s.start, s.index));
    let mut result = Vec::new();
    for (i, a) in ordered.iter().enumerate() {
        for b in &ordered[i..] {
            if b.start >= a.ende + Duration::hours(42) {
                break;
            }
            let overlap = a.start < b.ende && b.start < a.ende;
            let rest =
                a.start < b.ende + Duration::hours(11) && b.start < a.ende + Duration::hours(11);
            let night_to_day = a.ende.date() > a.start.date()
                && b.ende.date() == b.start.date()
                && b.start >= a.ende
                && b.start < a.ende + Duration::hours(42);
            if overlap || rest || night_to_day {
                result.push(ShiftRelation {
                    id: result.len(),
                    first: a.index,
                    second: b.index,
                    overlap,
                    rest,
                    night_to_day,
                });
            }
        }
    }
    result
}

pub fn blocks(shifts: &[Schicht]) -> Vec<BlockMembership> {
    let mut stations: BTreeMap<&str, Vec<&Schicht>> = BTreeMap::new();
    for s in shifts {
        stations.entry(&s.wache).or_default().push(s);
    }
    let mut result = Vec::new();
    let mut block = 0;
    for station in stations.values() {
        let mut events: BTreeMap<NaiveDateTime, (Vec<usize>, Vec<usize>)> = BTreeMap::new();
        for s in station {
            events.entry(s.start).or_default().0.push(s.index);
            events.entry(s.ende).or_default().1.push(s.index);
        }
        let mut active = BTreeSet::new();
        let mut previous = None;
        for (time, (starts, ends)) in events {
            if previous.is_some_and(|p| p < time) && !active.is_empty() {
                for shift in &active {
                    result.push(BlockMembership {
                        id: result.len(),
                        shift: *shift,
                        block,
                    });
                }
                block += 1;
            }
            for shift in ends {
                active.remove(&shift);
            }
            for shift in starts {
                active.insert(shift);
            }
            previous = Some(time);
        }
    }
    result
}

pub fn windows(shifts: &[Schicht]) -> Vec<WindowMinutes> {
    let week = Duration::days(7);
    let mut ordered: Vec<_> = shifts.iter().collect();
    ordered.sort_by_key(|s| (s.start, s.index));
    let mut endpoints = BTreeSet::new();
    let mut prefix_ends = Vec::with_capacity(ordered.len());
    let mut maximum = NaiveDateTime::MIN;
    for s in &ordered {
        endpoints.extend([s.start, s.ende - week]);
        maximum = maximum.max(s.ende);
        prefix_ends.push(maximum);
    }
    let mut unique = BTreeSet::new();
    let mut result = Vec::new();
    let mut window = 0;
    for start in endpoints {
        let end = start + week;
        let lo = prefix_ends.partition_point(|t| *t <= start);
        let hi = ordered.partition_point(|s| s.start < end);
        let mut contributions: Vec<_> = ordered[lo..hi]
            .iter()
            .filter_map(|s| {
                let value = (s.ende.min(end) - s.start.max(start)).num_minutes();
                (value > 0).then_some((s.index, value))
            })
            .collect();
        if contributions.is_empty() {
            continue;
        }
        contributions.sort_unstable();
        if !unique.insert(contributions.clone()) {
            continue;
        }
        for (shift, minutes) in contributions {
            result.push(WindowMinutes {
                id: result.len(),
                shift,
                window,
                minutes,
            });
        }
        window += 1;
    }
    result
}
