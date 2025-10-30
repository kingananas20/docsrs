use clap::Parser;
use docsrs::{Doc, Error};

/// A fast, fuzzy-search for rust-docs.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The crate to search in
    crate_name: String,

    /// The search query
    query: String,

    /// The version of the crate to search in
    #[arg(short = 'v', long = "crate-version", default_value = "latest", value_name = "VERSION")]
    crate_version: String,

    /// The number of search results to return
    #[arg(short, long, default_value_t = 10)]
    n: usize,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let doc = Doc::from_docs(&args.crate_name, &args.crate_version)?
        .fetch()?
        .decompress()?
        .parse()?
        .build_search_index();

    if let Some(results) = doc.search(&args.query, Some(args.n)) {
        if let Some((first, rest)) = results.split_first() {
            println!("{}", first.path.join("::"));
            if let Some(docs) = &first.docs {
                println!("\n{}", docs);
            }

            if !rest.is_empty() {
                println!("\n---\n");
                for item in rest {
                    println!("{}", item.path.join("::"));
                }
            }
        }
    } else {
        println!(
            "No results found for query `{}` in crate `{}`",
            args.query, args.crate_name
        );
    }

    Ok(())
}
