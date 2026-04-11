#[derive(Debug)]
pub struct WindowConfig {
    title: String,
    width: u32,
    height: u32,
    is_resizable: bool,
    has_decorations: bool,
}

pub struct WindowConfigBuilder {
    title: String, // Required field
    width: Option<u32>,
    height: Option<u32>,
    is_resizable: Option<bool>,
    has_decorations: Option<bool>,
}

impl WindowConfigBuilder {
    // Start building with the required field(s)
    pub fn new(title: String) -> Self {
        WindowConfigBuilder {
            title,
            width: None,
            height: None,
            is_resizable: None,
            has_decorations: None,
        }
    }

    // Methods to set optional fields, consuming and returning self (fluent interface)
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.is_resizable = Some(resizable);
        self
    }

    pub fn decorations(mut self, decorations: bool) -> Self {
        self.has_decorations = Some(decorations);
        self
    }

    // Finalize the build, providing defaults for unset options
    pub fn build(self) -> WindowConfig {
        WindowConfig {
            title: self.title,
            width: self.width.unwrap_or(800),
            height: self.height.unwrap_or(600),
            is_resizable: self.is_resizable.unwrap_or(true),
            has_decorations: self.has_decorations.unwrap_or(true),
        }
    }
}

use std::rc::Rc;
use std::cell::RefCell;

// The trait that all observers must implement.
trait Observer {
    // The subject calls this method to notify the observer of a change.
    fn update(&self, new_state: &str);
}

// The subject holds the state and a list of observers.
struct Subject {
    state: String,
    observers: RefCell<Vec<Rc<dyn Observer>>>,
}

impl Subject {
    fn new(initial_state: &str) -> Self {
        Subject {
            state: initial_state.to_string(),
            observers: RefCell::new(Vec::new()),
        }
    }

    // Add a new observer to the list.
    fn attach(&self, observer: Rc<dyn Observer>) {
        self.observers.borrow_mut().push(observer);
    }

    // Change the state and notify all observers.
    fn set_state(&mut self, new_state: &str) {
        self.state = new_state.to_string();
        println!("\nSubject: State changed to '{}'. Notifying observers...", self.state);
        // We borrow the observers list immutably to iterate and notify.
        for observer in self.observers.borrow().iter() {
            observer.update(&self.state);
        }
    }
}

// A concrete observer that logs updates.
struct Logger {
    name: String,
}

impl Observer for Logger {
    fn update(&self, new_state: &str) {
        println!("[Logger {}]: Received update! New state is: '{}'", self.name, new_state);
    }
}

// Another concrete observer that might perform a different action.
struct Notifier {
    email: String,
}

impl Observer for Notifier {
    fn update(&self, new_state: &str) {
        println!("[Notifier]: Sending email to {}. Subject: State changed to '{}'", self.email, new_state);
    }
}

fn main() {
    let basic_window = WindowConfigBuilder::new("My App".to_string()).build(); // Uses all defaults

    let custom_window = WindowConfigBuilder::new("Game Window".to_string())
        .width(1024)
        .height(768)
        .resizable(false)
        .build(); // Sets some fields, uses defaults for others

    let fullscreen_window = WindowConfigBuilder::new("Fullscreen".to_string())
        .decorations(false) // Only set decorations
        .build();

    println!("----------------------------------------");
    println!("Basic Window: {:?}", basic_window);
    println!("Custom Window: {:?}", custom_window);
    println!("Fullscreen Window: {:?}", fullscreen_window);

        let mut subject = Subject::new("Initial State");

    // Create observers. We wrap them in Rc to manage shared ownership.
    let logger = Rc::new(Logger { name: "FileLogger".to_string() });
    let notifier = Rc::new(Notifier { email: "admin@example.com".to_string() });

    // Attach the observers to the subject.
    subject.attach(Rc::clone(&logger) as Rc<dyn Observer>);
    subject.attach(Rc::clone(&notifier) as Rc<dyn Observer>);

    // Change the subject's state. This should trigger notifications.
    subject.set_state("State A");
    subject.set_state("State B");

}

