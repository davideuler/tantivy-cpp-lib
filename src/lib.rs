//#[macro_use]
extern crate tantivy;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::ReloadPolicy;
use tantivy::IndexWriter;
use tantivy::directory::MmapDirectory;

// include shared struct in Rust
use crate::ffi::DocumentField;
use crate::ffi::IdDocument;

use std::error::Error;
use std::vec;

#[cxx::bridge]
mod ffi {
    
    // Shared structs with fields visible to both languages.
    struct DocumentField{
        field_name: String,
        field_value: String,
        field_type: String, // "String", "Long", "Int", "Double"
    }

    struct IdDocument{
        docId: u64,
        title: String,
        score: f32, // score for matched document 
    }

    extern "Rust" {
        type Searcher;

        fn create_searcher(path: &String) -> Result<Box<Searcher>>;

        fn add_document(seacher: &mut Searcher, fields:Vec<DocumentField>) -> Result<()>;

        fn search(searcher: & mut Searcher, query: &String) -> Result<Vec<IdDocument>>;
    }
    extern "Rust" {
        fn rust_from_cpp() -> ();
    }
}

pub fn rust_from_cpp() -> () {
    println!("called rust_from_cpp()");
}


pub struct Searcher{
    _index_path: String,
    schema: Schema,
    index_writer: IndexWriter,
}


pub fn create_searcher(path: &String) -> Result<Box<Searcher>, Box<dyn Error>>{
    
    let index_dir = std::path::Path::new(path);
    let index_path = index_dir;
 
    if index_dir.exists() {
        std::fs::remove_dir_all(index_path)?;
    }

    std::fs::create_dir_all(index_path)?;

    let mut schema_builder = Schema::builder();
    schema_builder.add_u64_field("docId", NumericOptions::default() | STORED);
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT);

    let schema = schema_builder.build();

    //let index = Index::create_in_dir(&index_path, schema.clone())?;
    let mmap_directory = MmapDirectory::open(index_path)?;
    let index = Index::open_or_create(mmap_directory, schema.clone())?;

    let index_writer = index.writer(50_000_000)?;
    let searcher = Searcher{_index_path:path.to_string(), schema: schema, index_writer: index_writer};

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
        match doc_field.field_type.as_str() {
            "String" => old_man_doc.add_text(field, field_value),
            "Long" => old_man_doc.add_u64(field, field_value.as_str().parse::<u64>()?),
            "Int" => old_man_doc.add_i64(field, field_value.as_str().parse::<i64>()?),
            "Double" => old_man_doc.add_f64(field, field_value.as_str().parse::<f64>()?),
            _ => println!("Not supported Type{}", doc_field.field_type),
        }
    }    
 
    index_writer.add_document(old_man_doc)?;
    index_writer.commit()?;
    return Ok(());
}

pub fn search(searcher: & mut Searcher, query: & String) -> Result<Vec<IdDocument>, Box<dyn Error>> {

    // FIXME: 不用每次 new 一个 IndexReader
    let reader = searcher.index_writer.index()
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let index_searcher = reader.searcher();

    let title = searcher.schema.get_field("title").unwrap();
    let body = searcher.schema.get_field("body").unwrap();

    let query_parser = QueryParser::for_index(searcher.index_writer.index(), vec![title, body]);
    let query = query_parser.parse_query(query)?;

    let top_docs = index_searcher.search(&query, &TopDocs::with_limit(10))?;

    let mut id_documents: Vec<IdDocument> =  Vec::new();

    for (score, doc_address) in top_docs {
        let retrieved_doc = index_searcher.doc(doc_address)?;
        let doc_title = retrieved_doc.get_first(title) ;

        if doc_title.is_some() {
            let a = doc_title.expect( "error getting title");
            let document = IdDocument{docId:123, title: a.as_text().expect("error as_text()").to_string(), score: score };
            id_documents.push(document);
        }
        
        println!("{}",  searcher.schema.to_json(&retrieved_doc));
    }

    return Ok(id_documents);
}