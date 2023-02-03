#![allow(dead_code)]
#[macro_use]
extern crate tantivy;

use tantivy::schema::*;
use tantivy::Index;
use tantivy::directory::MmapDirectory;
use tantivy::collector::{DocSetCollector};

use tantivy::query::{BooleanQuery, Occur, RangeQuery, Query, TermQuery};

use std::ops::Bound;
use roaring::RoaringTreemap;

fn main() -> tantivy::Result<()> {
 
    let index_path = std::path::Path::new("/tmp/test_index/");
 
    // let index = Index::create_in_dir(&index_path, schema.clone())?;

    let mmap_directory = MmapDirectory::open(index_path)?;
    let index = Index::open(mmap_directory)?;

    // let index = Index::open_or_create(mmap_directory, schema.clone())?;
    let schema = index.schema();

    println!("Schema: {:?}", schema);
 
    let doc_id_fieldname = String::from("_docId");
    let doc_id_field = schema.get_field(&doc_id_fieldname).unwrap();
    // let stock_field = schema.get_field("stock").unwrap();

    let index_searcher = index.reader()?.searcher();

    let left: Bound<i64> = Bound::Unbounded;
    let right: Bound<i64> = Bound::Excluded(200);
    let range_query = RangeQuery::new_i64_bounds(doc_id_fieldname, left, right);
    
    println!("doing query:{:?}", range_query);

    let collector = DocSetCollector{};
    let top_docs = index_searcher.search(& range_query, & collector)?;    

    let mut bitmap = RoaringTreemap::new();

    for doc_address in top_docs {
        let retrieved_doc = index_searcher.doc(doc_address)?;
        // println!("score:{} {}", _score, searcher.schema.to_json(&retrieved_doc));

        let doc_id = retrieved_doc.get_first(doc_id_field) ;
    
        if doc_id.is_some() {
            let current_id = doc_id.expect("error getting doc_id").as_i64().expect("as_i64");
            println!("current_id:{}, {}", current_id, current_id.unsigned_abs());
            bitmap.insert(current_id.unsigned_abs());
        }
    }

    println!("contains 1? {} ", bitmap.contains(1));
    println!("contains 2? {} ", bitmap.contains(2));
    println!("contains 3? {} ", bitmap.contains(3));
    println!("contains 106? {} ", bitmap.contains(106));
    println!("contains 190? {} ", bitmap.contains(190));

    return Ok(());
}
