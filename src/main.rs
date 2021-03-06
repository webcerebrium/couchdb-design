use std::error::Error;
use std::path::PathBuf;
use structopt::StructOpt;

mod designcompare;
mod designdoc;
use designcompare::Compare;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "couchdb-design",
    about = "A command line interface to work with CouchDB design documents as YAML configurations"
)]
struct Opt {
    /// Local YAML file to be uploaded as design document.
    /// If not provided, it will read URLs and display it
    /// as YAML file in stdout
    #[structopt(short = "f", long, parse(from_os_str))]
    file: Option<PathBuf>,

    /// URL of the remote couch design document to be read of updated
    url: String,

    /// Supress diffs fore document views
    #[structopt(short = "q", long)]
    quiet: bool,

    /// Just show diff, do not actually upload
    #[structopt(short = "t", long)]
    test: bool,

    /// Force file creation
    #[structopt(long)]
    force: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    // println!("opt {:#?}", opt.clone());
    let url = opt.url.as_str();
    let remote: Option<designdoc::DesignDoc> = designdoc::DesignDoc::from_url(url).await?;

    if let Some(local_path) = opt.file {
        let mut local_doc = designdoc::DesignDoc::from_file(local_path).await?;
        match remote {
            None => {
                // case that we have no remote URL and need to create it
                if opt.force {
                    let result = local_doc.create(url).await?;
                    println!("CREATED: REV={} URL={}", result.rev.as_str(), url);
                    return Ok(());
                } else {
                    return Err(format!(
                        "ERROR: Document was not found at {}{}",
                        url, "Please consider --force option"
                    )
                    .into());
                }
            }
            Some(remote_doc) => {
                let comparison = Compare::docs(&local_doc, &remote_doc);
                if opt.test {
                    println!("{} URL={}", comparison, url);
                    comparison.show_details()?;
                } else if comparison.is_modified() {
                    let rev = remote_doc._rev.clone().unwrap();
                    local_doc._rev = Some(rev.clone());
                    let result = local_doc.update(url).await?;
                    println!("UPDATED: REV={} {} URL={}", 
                        result.rev.as_str(),comparison, url);
                    comparison.show_details()?;
                } else {
                    println!("{} URL={}", comparison, url);
                }
            }
        }
    } else {
        match remote {
            Some(doc) => println!("{}", doc),
            None => {
                return Err(format!("Failed to fetch document from {}", url).into());
            }
        }
    }
    Ok(())
}
