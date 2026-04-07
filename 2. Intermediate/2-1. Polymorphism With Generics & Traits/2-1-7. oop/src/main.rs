// Reminder: `mod math_utils;` must be declared here in the parent module (main.rs),
// NOT inside math_utils.rs itself. Files don't automatically become modules just by
// existing in src/ The module tree is built explicitly through
// mod declarations, starting from the crate root
// (main.rs or lib.rs). — the compiler only reads math_utils.rs because of this declaration.
//Think of it this way:
//  - mod math_utils; in main.rs means: "Hey
//  compiler, there's a child module called
//  math_utils. Go look for it in math_utils.rs or
//   math_utils/mod.rs and include it in my module
//   tree."
//  - Without this declaration, the compiler will
//  never even read math_utils.rs. The file is
//  essentially invisible to the build.
// Putting `mod math_utils;` inside math_utils.rs would mean the module declares itself
// as its own child, which is circular and meaningless.
mod math_utils;
mod trait_definition;

use crate::{math_utils::AveragedCollection, trait_definition::{NewsArticle, Summarizable, Tweet}};

fn main() {
    // ********** math_utils 
    println!("********** beginning of math_utils **********" );
    let mut avg_collection = AveragedCollection::new();
    avg_collection.add(1);
    avg_collection.add(3);
    avg_collection.add(5);
    println!("Add result: {:?}", avg_collection.average());
    avg_collection.remove();
    println!("Remove result: {:?}", avg_collection.average());

    // Cannot do this - fields are private:
    // collection.list.push(100); // Compile Error!
    // collection.average = 50.0; // Compile Error!

    // Cannot call private methods:
    // collection.update_average(); // Compile Error!

    println!("********** end of math_utils **********" );
    // ********** end of math_utils

    println!();

    // ********** trait definition 
    println!("********** beginning of trait_definition **********" );
    let article = NewsArticle {
        headline: String::from("Breaking news"),
        location: String::from("US"),
        author: String::from("Test"),
        content: String::from("Test News"),
    };

    let tweet = Tweet {
        username: String::from("test_user"),
        content: String::from("Test tweet content"),
        reply: true,
        retweet: true,
    };

    println!("Article summary: {}", article.summary());
    println!("Article default summary: {}", article.default_summary());
    println!("Tweet summary: {}", tweet.summary());
    println!("Tweet default summary: {}", tweet.default_summary());


    println!("********** beginning of trait_definition **********" );
    // ********** end of trait definition 

    println!();
}
