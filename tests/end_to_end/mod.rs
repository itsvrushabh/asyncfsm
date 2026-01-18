use textfsm_rs::TextFSM;

#[test]
fn test_end_to_end() {
    let mut textfsm = TextFSM::from_file("tests/end_to_end/sample.template").unwrap();
    let result = textfsm
        .parse_file("tests/end_to_end/sample.data", None)
        .unwrap();

    assert_eq!(result.len(), 2);

    let record1 = &result[0];
    let fields1 = &record1.fields;
    assert_eq!(fields1.get("Name").unwrap().to_string(), "John");
    assert_eq!(fields1.get("Age").unwrap().to_string(), "42");

    let record2 = &result[1];
    let fields2 = &record2.fields;
    assert_eq!(fields2.get("Name").unwrap().to_string(), "Jane");
    assert_eq!(fields2.get("Age").unwrap().to_string(), "34");
}
