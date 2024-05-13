use gargoyle::{Notify, Schedule};
use gargoyle_feed_monitor::WebFeedUpdate;
use std::{thread::sleep, time::Duration};

fn main() {
    let stdout_notifier = Stdout;
    let mut rss_monitor = WebFeedUpdate::new("http://lorem-rss.herokuapp.com/feed?unit=second&interval=10");
    let mut schedule = Schedule::default();
    schedule.add(
        &format!("An update from the feed"),
        &format!("This should not run, if you get this, this is an error."),
        Duration::from_secs(5),
        &mut rss_monitor,
        &stdout_notifier,
    );

    loop {
        schedule.run();
        sleep(Duration::from_millis(100));
    }
}

struct Stdout;
impl Notify for Stdout {
    fn send(&self, msg: &str, diagnostic: Option<String>) -> Result<(), String> {
        if let Some(diagnostic) = diagnostic {
            println!("{msg}:\n{diagnostic}");
        } else {
            println!("{msg}");
        }

        Ok(())
    }
}

