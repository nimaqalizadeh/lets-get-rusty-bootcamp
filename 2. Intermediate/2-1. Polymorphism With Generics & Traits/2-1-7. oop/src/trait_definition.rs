pub trait Summarizable {
    fn summary(&self) -> String;

    fn default_summary(&self) -> String {
        String::from("Default summary ... ")
    }
}

#[derive(Debug)]
pub struct NewsArticle {
    pub headline: String,
    pub location: String,
    pub author: String,
    pub content: String,
}

#[derive(Debug)]
pub struct Tweet {
    pub username: String,
    pub content: String,
    pub reply: bool,
    pub retweet: bool,
}

impl Summarizable for NewsArticle {
    fn summary(&self) -> String {
        format!("{} by {} in {}", self.headline, self.author, self.location)
    }
}

impl Summarizable for Tweet {
    fn summary(&self) -> String {
        format!("{}: {}", self.username, self.content)
    }

    fn default_summary(&self) -> String {
        if self.retweet {
            format!("Retweeted: {}", self.summary())
        } else {
            self.summary()
        }
    }
}
