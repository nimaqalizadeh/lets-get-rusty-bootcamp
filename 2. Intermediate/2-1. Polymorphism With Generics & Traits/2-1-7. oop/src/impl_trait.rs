use crate::trait_definition::Summarizable;

// This function accepts any type that implements Summarizable
pub fn notify(text: &impl Summarizable) -> String {
    format!("From Notify: {}", text.summary())
}
