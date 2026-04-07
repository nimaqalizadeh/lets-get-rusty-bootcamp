use crate::trait_definition::Summarizable;

// This function accepts any type that implements Summarizable
pub fn notify(item: &impl Summarizable) -> String {
    format!("From Notify: {}", item.summary())
}

pub fn notify_generic<T: Summarizable>(item: &T) -> String {
    format!("From Notify Generic: {}", item.summary())
}

pub fn notify_complex<T>(item1: &T, item2: &T) -> String
where
    T: Summarizable + std::fmt::Debug,
{
    format!(
        "From Notify Complex: item 1: {}- item 2: {}",
        item1.summary(),
        item2.default_summary()
    )
}

pub fn notify_complex_other_form<T: Summarizable>(item1: &T, item2: &T) -> String {
    format!(
        "From Notify Complex other form: item 1: {} - item 2 {}",
        item1.default_summary(),
        item2.summary()
    )
}

pub fn notify_complex_with_different_types(
    item1: &impl Summarizable,
    item2: &impl Summarizable,
) -> String {
    format!(
        "From Notify Complex with different types: item 1: {} - item 2 {}",
        item1.default_summary(),
        item2.summary()
    )
}
