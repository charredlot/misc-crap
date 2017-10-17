use mac::sha1_cat_mac;

pub fn mac_test() {
    println!("sha1_cat_mac {:?}", &sha1_cat_mac("abcd".as_bytes(),
                                                "efgh".as_bytes()));
    println!("Finished MAC tests");
}
