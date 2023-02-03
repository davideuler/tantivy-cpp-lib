// # Custom collector example
//
// This example shows how you can implement your own
// collector. As an example, we will compute a collector
// that computes the standard deviation of a given fast field.
//
// Of course, you can have a look at the tantivy's built-in collectors
// such as the `CountCollector` for more examples.

use std::sync::Arc;
use std::time::Instant;
use futures::executor::block_on;

use roaring::RoaringTreemap;

// ---
// Fast field (i64/u64) value collector, the i64/u64 value(like a docId) is collected into a RoaringBitmap
use tantivy::collector::{Collector, SegmentCollector};
use tantivy::directory::MmapDirectory;
use tantivy::fastfield::Column;
use tantivy::query::TermQuery;
use tantivy::schema::{Schema, FAST, INDEXED, TEXT, IndexRecordOption};
use tantivy::{doc, Index, Score, SegmentReader, Term};

#[derive(Default)]
struct Stats {
    bitmap: RoaringTreemap,
}

impl Stats {
    fn stat_self(self) -> Option<Stats> {
        Some(self)
    }
}

struct FastFieldCollector {
    field: String,
}

impl FastFieldCollector {
    fn with_field(field: String) -> FastFieldCollector {
        FastFieldCollector { field: field}
    }
}

impl Collector for FastFieldCollector {
    // That's the type of our result.
    // Our standard deviation will be a float.
    type Fruit = Option<Stats>;

    type Child = FastFieldSegmentCollector;

    fn for_segment(
        &self,
        _segment_local_id: u32,
        segment_reader: &SegmentReader,
    ) -> tantivy::Result<FastFieldSegmentCollector> {
        let fast_field_reader = segment_reader.fast_fields().i64(self.field.as_str())?;
        
        Ok(FastFieldSegmentCollector {
            fast_field_reader,
            stats: Stats::default(),
        })
    }

    fn requires_scoring(&self) -> bool {
        // this collector does not care about score.
        false
    }

    fn merge_fruits(&self, segment_stats: Vec<Option<Stats>>) -> tantivy::Result<Option<Stats>> {
        let mut stats = Stats::default();
        for segment_stats in segment_stats.into_iter().flatten() {
            stats.bitmap  = stats.bitmap | segment_stats.bitmap;
        }
        Ok(stats.stat_self())
    }
}

struct FastFieldSegmentCollector {
    fast_field_reader: Arc<dyn Column<i64>>,
    stats: Stats,
}

impl SegmentCollector for FastFieldSegmentCollector {
    type Fruit = Option<Stats>;

    fn collect(&mut self, doc: u32, _score: Score) {
        let value = self.fast_field_reader.get_val(doc) as i64;
        self.stats.bitmap.insert(value.unsigned_abs());
    }

    fn harvest(self) -> <Self as SegmentCollector>::Fruit {
        self.stats.stat_self()
    }
}

fn main() -> tantivy::Result<()> {

    // first we need to define a schema ...
    let mut schema_builder = Schema::builder();

    // We'll assume a fictional index containing
    // products, and with a name, a description, and a price.
    let doc_id = schema_builder.add_i64_field("_docId", INDEXED | FAST);
    let stock = schema_builder.add_u64_field("stock", INDEXED | FAST);
    let product_name = schema_builder.add_text_field("name", TEXT);
    let product_description = schema_builder.add_text_field("description", TEXT);
    let price = schema_builder.add_u64_field("price", INDEXED | FAST);
    let schema = schema_builder.build();

    let index_path = std::path::Path::new("/tmp/collector_benchmark/");
    let mut create_new = false;
    if !index_path.exists() {
        std::fs::create_dir_all(index_path)?;

        let mmap_directory = MmapDirectory::open(index_path)?;
        let index = Index::open_or_create(mmap_directory, schema.clone())?;
    
        let mut index_writer = index.writer(50_000_000)?;

        println!("creating index...");
        for i in 0i64..250000i64{
            index_writer.add_document(doc!(
                doc_id => i*4,
                stock => 110u64,
                product_name => "Super Broom 2000",
                product_description => "While it is ok for short distance travel, this broom \
                was designed quiditch. It will up your game.",
                price => 30_200u64
            ))?;
            index_writer.add_document(doc!(
                doc_id => i*4 + 1, 
                stock => 110u64,
                product_name => "Turbulobroom",
                product_description => "You might have heard of this broom before : it is the sponsor of the Wales team.\
                    You'll enjoy its sharp turns, and rapid acceleration",
                price => 29_240u64
            ))?;
            index_writer.add_document(doc!(
                doc_id => i*4 + 2, 
                stock => 110u64,
                product_name => "Broomio",
                product_description => "Great value for the price. This broom is a market favorite",
                price => 21_240u64
            ))?;
            index_writer.add_document(doc!(
                doc_id => i*4 + 3, 
                stock => 110u64,
                product_name => "Whack a Mole",
                product_description => "Prime quality bat.",
                price => 5_200u64
            ))?;
        }
    
        index_writer.commit()?;
        index_writer.wait_merging_threads()?; // wait merging threads for later merging of all segments
        create_new = true;
        
        println!("done creating index...");
    }
    
    // let index = Index::create_in_ram(schema);
    // let index = Index::create_in_dir(&index_path, schema.clone())?;
    // let index = Index::open_in_dir(&index_path)?;
    let mmap_directory = MmapDirectory::open(index_path)?;
    let index = Index::open(mmap_directory)?;

    if create_new {
        // merge all segments:
        let mut index_writer = index.writer(50_000_000)?;
        let segment_ids = index.searchable_segment_ids()?;
        block_on(index_writer.merge(&segment_ids))?;
    }

    let reader = index.reader()?;
    let searcher = reader.searcher();
    
    let start = Instant::now();

    
    let query = TermQuery::new(
    Term::from_field_u64(stock, 110u64),
        IndexRecordOption::Basic,);

    println!("doing query:{:?}", query);

    if let Some(stats) = searcher.search(&query, &FastFieldCollector::with_field(String::from("_docId")))? {
        println!("count: {}", stats.bitmap.len());
    }
    println!("do_filter_query, search duration:{} ", start.elapsed().as_millis());

    Ok(())
}
