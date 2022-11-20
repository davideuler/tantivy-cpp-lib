#[macro_use]
extern crate tantivy;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::ReloadPolicy;
use tantivy::IndexReader;
use tantivy::IndexWriter;
use tantivy::directory::MmapDirectory;

// include shared struct in Rust
use crate::ffi::DocumentField;

use std::error::Error;

#[cxx::bridge]
mod ffi {
    
    // Shared structs with fields visible to both languages.
    struct DocumentField{
        field_name: String,
        field_value: String,
        filed_type: String, // "String", "Long", "Int", "Double"
    }

    extern "Rust" {
        type Searcher;

        fn create_searcher(path: &String) -> Result<Box<Searcher>>;

        fn add_document(seacher: &mut Searcher, fields:Vec<DocumentField>) -> Result<()>;
    }
    extern "Rust" {
        fn rust_from_cpp() -> ();
    }
}

pub fn rust_from_cpp() -> () {
    println!("called rust_from_cpp()");
    
}


pub struct Searcher{
    index_path: String,
    schema: Schema,
    index_writer: IndexWriter,
    index_reader: IndexReader
}


pub fn create_searcher(path: &String) -> Result<Box<Searcher>, Box<dyn Error>>{
    
    let index_dir = std::path::Path::new(path);
    let index_path = index_dir;
 
    if !index_dir.exists() {
        std::fs::create_dir_all(index_path)?;
    }

    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT);

    let schema = schema_builder.build();

    //let index = Index::create_in_dir(&index_path, schema.clone())?;
    let mmap_directory = MmapDirectory::open(index_path)?;
    let index = Index::open_or_create(mmap_directory, schema.clone())?;

    let mut index_writer = index.writer(50_000_000)?;

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let searcher = Searcher{index_path:path.to_string(), schema: schema, index_writer:index_writer, index_reader:reader};

    return Ok(Box::new(searcher));
}

pub fn add_document(searcher: & mut Searcher, fields:Vec<DocumentField>) -> Result<(), Box<dyn Error>>{
    let index_writer = &mut searcher.index_writer;
    let mut old_man_doc = Document::default();

    // let title = schema.get_field("title").unwrap();
    // let body = schema.get_field("body").unwrap();

    for doc_field in fields{
        let field = searcher.schema.get_field(&doc_field.field_name).unwrap();
        let field_value = doc_field.field_value;
        old_man_doc.add_text(field, field_value);
    }    
 
    index_writer.add_document(old_man_doc)?;
    index_writer.commit()?;
    return Ok(());
}

pub fn search(seacher: & mut Searcher) -> () {
    let index_searcher = seacher.index_reader.searcher();
}