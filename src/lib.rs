// An attribute to hide warnings for unused code.
#![allow(dead_code)]
#[macro_use]

extern crate simple_error;
extern crate tantivy;
extern crate roaring;

use log::LevelFilter;
use std::ops::Bound;
use std::time::Instant;
use std::collections::HashMap;

use roaring::RoaringTreemap;

use tantivy::IndexReader;
use tantivy::collector::{TopDocs, DocSetCollector};
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::fastfield::Column;
use tantivy::Index;
use tantivy::ReloadPolicy;
use tantivy::IndexWriter;
use tantivy::directory::MmapDirectory;
use tantivy::merge_policy::LogMergePolicy;
use tantivy::query::{BooleanQuery, Occur, RangeQuery, Query, TermQuery};

// include shared struct in Rust
use crate::ffi::DocumentField;
use crate::ffi::FieldType;
use crate::ffi::IdDocument;
use crate::ffi::FieldMapping;
use crate::ffi::TOccur;
use crate::ffi::SearchParam;
use crate::ffi::IndexParam;
use crate::ffi::RangeBound;
use crate::ffi::StringBound;
use crate::ffi::FloatBound;
use crate::ffi::LongBound;

use std::error::Error;

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
    #[derive(Debug)]
    struct DocumentField{
        field_name: String,
        field_value: String,
        field_type: FieldType, // "String", "Long", "Int", "Double", "Boolean"
    }

    struct SearchParam{
        topK: usize,
    }
    
    struct IndexParam{
        memory_mbytes: usize,
    }

    #[derive(Debug, Clone, Copy)]
    enum RangeBound {
        /// An inclusive bound.
        Included,

        /// An exclusive bound.
        Excluded,

        /// An infinite endpoint. Indicates that there is no bound in this direction.
        Unbounded,
    }

    struct StringBound{
        bound: RangeBound,
        value: String,
    }

    struct FloatBound{
        bound:RangeBound,
        value:f64,
    }

    struct LongBound{
        bound:RangeBound,
        value:i64,
    }

    #[derive(Debug)]
    struct IdDocument{
        docId: i64, // i64 instead of u64 for easier query against _doctId field by term_query_long()
        // title: String,
        fieldValues: Vec<DocumentField>,
        score: f32, // score for matched document 
    }

    /// Defines whether a term in a query must be present,
    /// should be present or must not be present.
    #[derive(Debug, Clone, Copy)]
    pub enum TOccur {
        /// For a given document to be considered for scoring,
        /// at least one of the terms with the Should or the Must
        /// Occur constraint must be within the document.
        Should,
        /// Document without the term are excluded from the search.
        Must,
        /// Document that contain the term are excluded from the
        /// search.
        MustNot,
    }
    
    extern "Rust" {
        type TQuery;

        type TQueryOccur;

        type TQueryOccurVec;

        fn query_occur_vec() -> Box<TQueryOccurVec>;

        fn append_query_occur_to_vec(occurs_vec: & mut TQueryOccurVec, query_occur: & mut TQueryOccur);
    }

    // definition of Rust interface 
    extern "Rust" {
        type Searcher;
        
        type SearchResultBitmap;

        fn create_searcher(path: &String, field_mappings:Vec<FieldMapping>) -> Result<Box<Searcher>>;
        
        fn create_searcher_with_param(path: &String, field_mappings:Vec<FieldMapping>, param: IndexParam) -> Result<Box<Searcher>>;
        
        fn search_compact_all(searcher: & mut Searcher, query: & TQuery) -> Result<Box<SearchResultBitmap>>;
        
        fn num_docs(searcher: & mut Searcher) -> Result<u64>;

        fn is_member(result_map: & mut SearchResultBitmap, doc_id: u64) -> Result<bool>;

        fn add_document(searcher: &mut Searcher, docs:Vec<IdDocument>, commit: bool) -> Result<()>;

        fn search(searcher: & mut Searcher, query: &String, search_fields: & Vec<String>, search_param: & SearchParam) -> Result<Vec<IdDocument>>;

        fn search_by_query(searcher: & mut Searcher, query: & TQuery, search_param: & SearchParam) -> Result<Vec<IdDocument>>;

        fn term_query(searcher: &mut Searcher, field_name: &String, field_value: &String) -> Result<Box<TQuery>>;

        fn term_query_long(searcher: &mut Searcher, field_name: &String, field_value: i64) -> Result<Box<TQuery>>;

        fn range_query(searcher: &mut Searcher, field_name: &String, from_value: &StringBound, to_value: &StringBound) -> Result<Box<TQuery>>;

        fn range_query_float(searcher: &mut Searcher, field_name: &String, from_value: &FloatBound, to_value: &FloatBound) -> Result<Box<TQuery>> ;

        fn range_query_long(searcher: &mut Searcher, field_name: &String, from_value: &LongBound, to_value: &LongBound) -> Result<Box<TQuery>>;
     
        fn query_occurr(occurr: & TOccur, query: & mut TQuery) -> Box<TQueryOccur>;

        fn boolean_query(queries: & TQueryOccurVec ) -> Result<Box<TQuery>>;
        
        fn delete_document(searcher: &mut Searcher, doc_ids:Vec<i64>, commit: bool) -> Result<()>;

        pub fn commit_index(searcher: &mut Searcher)  -> Result<()>;
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

impl std::fmt::Display for TQuery {
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

pub struct SearchResultBitmap{
    bitmap:  RoaringTreemap,
}

#[derive(Debug)]
pub struct TQuery{
    query: Box<dyn Query>,
}

#[derive(Debug)]
pub struct TQueryOccur{
    occur: TOccur,
    query: Box<dyn Query>,
}

pub struct TQueryOccurVec {
    occurs: Vec<TQueryOccur>,
}

pub fn term_query(searcher: &mut Searcher, field_name: &String, field_value: &String) -> Result<Box<TQuery>, Box<dyn Error>> {
    let field_option = searcher.schema.get_field(field_name);

    if field_option.is_err(){
        bail!(format!("field {field_name} not found! "));
    }

    let field = field_option.unwrap();

    let tq = TQuery{ query: Box::new( TermQuery::new(
        Term::from_field_text(field, field_value),
        IndexRecordOption::Basic,
    ))};

    return Ok(Box::new(tq));
}

pub fn term_query_long(searcher: &mut Searcher, field_name: &String, field_value: i64) -> Result<Box<TQuery>, Box<dyn Error>> {
    let field_option = searcher.schema.get_field(field_name);

    if field_option.is_err(){
        bail!(format!("field {field_name} not found! "));
    }

    let field = field_option.unwrap();

    let tq = TQuery{ query: Box::new( TermQuery::new(
        Term::from_field_i64(field, field_value),
        IndexRecordOption::Basic,
    ))};

    return Ok(Box::new(tq));
}

pub fn range_query(searcher: &mut Searcher, field_name: &String, from_value: &StringBound, to_value: &StringBound) -> Result<Box<TQuery>, Box<dyn Error>> {
    let field_option = searcher.schema.get_field(field_name);

    if field_option.is_err(){
        bail!(format!("field {field_name} not found! "));
    }

    let field = field_option.unwrap();

    //let left: Bound<&str> = Bound::Included(&from_value.value);

    let left: Bound<&str> = match from_value.bound{
        RangeBound::Included => Bound::Included(&from_value.value),
        RangeBound::Excluded => Bound::Excluded(&from_value.value),
        _ => Bound::Unbounded,
    };

    let right: Bound<&str> = match to_value.bound{
        RangeBound::Included => Bound::Included(&to_value.value),
        RangeBound::Excluded => Bound::Excluded(&to_value.value),
        _ => Bound::Unbounded,
    };

    let tq = TQuery{ query: Box::new(
        // RangeQuery::new_str(field, &from_value.value..&to_value.value)
        RangeQuery::new_str_bounds(field_name.clone(), left, right)
     )};

    return Ok(Box::new(tq));
}

pub fn range_query_float(searcher: &mut Searcher, field_name: &String, from_value: &FloatBound, to_value: &FloatBound) -> Result<Box<TQuery>, Box<dyn Error>> {
    let field_option = searcher.schema.get_field(field_name);

    if field_option.is_err(){
        bail!(format!("field {field_name} not found! "));
    }

    let field = field_option.unwrap();

    let left: Bound<f64> = match from_value.bound{
        RangeBound::Included => Bound::Included(from_value.value),
        RangeBound::Excluded => Bound::Excluded(from_value.value),
        _ => Bound::Unbounded,
    };

    let right: Bound<f64> = match to_value.bound{
        RangeBound::Included => Bound::Included(to_value.value),
        RangeBound::Excluded => Bound::Excluded(to_value.value),
        _ => Bound::Unbounded,
    };

    let tq = TQuery{ query: Box::new(
        // RangeQuery::new_f64(field, from_value.value..to_value.value)
        RangeQuery::new_f64_bounds(field_name.clone(), left, right)
     )};

    return Ok(Box::new(tq));
}

pub fn range_query_long(searcher: &mut Searcher, field_name: &String, from_value: &LongBound, to_value: &LongBound) -> Result<Box<TQuery>, Box<dyn Error>> {
    let field_option = searcher.schema.get_field(field_name);

    if field_option.is_err(){
        bail!(format!("field {field_name} not found! "));
    }

    let field = field_option.unwrap();

    let left: Bound<i64> = match from_value.bound{
        RangeBound::Included => Bound::Included(from_value.value),
        RangeBound::Excluded => Bound::Excluded(from_value.value),
        _ => Bound::Unbounded,
    };

    let right: Bound<i64> = match to_value.bound{
        RangeBound::Included => Bound::Included(to_value.value),
        RangeBound::Excluded => Bound::Excluded(to_value.value),
        _ => Bound::Unbounded,
    };


    let tq = TQuery{ query: Box::new(
        // RangeQuery::new_i64(field, from_value.value..to_value.value)
        RangeQuery::new_i64_bounds(field_name.clone(), left, right)
     )};

    return Ok(Box::new(tq));
}


pub fn boolean_query(queries: & TQueryOccurVec ) -> Result<Box<TQuery>, Box<dyn Error>> {
    let mut queries_with_occur: Vec<(Occur, Box<dyn Query>)> = Vec::new();

    for query in & queries.occurs {
        match query.occur{
            TOccur::Must => queries_with_occur.push((Occur::Must, query.query.box_clone())),
            TOccur::MustNot => queries_with_occur.push((Occur::MustNot, query.query.box_clone())),
            TOccur::Should => queries_with_occur.push((Occur::Should, query.query.box_clone())),
            _ => log::warn!("Not supported occur!"),
        }
    }

    let tq = TQuery{ query: Box::new(BooleanQuery::new(queries_with_occur))};

    return Ok(Box::new(tq));
}

pub fn query_occurr(occurr: & TOccur, tquery: & mut TQuery) -> Box<TQueryOccur> {
    let query_occur = TQueryOccur{occur: *occurr, query: tquery.query.box_clone()};
    return Box::new(query_occur);
}

pub fn query_occur_vec() -> Box<TQueryOccurVec>{
    return Box::new(TQueryOccurVec{occurs: Vec::new()});
}


pub fn append_query_occur_to_vec(occurs_vec: & mut TQueryOccurVec, query_occur: & mut TQueryOccur) {
    let t = TQueryOccur{occur: query_occur.occur, query: query_occur.query.box_clone()};
    occurs_vec.occurs.push(t);
}


pub fn create_searcher(path: &String, field_mappings:Vec<FieldMapping>) -> Result<Box<Searcher>, Box<dyn Error>>{
    create_searcher_with_param(path, field_mappings, IndexParam{memory_mbytes: 256})
}

pub fn create_searcher_with_param(path: &String, field_mappings:Vec<FieldMapping>, param: IndexParam) -> Result<Box<Searcher>, Box<dyn Error>>
{
    std::fs::create_dir_all("logs")?;
    _ = simple_logging::log_to_file("logs/tantivy_index.log", LevelFilter::Info);
    log::info!("Rust logging initialized");

    let index_dir = std::path::Path::new(path);
    let index_path = index_dir;

    std::fs::create_dir_all(index_path)?; // create dir if not exist

    index_path.to_str().expect("index_path should not be empty");

    let mut schema_builder = Schema::builder();

    // set the _docId to be INDEXED for query & delete
    schema_builder.add_i64_field("_docId", NumericOptions::default() | STORED | INDEXED | FAST);
    for field_mapping in field_mappings {

        let _ = match field_mapping.field_type{
            FieldType::int_field  => schema_builder.add_i64_field(&field_mapping.field_name, NumericOptions::default() | STORED | INDEXED),
            FieldType::long_field => schema_builder.add_i64_field(&field_mapping.field_name, NumericOptions::default() | STORED | INDEXED),
            FieldType::float_field => schema_builder.add_f64_field(&field_mapping.field_name, NumericOptions::default() | STORED | INDEXED ),
            FieldType::double_field => schema_builder.add_f64_field(&field_mapping.field_name, NumericOptions::default() | STORED | INDEXED),
            FieldType::str_field => schema_builder.add_text_field(&field_mapping.field_name, STRING),
            FieldType::bool_field => schema_builder.add_bool_field(&field_mapping.field_name, STORED | INDEXED),
            FieldType::text_field => schema_builder.add_text_field(&field_mapping.field_name, TEXT | STORED),

            _ => schema_builder.add_text_field(&field_mapping.field_name, STRING),
        };
    }

    let schema = schema_builder.build();

    //let index = Index::create_in_dir(&index_path, schema.clone())?;
    let mmap_directory = MmapDirectory::open(index_path)?;
    let index = Index::open_or_create(mmap_directory, schema.clone())?;

    let index_writer = index.writer(param.memory_mbytes * 1000_000)?;

    let mut merge_policy = LogMergePolicy::default();
    merge_policy.set_max_docs_before_merge(320000);
    merge_policy.set_min_num_segments(3);
    merge_policy.set_min_layer_size(60000);

    index_writer.set_merge_policy(Box::new(merge_policy));

    let reader = index_writer.index()
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;
    let searcher = Searcher{_index_path:path.to_string(), schema: schema, index_writer: index_writer, index_reader: reader};

    return Ok(Box::new(searcher));
}


pub fn add_document(searcher: & mut Searcher, docs:Vec<IdDocument>, commit: bool) -> Result<(), Box<dyn Error>>{
    let index_writer = &mut searcher.index_writer;
    
    let id_field = searcher.schema.get_field("_docId").unwrap();

    for doc in docs{
        let mut document = Document::default();

        document.add_i64(id_field, doc.docId);

        for doc_field in doc.fieldValues{
            let field_option = searcher.schema.get_field(&doc_field.field_name);

            match field_option {
                Ok(field) => {
                    let field_value = doc_field.field_value;
                    let _ = match doc_field.field_type {
                        FieldType::int_field  => document.add_i64(field, field_value.as_str().parse::<i64>()?),
                        FieldType::long_field => document.add_i64(field, field_value.as_str().parse::<i64>()?),
                        FieldType::float_field => document.add_f64(field, field_value.as_str().parse::<f64>()?),
                        FieldType::double_field => document.add_f64(field, field_value.as_str().parse::<f64>()?),
                        FieldType::str_field => document.add_text(field, field_value),
                        //field_value should be "true", "false"
                        FieldType::bool_field => document.add_bool(field, field_value.to_lowercase().as_str().parse::<bool>()?),
                        FieldType::text_field => document.add_text(field, field_value),

                        _ => log::warn!("Not supported FieldType {}", doc_field.field_type.to_string()),
                    };
                }
                Err(err) => {
                    bail!(format!("field {} not found! ", doc_field.field_name));
                }
            }
        }
        
        index_writer.add_document(document)?;
    }
     
    if commit {
        index_writer.commit()?;
        _ = searcher.index_reader.reload(); // reload reader after commit
    }
    
    return Ok(());
}

pub fn delete_document(searcher: &mut Searcher, doc_ids:Vec<i64>, commit: bool) -> Result<(), Box<dyn Error>> {

    log::info!("delete doc_ids:{:?}", doc_ids);

    let id_field = searcher.schema.get_field("_docId").unwrap();

    let index_writer = &mut searcher.index_writer;

    for doc_id in doc_ids {
        let term = Term::from_field_i64(id_field, doc_id);
        index_writer.delete_term(term);
    }
    
    if commit {
        index_writer.commit()?;
        _ = searcher.index_reader.reload(); // reload reader after commit
    }

    return Ok(());
}

pub fn commit_index(searcher: &mut Searcher)  -> Result<(), Box<dyn Error>> {
    let index_writer = &mut searcher.index_writer;

    index_writer.commit()?;
    _ = searcher.index_reader.reload(); // reload reader after commit

    return Ok(());
}

pub fn search(searcher: & mut Searcher, query: & String, search_fields: & Vec<String>, search_param: & SearchParam) -> Result<Vec<IdDocument>, Box<dyn Error>> {

    let index_searcher = searcher.index_reader.searcher();
    log::info!("query:{}", query);

    let doc_id_field = searcher.schema.get_field("_docId").unwrap();

    let mut fields: Vec<Field> = Vec::new();

    for search_field in search_fields{
        let field_option = searcher.schema.get_field(search_field.as_str());

        match field_option {
            Ok(field) => {
                fields.push(field);
            }
            Err(err) => {
                bail!(format!("field {search_field} not found! "));
            }
        }
    }

    let query_parser = QueryParser::for_index(searcher.index_writer.index(), fields);
    let query = query_parser.parse_query(query.as_str())?;

    let top_docs = index_searcher.search(&query, &TopDocs::with_limit(search_param.topK))?;    

    let mut id_documents: Vec<IdDocument> =  Vec::new();

    for (_score, doc_address) in top_docs {
        let retrieved_doc = index_searcher.doc(doc_address)?;
        println!("score:{} {}", _score, searcher.schema.to_json(&retrieved_doc));

        let doc_id = retrieved_doc.get_first(doc_id_field) ;
    
        if doc_id.is_some() {
            let current_id = doc_id.expect("error getting doc_id").as_i64().expect("as_i64");
            let document = IdDocument{docId:current_id, fieldValues: Vec::new(), score: _score };
            id_documents.push(document);
        }
    }

    return Ok(id_documents);
}

pub fn search_by_query(searcher: & mut Searcher, query: & TQuery, search_param: & SearchParam) -> Result<Vec<IdDocument>, Box<dyn Error>> {
    let index_searcher = searcher.index_reader.searcher();
    //println!("query:{}", query);

    let doc_id_field = searcher.schema.get_field("_docId").unwrap();
    let top_docs = index_searcher.search(&query.query, &TopDocs::with_limit(search_param.topK))?;    

    let mut id_documents: Vec<IdDocument> =  Vec::new();

    for (_score, doc_address) in top_docs {
        let retrieved_doc = index_searcher.doc(doc_address)?;
        log::info!("score:{} {}", _score, searcher.schema.to_json(&retrieved_doc));

        let doc_id = retrieved_doc.get_first(doc_id_field) ;
    
        if doc_id.is_some() {
            let current_id = doc_id.expect("error getting doc_id").as_i64().expect("as_i64");
            let document = IdDocument{docId:current_id, fieldValues: Vec::new(), score: _score };
            id_documents.push(document);
        }
    }

    return Ok(id_documents);
}

pub fn num_docs(searcher: & mut Searcher) -> Result<u64, Box<dyn Error>> {
    return Ok(searcher.index_reader.searcher().num_docs());   
}

pub fn search_compact_all(searcher: & mut Searcher, query: & TQuery) -> Result<Box<SearchResultBitmap>, Box<dyn Error>>{

    let index_searcher = searcher.index_reader.searcher();
    let start = Instant::now();
    log::info!("search_compact_all fulltext query:{:?}", query);

    let doc_id_field = searcher.schema.get_field("_docId").unwrap();
    let collector = DocSetCollector{};
    let top_docs = index_searcher.search(&query.query, & collector)?;
    log::info!("search_compact_all, search duration:{} ", start.elapsed().as_millis(), );

    let mut bitmap = RoaringTreemap::new();

    let mut seg_id_readers :HashMap<u32, std::sync::Arc<dyn Column<i64>>> = HashMap::new();

    for doc_address in top_docs {
        // cache doc_id_reader by segment_ord,
        let segment_id = doc_address.segment_ord.clone();
        
        let doc_id_reader_option = seg_id_readers.get(&segment_id);
        match doc_id_reader_option {
            Some(doc_id_reader) => {
                let doc_id = doc_id_reader.get_val(doc_address.doc_id);
                bitmap.insert(doc_id.unsigned_abs());
            }
            None => {
                let segment_reader = index_searcher.segment_reader(segment_id);
                let doc_id_reader = segment_reader.fast_fields().i64("_docId").unwrap();
                let doc_id = doc_id_reader.get_val(doc_address.doc_id);
                seg_id_readers.insert(segment_id, doc_id_reader);
                bitmap.insert(doc_id.unsigned_abs());
            }
        }
        
    }

    let duration = start.elapsed();
    log::info!("search_compact_all, collect duration:{} fulltext query:{:?}", duration.as_millis(), query);
    return Ok(Box::new(SearchResultBitmap { bitmap }));
}

pub fn is_member(result_map: & mut SearchResultBitmap, doc_id: u64) -> Result<bool, Box<dyn Error>> {
    Ok(result_map.bitmap.contains(doc_id))
}

