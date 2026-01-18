use std::fs;
use std::path::Path;
#[cfg(feature = "clitable")]
use textfsm_rs::CliTable;
use textfsm_rs::TextFSM;

fn get_data_dir() -> String {
    "tests/basic_template/cli".to_string()
}

#[test]
fn test_individual_templates() {
    let data_dir = get_data_dir();
    let paths = fs::read_dir(&data_dir).expect("Could not read data dir");

    for path in paths {
        let path = path.expect("Could not get path").path();
        let file_name = path.file_name().unwrap().to_string_lossy();

        if file_name.ends_with("_template") {
            let template_path = path.to_str().unwrap();
            let example_path = template_path.replace("_template", "_example");

            if Path::new(&example_path).exists() {
                println!("Testing template: {}", template_path);

                let mut textfsm = TextFSM::from_file(template_path)
                    .unwrap_or_else(|_| panic!("Failed to parse template {}", template_path));

                let result = textfsm
                    .parse_file(&example_path, None)
                    .unwrap_or_else(|_| panic!("Failed to parse data file {}", example_path));

                assert!(
                    !result.is_empty(),
                    "Result shouldn't be empty for {}",
                    template_path
                );

                // Basic validation: ensure we have some fields
                for record in result {
                    assert!(
                        !record.fields.is_empty(),
                        "Record fields shouldn't be empty in {}",
                        template_path
                    );
                }
            }
        }
    }
}

#[test]
#[cfg(feature = "clitable")]
fn test_cli_table_index() {
    let data_dir = get_data_dir();
    let index_path = format!("{}/index", data_dir);

    // CliTable::from_file expects the index file to exist
    let cli_table = CliTable::from_file(&index_path).expect("Failed to load CLI table index");

    // Test cases derived from index file content
    let test_cases = vec![
        ("Cisco", "show ip bgp summary", "cisco_bgp_summary_template"),
        ("Cisco", "show version", "cisco_version_template"),
        (
            "Force10",
            "show ip bgp summary",
            "f10_ip_bgp_summary_template",
        ),
        ("Force10", "show version", "f10_version_template"),
        (
            "Juniper",
            "show bgp summary",
            "juniper_bgp_summary_template",
        ),
        ("Juniper", "show version", "juniper_version_template"),
        // Test variable command expansion
        ("Cisco", "sh ip bgp su", "cisco_bgp_summary_template"),
        ("Cisco", "show ve", "cisco_version_template"),
    ];

    for (platform, command, expected_template) in test_cases {
        let result = cli_table.get_template_for_command(platform, command);
        assert!(
            result.is_some(),
            "Failed to match command '{}' for platform '{}'",
            command,
            platform
        );

        let (dir, row) = result.unwrap();
        // The dir returned by CliTable is the directory of the index file
        assert_eq!(dir, data_dir, "Directory mismatch");
        assert!(
            row.templates.contains(&expected_template.to_string()),
            "Template mismatch for '{}'. Expected {}, got {:?}",
            command,
            expected_template,
            row.templates
        );
    }
}
