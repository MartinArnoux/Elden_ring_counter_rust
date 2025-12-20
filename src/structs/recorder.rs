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
    pub fn get_status_recorder(&self) -> bool {
        self.active
    }
}
