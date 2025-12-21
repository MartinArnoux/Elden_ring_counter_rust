pub struct Recorder {
    title: String,
    counter: u128,
    active: bool,
}

impl Recorder {
    pub fn new(title: String) -> Recorder {
        Recorder {
            title,
            counter: 0,
            active: true,
        }
    }

    pub fn increment(&mut self) -> () {
        if self.active {
            self.counter += 1;
        }
    }
    pub fn get_counter(&self) -> u128 {
        self.counter
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }
    pub fn get_title(&self) -> String {
        self.title.clone()
    }

    pub fn activate_deactivate(&mut self) {
        self.active = !self.active
    }
    pub fn activate(&mut self) {
        self.active = true
    }

    pub fn deactivate(&mut self) {
        self.active = false
    }
    pub fn get_status_recorder(&self) -> bool {
        self.active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_recorder_is_active() {
        let r = Recorder::new("Test".to_string());
        assert!(r.get_status_recorder());
        assert_eq!(r.get_counter(), 0);
    }

    #[test]
    fn increment_only_when_active() {
        let mut r = Recorder::new("Test".to_string());

        r.increment();
        r.deactivate();
        r.increment();

        assert_eq!(r.get_counter(), 1);
    }

    #[test]
    fn activate_deactivate_works() {
        let mut r = Recorder::new("Test".to_string());

        r.deactivate();
        assert!(!r.get_status_recorder());

        r.activate();
        assert!(r.get_status_recorder());
    }
}
