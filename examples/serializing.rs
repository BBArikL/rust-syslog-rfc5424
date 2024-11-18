use syslog_rfc5424::{parse_message, SyslogMessage};

fn main() -> () {
    let average_message = r#"<14>1 2017-07-26T14:47:35.869952+05:30 my_hostname custom_appname 5678 some_unique_msgid - \u{feff}Some other message"#;

    let m = parse_message(average_message).unwrap();
    let s = serde_json::to_string(&m).unwrap();

    println!("{}", s);

    let ms: SyslogMessage = serde_json::from_str(&s).unwrap();
    assert_eq!(m, ms);
    println!("{:?}", m);
    println!("Deserialized:");
    println!("{:?}", ms);
}
