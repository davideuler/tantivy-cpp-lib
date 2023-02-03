#![allow(dead_code)]
#[macro_use]
extern crate tantivy;


use std::collections::HashSet;
use std::time::Instant;
use std::ops::Bound;

use tantivy::Index;
use tantivy::directory::MmapDirectory;
use tantivy::collector::{DocSetCollector};

use tantivy::collector::{TopDocs, FilterCollector};
use tantivy::DocAddress;
 
use tantivy::query::{BooleanQuery, Occur, RangeQuery, Query, TermQuery};
use tantivy::schema::{NumericOptions, Cardinality};

use crate::fast::FastDocCollector;

pub mod fast;

fn main() -> tantivy::Result<()> {
 
    let index_path = std::path::Path::new("/tmp/tantivy.35EB0731");

    let mmap_directory = MmapDirectory::open(index_path)?;
    let index = Index::open(mmap_directory)?;

    let schema = index.schema(); 
    let doc_id_field = schema.get_field("_docId").unwrap();
    // let stock_field = schema.get_field("stock").unwrap();

    let index_searcher = index.reader()?.searcher();
    let right: Bound<i64> = Bound::Unbounded;
    let left: Bound<i64> = Bound::Excluded(1);
    let range_query = RangeQuery::new_i64_bounds(String::from("_docId"), left, right);
    
    println!("doing query:{:?}", range_query);

    do_query(&index_searcher, & range_query)?;
    do_filter_query(&index_searcher, & range_query)?;

    do_query(&index_searcher, & range_query)?;
    do_filter_query(&index_searcher, & range_query)?;

    return Ok(());
}

fn do_query(index_searcher: & tantivy::Searcher, range_query: & RangeQuery) -> Result<HashSet<DocAddress>, tantivy::TantivyError> {
    let start = Instant::now();
    let collector = FastDocCollector{};
    let doc_addresses = index_searcher.search(range_query, & collector)?;
    println!("do_query, search duration:{} ", start.elapsed().as_millis(), );
    Ok(doc_addresses)
}

fn do_filter_query(index_searcher: & tantivy::Searcher, range_query: & RangeQuery) -> Result<(), tantivy::TantivyError> {
    let start = Instant::now();
   
    
    let score_options = NumericOptions::default()
            .set_indexed()
            .set_fieldnorm()
            .set_fast(Cardinality::SingleValue);

    let schema = index_searcher.index().schema();
    let doc_id_field = schema.get_field("_docId").unwrap();

    // let no_filter_collector = FilterCollector::new(doc_id_field, &|value: i64| value > 0i64, TopDocs::with_limit(2));
    // let top_docs = index_searcher.search(range_query, &no_filter_collector)?;
   
    // assert_eq!(top_docs.len(), 1);
    // assert_eq!(top_docs[0].1, DocAddress::new(0, 1));
   
    let filter_all_collector: FilterCollector<_, _, i64> = FilterCollector::new(doc_id_field, &|value:i64| value >= 0i64, TopDocs::with_limit(2000000));
    let filtered_top_docs = index_searcher.search(range_query, &filter_all_collector)?;
   
    // assert_eq!(filtered_top_docs.len(), 0);

    println!("do_filter_query, search duration:{} count:{}", start.elapsed().as_millis(), filtered_top_docs.len());
    Ok(())
}
