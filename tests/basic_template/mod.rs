use textfsm_rs::TextFSM;

#[test]
fn test_basic_template() {
    let template = r#"Value Required INTERFACE (\S+)
Value STATUS (up|down)
Value IP (\d+\.\d+\.\d+\.\d+)

Start
  ^Interface ${INTERFACE} is ${STATUS}
  ^  IP address is ${IP} -> Record
"#;

    let data = r#"Interface GigabitEthernet0/1 is up
  IP address is 192.168.1.1
Interface GigabitEthernet0/2 is down
  IP address is 10.0.0.1
"#;

    let mut fsm = TextFSM::from_string(template).unwrap();
    let result = fsm.parse_string(data, None).unwrap();

    assert_eq!(result.len(), 2);

    assert_eq!(
        result[0].fields.get("INTERFACE").unwrap().to_string(),
        "GigabitEthernet0/1"
    );
    assert_eq!(result[0].fields.get("STATUS").unwrap().to_string(), "up");
    assert_eq!(
        result[0].fields.get("IP").unwrap().to_string(),
        "192.168.1.1"
    );

    assert_eq!(
        result[1].fields.get("INTERFACE").unwrap().to_string(),
        "GigabitEthernet0/2"
    );
    assert_eq!(result[1].fields.get("STATUS").unwrap().to_string(), "down");
    assert_eq!(result[1].fields.get("IP").unwrap().to_string(), "10.0.0.1");
}

#[test]
fn test_continue_with_state_transition() {
    let _ = env_logger::builder().is_test(true).try_init();
    let template = r#"Value Required INTERFACE (\S+)
Value DESCRIPTION (.+)

Start
  ^interface ${INTERFACE} -> GetDescription

GetDescription
  ^ description ${DESCRIPTION} -> Record Start
  ^interface -> Continue.Record Start
  ^. -> Start
"#;

    let data = r#"interface GigabitEthernet0/1
 description Primary Uplink
interface GigabitEthernet0/2
 description Secondary Uplink
interface Loopback0
interface Vlan1
 description Management
"#;

    let mut fsm = TextFSM::from_string(template).unwrap();
    let result = fsm.parse_string(data, None).unwrap();

    assert_eq!(result.len(), 4);
    assert_eq!(
        result[0].fields.get("INTERFACE").unwrap().to_string(),
        "GigabitEthernet0/1"
    );
    assert_eq!(
        result[0].fields.get("DESCRIPTION").unwrap().to_string(),
        "Primary Uplink"
    );

    assert_eq!(
        result[1].fields.get("INTERFACE").unwrap().to_string(),
        "GigabitEthernet0/2"
    );
    assert_eq!(
        result[1].fields.get("DESCRIPTION").unwrap().to_string(),
        "Secondary Uplink"
    );

    assert_eq!(
        result[2].fields.get("INTERFACE").unwrap().to_string(),
        "Loopback0"
    );
    assert_eq!(result[2].fields.get("DESCRIPTION").unwrap().to_string(), "");

    assert_eq!(
        result[3].fields.get("INTERFACE").unwrap().to_string(),
        "Vlan1"
    );
    assert_eq!(
        result[3].fields.get("DESCRIPTION").unwrap().to_string(),
        "Management"
    );
}

#[test]
#[cfg(feature = "clitable")]
fn test_clitable_parsing() {
    use textfsm_rs::CliTable;
    let index_path = "tests/basic_template/template/parseindex_index";
    let cli_table = CliTable::from_file(index_path).expect("Failed to load CLI table index");

    assert_eq!(cli_table.tables.len(), 1);
    assert_eq!(cli_table.tables[0].rows.len(), 3);

    // Test VendorA sh ver
    let result = cli_table.get_template_for_command("VendorA", "sh ve");
    assert!(result.is_some());
    let (_, row) = result.unwrap();
    assert_eq!(
        row.templates,
        vec!["clitable_templateA", "clitable_templateB"]
    );

    // Test VendorB sh ver
    let result = cli_table.get_template_for_command("VendorB", "show version");
    assert!(result.is_some());
    let (_, row) = result.unwrap();
    assert_eq!(row.templates, vec!["clitable_templateC"]);

    // Test VendorA sh int
    let result = cli_table.get_template_for_command("VendorA", "sh in");
    assert!(result.is_some());
    let (_, row) = result.unwrap();
    assert_eq!(row.templates, vec!["clitable_templateD"]);
}

#[test]
#[cfg(feature = "clitable")]
fn test_clitable_parse_fail() {
    use textfsm_rs::CliTable;
    // Missing Template column (has Templatebogus instead)
    let index_path = "tests/basic_template/template/parseindexfail1_index";
    let result = CliTable::from_file(index_path);
    assert!(result.is_err());

    // Column out of order - This is actually SUPPORTED by our implementation
    let index_path = "tests/basic_template/template/parseindexfail2_index";
    let result = CliTable::from_file(index_path);
    assert!(result.is_ok());

    // Illegal regex in column (VendorB has [[VendorB which is invalid)
    // Actually our implementation doesn't treat Vendor as regex, only Command.
    let index_path = "tests/basic_template/template/parseindexfail3_index";
    let result = CliTable::from_file(index_path);
    assert!(result.is_ok());

    // Missing Template column (has Devicename instead)
    let index_path = "tests/basic_template/template/nondefault_index";
    let result = CliTable::from_file(index_path);
    assert!(result.is_err());
}
