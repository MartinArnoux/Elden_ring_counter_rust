use std::fmt;
use xcap::Monitor;

pub fn get_screens_vec() -> Result<Vec<ScreenInfo>, String> {
    let monitors = Monitor::all().map_err(|e| format!("Erreur Monitor::all: {}", e))?;

    let mut result = Vec::new();

    for (index, monitor) in monitors.into_iter().enumerate() {
        let index = index as i8;

        let name = monitor
            .name()
            .map_err(|e| format!("Erreur monitor.name() pour écran {}: {}", index, e))?;

        result.push(ScreenInfo { index, name });
    }

    Ok(result)
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenInfo {
    pub index: i8,
    pub name: String,
}

impl fmt::Display for ScreenInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (écran {})", self.name, self.index + 1)
    }
}
