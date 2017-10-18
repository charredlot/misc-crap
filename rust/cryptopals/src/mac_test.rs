use mac::{sha1_cat_mac, sha1_pad};

pub fn mac_test() {
    println!("sha1_cat_mac {:?}", &sha1_cat_mac("abcd".as_bytes(),
                                                "efgh".as_bytes()));
    let padded = sha1_pad("beep boop".as_bytes());
    println!("sha1_pad {:?} {}", padded, padded.len());
    println!("Finished MAC tests");
}
