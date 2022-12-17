// An attribute to hide warnings for unused code.
#![allow(dead_code)]
#[macro_use]

extern crate simple_error;
extern crate tantivy;

use tantivy::IndexReader;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::ReloadPolicy;
use tantivy::IndexWriter;
use tantivy::directory::MmapDirectory;

// include shared struct in Rust
use crate::ffi::DocumentField;
use crate::ffi::FieldType;
use crate::ffi::IdDocument;
use crate::ffi::FieldMapping;

use std::error::Error;
use std::vec;

#[cxx::bridge]
mod ffi {
    
    #[derive(Debug, Clone, Copy)]
    enum FieldType {
        unspecified_field_type = 0,
        int_field = 1,
        long_field = 2,
        float_field = 3,
        double_field = 4,
        str_field = 5, // untokenized and indexed
        bool_field = 6,
        text_field = 7, // tokenized and indexed
    }

    struct FieldMapping{
        field_name: String,
        field_type: FieldType,
    }

    // Shared structs with fields visible to both languages.
    struct DocumentField{
        field_name: String,
        field_value: String,
        field_type: FieldType, // "String", "Long", "Int", "Double", "Boolean"
    }


    struct IdDocument{
        docId: u64,
        // title: String,
        fieldValues: Vec<DocumentField>,
        score: f32, // score for matched document 
    }

    // definition of Rust interface 
    extern "Rust" {
        type Searcher;

        fn create_searcher(path: &String, field_mappings:Vec<FieldMapping>) -> Result<Box<Searcher>>;

        fn add_document(seacher: &mut Searcher, docs:Vec<IdDocument>) -> Result<()>;

        fn search(searcher: & mut Searcher, query: &String) -> Result<Vec<IdDocument>>;
    }
    extern "Rust" {
        fn rust_from_cpp() -> ();
    }
}

pub fn rust_from_cpp() -> () {
    println!("called rust_from_cpp()");
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

pub struct Searcher{
    _index_path: String,
    schema: Schema,
    index_writer: IndexWriter,
    index_reader: IndexReader,
}


pub fn create_searcher(path: &String, field_mappings:Vec<FieldMapping>) -> Result<Box<Searcher>, Box<dyn Error>>{
    
    let index_dir = std::path::Path::new(path);
    let index_path = index_dir;
 
    if index_dir.exists() {
        std::fs::remove_dir_all(index_path)?;
    }

    index_path.to_str().expect("msg");
    
    std::fs::create_dir_all(index_path)?;

    let mut schema_builder = Schema::builder();
    
    schema_builder.add_u64_field("docId", NumericOptions::default() | STORED);
    for field_mapping in field_mappings {

        let _ = match field_mapping.field_type{
            FieldType::int_field  => schema_builder.add_u64_field(&field_mapping.field_name, NumericOptions::default() | STORED),
            FieldType::long_field => schema_builder.add_u64_field(&field_mapping.field_name, NumericOptions::default() | STORED),
            FieldType::float_field => schema_builder.add_f64_field(&field_mapping.field_name, NumericOptions::default() | STORED),
            FieldType::double_field => schema_builder.add_u64_field(&field_mapping.field_name, NumericOptions::default() | STORED),
            FieldType::str_field => schema_builder.add_text_field(&field_mapping.field_name, STRING),
            FieldType::bool_field => schema_builder.add_bool_field(&field_mapping.field_name, NumericOptions::default() | STORED),
            FieldType::text_field => schema_builder.add_text_field(&field_mapping.field_name, TEXT | STORED),
            
            _ => schema_builder.add_text_field(&field_mapping.field_name, STRING),
        };

    }

    let schema = schema_builder.build();

    //let index = Index::create_in_dir(&index_path, schema.clone())?;
    let mmap_directory = MmapDirectory::open(index_path)?;
    let index = Index::open_or_create(mmap_directory, schema.clone())?;

    let index_writer = index.writer(50_000_000)?;

    let reader = index_writer.index()
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;
    let searcher = Searcher{_index_path:path.to_string(), schema: schema, index_writer: index_writer, index_reader: reader};

    return Ok(Box::new(searcher));
}


pub fn add_document(searcher: & mut Searcher, docs:Vec<IdDocument>) -> Result<(), Box<dyn Error>>{
    let index_writer = &mut searcher.index_writer;
    
    let id_field = searcher.schema.get_field("docId").unwrap();

    for doc in docs{
        let mut document = Document::default();

        document.add_u64(id_field, doc.docId);

        for doc_field in doc.fieldValues{
            let field_option = searcher.schema.get_field(&doc_field.field_name);

            match field_option {
                Some(field) => {
                    let field_value = doc_field.field_value;
                    let _ = match doc_field.field_type {
                        FieldType::int_field  => document.add_i64(field, field_value.as_str().parse::<i64>()?),
                        FieldType::long_field => document.add_i64(field, field_value.as_str().parse::<i64>()?),
                        FieldType::float_field => document.add_f64(field, field_value.as_str().parse::<f64>()?),
                        FieldType::double_field => document.add_f64(field, field_value.as_str().parse::<f64>()?),
                        FieldType::str_field => document.add_text(field, field_value),
                        FieldType::bool_field => document.add_field_value(field, field_value),
                        FieldType::text_field => document.add_text(field, field_value),
                        
                        _ => println!("Not supported FieldType{}", doc_field.field_type.to_string()),
                    };
                }
                None => {
                    bail!("field {doc_field.field_name} not found! ");  
                }
            }
        } 
        
        index_writer.add_document(document)?;
    }
     
    index_writer.commit()?;

    _ = searcher.index_reader.reload(); // reload reader after commit
    return Ok(());
}

pub fn search(searcher: & mut Searcher, query: & String) -> Result<Vec<IdDocument>, Box<dyn Error>> {

    // FIXME: 不用每次 new 一个 IndexReader
    let index_searcher = searcher.index_reader.searcher();
    println!("query:{}", query);

    let doc_id_field = searcher.schema.get_field("docId").unwrap();
    let title = searcher.schema.get_field("title").unwrap();
    let body = searcher.schema.get_field("body").unwrap();

    let query_parser = QueryParser::for_index(searcher.index_writer.index(), vec![title, body]);
    let query = query_parser.parse_query(query.as_str())?;

    let top_docs = index_searcher.search(&query, &TopDocs::with_limit(10))?;

    let mut id_documents: Vec<IdDocument> =  Vec::new();

    for (_score, doc_address) in top_docs {
        let retrieved_doc = index_searcher.doc(doc_address)?;
        println!("score:{} {}", _score, searcher.schema.to_json(&retrieved_doc));

        let doc_id = retrieved_doc.get_first(doc_id_field) ;
        let doc_title = retrieved_doc.get_first(title) ;

        if doc_title.is_some() && doc_id.is_some() {
            let a = doc_title.expect( "error getting title");
            let current_id = doc_id.expect("error getting doc_id").as_u64().expect("as_u64");
            let mut field_values: Vec<DocumentField> = Vec::new();
            let title_field = DocumentField{field_name: String::from("title"), field_value: a.as_text().expect("error as_text()").to_string(), field_type: FieldType::str_field};
            field_values.push(title_field);
            let document = IdDocument{docId:current_id, fieldValues: field_values, score: _score };
            id_documents.push(document);
        }
    }

    return Ok(id_documents);
}