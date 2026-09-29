//! Simple test to understand yrs API

use yrs::{Doc, Transact};

#[test]
fn test_yrs_basic() {
    let doc = Doc::new();
    
    // Try to understand the API
    let txn = doc.transact();
    
    // What methods does doc have?
    // doc.get_or_insert_map - exists
    // doc.get_map - doesn't exist?
    
    let map = doc.get_or_insert_map("test");
    
    // What type is map? MapRef
    // What methods does MapRef have?
    // map.insert - needs &mut TransactionMut
    
    // How to get TransactionMut?
    // Maybe txn is Transaction, not TransactionMut
    
    // Let's try to see what we can do
    println!("Have doc and map");
}
