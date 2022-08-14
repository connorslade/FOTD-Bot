use chrono::prelude::*;

pub struct SendTime {
    hour: u32,
    minute: u32,
}

impl SendTime {
    pub fn from_str(time: &str) -> Self {
        let time_parts: Vec<&str> = time.split(':').collect();
        SendTime {
            hour: time_parts[0].parse::<u32>().expect("Invalid Send Hour"),
            minute: time_parts[1].parse::<u32>().expect("Invalid Send Minute"),
        }
    }

    pub fn is_time(&self) -> bool {
        let now = Local::now();
        now.hour() == self.hour && now.minute() == self.minute
    }
}
