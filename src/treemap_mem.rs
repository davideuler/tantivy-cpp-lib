extern crate roaring;
use roaring::RoaringTreemap;

// use std::{thread, time::Duration};

fn main() {
    // 1000w: 2m, 4000w: 5.6m
    let rb1 = (0..10_00_000).collect::<RoaringTreemap>();
    println!("rb1.len:{} ", rb1.len() );

    for i in 0..10_00_000{
        let _= rb1.contains(i);
        // println!("{b}");
    }
    // 2m
    // let rb1 = (0..20_000_000).collect::<RoaringTreemap>();

    // let rb2 = (20_000_000..40_000_000).collect::<RoaringTreemap>();

    // println!("rb1.len:{} rb2.len:{}", rb1.len(), rb2.len() );

    // thread::sleep(Duration::from_secs(400));

}
