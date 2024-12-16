use std::time::{
    UNIX_EPOCH,
    Duration,
    SystemTime,
};

const AMIGA_EPOCH_OFFSET : Duration = Duration::from_secs(252460800);
const TICKS_PER_SECOND   : u32 = 50;
const SECONDS_PER_DAYS   : u32 = 24*60*60;
const SECONDS_PER_MINS   : u32 = 60;

pub fn date_triplet_from_system_time(date_time: &SystemTime) -> (u32, u32, u32) {
    match date_time.duration_since(UNIX_EPOCH + AMIGA_EPOCH_OFFSET) {
        Ok(duration) => {
            let seconds = duration.as_secs() as u32;
            let (days, seconds) = (
                seconds/SECONDS_PER_DAYS,
                seconds%SECONDS_PER_DAYS,
            );
            let (mins, seconds) = (
                seconds/SECONDS_PER_MINS,
                seconds%SECONDS_PER_MINS,
            );

            (days, mins, seconds*TICKS_PER_SECOND)
        },
        _ => {
            (0, 0, 0)
        }
    }
}

pub fn date_triplet_to_system_time(days: u32, mins: u32, ticks: u32) -> SystemTime {
    let seconds = ((days*24*60 + mins)*60 + ticks/TICKS_PER_SECOND) as u64;
    UNIX_EPOCH + AMIGA_EPOCH_OFFSET + Duration::from_secs(seconds)
}
