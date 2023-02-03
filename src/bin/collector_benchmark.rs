// # Custom collector example
//
// This example shows how you can implement your own
// collector. As an example, we will compute a collector
// that computes the standard deviation of a given fast field.
//
// Of course, you can have a look at the tantivy's built-in collectors
// such as the `CountCollector` for more examples.

use std::ops::Bound;
use std::sync::Arc;
use std::time::Instant;

// ---
// Importing tantivy...
use tantivy::collector::{Collector, SegmentCollector};
use tantivy::directory::MmapDirectory;
use tantivy::fastfield::Column;
use tantivy::query::{QueryParser, RangeQuery, AllQuery};
use tantivy::schema::{Field, Schema, FAST, INDEXED, TEXT};
use tantivy::{doc, Index, Score, SegmentReader};

#[derive(Default)]
struct Stats {
    count: usize,
    sum: f64,
    squared_sum: f64,
}

impl Stats {
    pub fn count(&self) -> usize {
        self.count
    }

    pub fn mean(&self) -> f64 {
        self.sum / (self.count as f64)
    }

    fn square_mean(&self) -> f64 {
        self.squared_sum / (self.count as f64)
    }

    pub fn standard_deviation(&self) -> f64 {
        let mean = self.mean();
        (self.square_mean() - mean * mean).sqrt()
    }

    fn non_zero_count(self) -> Option<Stats> {
        if self.count == 0 {
            None
        } else {
            Some(self)
        }
    }
}

struct StatsCollector {
    field: String,
}

impl StatsCollector {
    fn with_field(field: String) -> StatsCollector {
        StatsCollector { field }
    }
}

impl Collector for StatsCollector {
    // That's the type of our result.
    // Our standard deviation will be a float.
    type Fruit = Option<Stats>;

    type Child = StatsSegmentCollector;

    fn for_segment(
        &self,
        _segment_local_id: u32,
        segment_reader: &SegmentReader,
    ) -> tantivy::Result<StatsSegmentCollector> {
        let fast_field_reader = segment_reader.fast_fields().u64(self.field.as_str())?;
        Ok(StatsSegmentCollector {
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
            stats.count += segment_stats.count;
            stats.sum += segment_stats.sum;
            stats.squared_sum += segment_stats.squared_sum;
        }
        Ok(stats.non_zero_count())
    }
}

struct StatsSegmentCollector {
    fast_field_reader: Arc<dyn Column<u64>>,
    stats: Stats,
}

impl SegmentCollector for StatsSegmentCollector {
    type Fruit = Option<Stats>;

    fn collect(&mut self, doc: u32, _score: Score) {
        let value = self.fast_field_reader.get_val(doc) as f64;
        self.stats.count += 1;
        self.stats.sum += value;
        self.stats.squared_sum += value * value;
    }

    fn harvest(self) -> <Self as SegmentCollector>::Fruit {
        self.stats.non_zero_count()
    }
}

fn main() -> tantivy::Result<()> {

    // first we need to define a schema ...
    let mut schema_builder = Schema::builder();

    // We'll assume a fictional index containing
    // products, and with a name, a description, and a price.
    let doc_id = schema_builder.add_u64_field("_docId", INDEXED | FAST);
    let product_name = schema_builder.add_text_field("name", TEXT);
    let product_description = schema_builder.add_text_field("description", TEXT);
    let price = schema_builder.add_u64_field("price", INDEXED | FAST);
    let schema = schema_builder.build();

    let index_path = std::path::Path::new("/tmp/collector_benchmark/");
    if !index_path.exists() {
        std::fs::create_dir_all(index_path)?;

        let mmap_directory = MmapDirectory::open(index_path)?;
        let index = Index::open_or_create(mmap_directory, schema.clone())?;
    
        let mut index_writer = index.writer(50_000_000)?;

        println!("creating index...");
        for i in 0u64..250000u64{
            index_writer.add_document(doc!(
                doc_id => i*4,
                product_name => "Super Broom 2000",
                product_description => "While it is ok for short distance travel, this broom \
                was designed quiditch. It will up your game.",
                price => 30_200u64
            ))?;
            index_writer.add_document(doc!(
                doc_id => i*4 + 1, 
                product_name => "Turbulobroom",
                product_description => "You might have heard of this broom before : it is the sponsor of the Wales team.\
                    You'll enjoy its sharp turns, and rapid acceleration",
                price => 29_240u64
            ))?;
            index_writer.add_document(doc!(
                doc_id => i*4 + 2, 
                product_name => "Broomio",
                product_description => "Great value for the price. This broom is a market favorite",
                price => 21_240u64
            ))?;
            index_writer.add_document(doc!(
                doc_id => i*4 + 3, 
                product_name => "Whack a Mole",
                product_description => "Prime quality bat.",
                price => 5_200u64
            ))?;
        }
    
        index_writer.commit()?;
        println!("done creating index...");
    }
    
    // let index = Index::create_in_ram(schema);
    // let index = Index::create_in_dir(&index_path, schema.clone())?;
    // let index = Index::open_in_dir(&index_path)?;
    let mmap_directory = MmapDirectory::open(index_path)?;
    let index = Index::open(mmap_directory)?;

    let reader = index.reader()?;
    let searcher = reader.searcher();
    
    let start = Instant::now();

    // let query_parser = QueryParser::for_index(&index, vec![product_name, product_description]);
    // let query = query_parser.parse_query("broom")?;

    let right: Bound<u64> = Bound::Unbounded;
    let left: Bound<u64> = Bound::Excluded(1);
    let range_query = RangeQuery::new_u64_bounds(String::from("_docId"), left, right);
    
    println!("doing query:{:?}", range_query);

    if let Some(stats) = searcher.search(&range_query, &StatsCollector::with_field(String::from("_docId")))? {
        println!("count: {}", stats.count());
        println!("mean: {}", stats.mean());
        println!("standard deviation: {}", stats.standard_deviation());
    }
    println!("do_filter_query, search duration:{} ", start.elapsed().as_millis());

    Ok(())
}
