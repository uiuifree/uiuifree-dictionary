use serde_json::Value;
use uiuifree_dictionary::Dictionary;

#[test]
fn test_01(){
    let locations  = Dictionary::location("東京都渋谷区代々木","");
    println!("{:?}",locations);
    let locations  = Dictionary::location("東京都渋谷区代々木","").fix_location();
    println!("{:?}",locations);
}