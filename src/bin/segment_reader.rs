#![allow(dead_code)]
extern crate tantivy;

use tantivy::Index;
use tantivy::directory::MmapDirectory;
use tantivy::SegmentId;

fn main() -> tantivy::Result<()> {
 
    let index_path = std::path::Path::new("/tmp/tantivy.35EB0731");
 

    let mmap_directory = MmapDirectory::open(index_path)?;
    let index = Index::open(mmap_directory)?;

    let segment_ids:Vec<SegmentId> = index.searchable_segment_ids()?;
    println!("segment_ids: {:?}", segment_ids);

    return Ok(());
}
