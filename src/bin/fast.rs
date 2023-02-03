use std::collections::HashSet;

use tantivy::collector::{DocSetCollector};

use tantivy::collector::{TopDocs, FilterCollector};
use tantivy::collector::{Collector, SegmentCollector};
use tantivy::{DocAddress, DocId, Score};


/// Collectors that returns the set of DocAddress that matches the query.
///
/// This collector is mostly useful for tests.
pub struct FastDocCollector;

impl Collector for FastDocCollector {
    type Fruit = HashSet<DocAddress>;
    type Child = FastDocChildCollector;

    fn for_segment(
        &self,
        segment_local_id: tantivy::SegmentOrdinal,
        _segment: &tantivy::SegmentReader,
    ) -> tantivy::Result<Self::Child> {
        Ok(FastDocChildCollector {
            segment_local_id,
            docs: HashSet::new(),
        })
    }

    fn requires_scoring(&self) -> bool {
        false
    }

    fn merge_fruits(
        &self,
        segment_fruits: Vec<(u32, HashSet<DocId>)>,
    ) -> tantivy::Result<Self::Fruit> {
        // let len: usize = segment_fruits.iter().map(|(_, docset)| docset.len()).sum();
        let mut result = HashSet::with_capacity(128);
        for (segment_local_id, docs) in segment_fruits {
            for doc in docs {
                // result.insert(DocAddress::new(segment_local_id, doc));
            }
        }
        Ok(result)
    }
}

pub struct FastDocChildCollector {
    segment_local_id: u32,
    docs: HashSet<DocId>,
}

impl SegmentCollector for FastDocChildCollector {
    type Fruit = (u32, HashSet<DocId>);

    fn collect(&mut self, doc: tantivy::DocId, _score: Score) {
        self.docs.insert(doc);
    }

    fn harvest(self) -> (u32, HashSet<DocId>) {
        (self.segment_local_id, self.docs)
    }
}

fn main(){
    
}
