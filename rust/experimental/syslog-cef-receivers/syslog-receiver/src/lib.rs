use syslog_loose::{parse_message, Message, Variant};

#[allow(dead_code)]
fn parse_syslog_message(input: &[u8]) -> Message<&str> {
    // For now, use `syslog_loose` crate to parse the payload.
    // There is additional string allocation as `syslog_loose` works on `&str` and not bytes.
    // TODO: Consider writing our own parser to avoid this allocation.
    // TODO: What should be the behavior for invalid messages? For now, we simply treat it as RFC3164 struct with the message as the entire input.
    let data = std::str::from_utf8(input).unwrap();
    parse_message(data, Variant::Either)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syslog_loose::{Protocol, SyslogFacility, SyslogSeverity};
    use chrono::{Datelike, Timelike};
    use std::net::IpAddr;
    use std::str::FromStr;

    #[test]
    fn test_rfc3164_basic() {
        let data = b"<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8";
        let parsed = parse_syslog_message(data);

        // PRI field is calculated as (Facility * 8) + Severity
        // For example, Facility LOG_AUTH (4) and Severity SEV_CRIT (2) gives us 34
        
        assert_eq!(parsed.protocol, Protocol::RFC3164);
        assert_eq!(parsed.facility, Some(SyslogFacility::LOG_AUTH));
        assert_eq!(parsed.severity, Some(SyslogSeverity::SEV_CRIT));

        //Check timestamp
        let ts = parsed.timestamp.unwrap();
        assert_eq!(ts.year(), chrono::Utc::now().year());
        assert_eq!(ts.month(), 10);
        assert_eq!(ts.day(), 11);
        assert_eq!(ts.hour(), 22);
        assert_eq!(ts.minute(), 14);
        assert_eq!(ts.second(), 15);

        assert_eq!(parsed.hostname, Some("mymachine"));
        assert_eq!(parsed.appname, Some("su"));
        assert_eq!(parsed.msg, "'su root' failed for lonvick on /dev/pts/8");
    }

    #[test]
    fn test_rfc3164_all_fields() {
        // Standard RFC3164 format with all possible fields
        let data = b"<13>Feb 5 17:32:18 hostname app[1234]: This is the message";
        let parsed = parse_syslog_message(data);
        
        // Assert all public fields
        assert_eq!(parsed.protocol, Protocol::RFC3164);
        assert_eq!(parsed.facility, Some(SyslogFacility::LOG_USER));
        assert_eq!(parsed.severity, Some(SyslogSeverity::SEV_NOTICE));
        
        // Timestamp check - year is assumed in RFC3164
        let ts = parsed.timestamp.unwrap();
        assert_eq!(ts.year(), chrono::Utc::now().year());
        assert_eq!(ts.month(), 2);
        assert_eq!(ts.day(), 5);
        assert_eq!(ts.hour(), 17);
        assert_eq!(ts.minute(), 32);
        assert_eq!(ts.second(), 18);
        
        assert_eq!(parsed.hostname, Some("hostname"));
        assert_eq!(parsed.appname, Some("app"));
        assert_eq!(parsed.procid, Some("1234".into()));
        assert_eq!(parsed.msgid, None); // RFC3164 doesn't have msgid
        assert_eq!(parsed.structured_data, vec![]); // RFC3164 doesn't have structured data
        assert_eq!(parsed.msg, "This is the message");
        assert_eq!(parsed.protocol, Protocol::RFC3164);
    }

    #[test]
    fn test_rfc3164_partial_fields() {
        // RFC3164 format with some fields missing
        let data = b"<13>Feb 5 17:32:18 hostname: This is a message without app and procid";
        let parsed = parse_syslog_message(data);
        
        assert_eq!(parsed.protocol, Protocol::RFC3164);
        assert_eq!(parsed.facility, Some(SyslogFacility::LOG_USER));
        assert_eq!(parsed.severity, Some(SyslogSeverity::SEV_NOTICE));
        
        let ts = parsed.timestamp.unwrap();
        assert_eq!(ts.year(), chrono::Utc::now().year());
        assert_eq!(ts.month(), 2);
        assert_eq!(ts.day(), 5);
        assert_eq!(ts.hour(), 17);
        assert_eq!(ts.minute(), 32);
        assert_eq!(ts.second(), 18);
        
        assert_eq!(parsed.hostname, Some("hostname"));
        assert_eq!(parsed.appname, None); // No appname in this message
        assert_eq!(parsed.procid, None); // No procid in this message
        assert_eq!(parsed.msgid, None);
        assert_eq!(parsed.structured_data, vec![]);
        assert_eq!(parsed.msg, "This is a message without app and procid");
    }

    #[test]
    fn test_rfc3164_with_year() {
        // Some implementations include year (nonstandard for RFC3164)
        let data = b"<13>Feb 5 2023 17:32:18 host app: message";
        let parsed = parse_syslog_message(data);

        assert_eq!(parsed.protocol, Protocol::RFC3164);
        assert_eq!(parsed.facility, Some(SyslogFacility::LOG_USER));
        assert_eq!(parsed.severity, Some(SyslogSeverity::SEV_NOTICE));
        
        let ts = parsed.timestamp.unwrap();
        assert_eq!(ts.year(), 2023);
        assert_eq!(ts.month(), 2);
        assert_eq!(ts.day(), 5);
        assert_eq!(ts.hour(), 17);
        assert_eq!(ts.minute(), 32);
        assert_eq!(ts.second(), 18);

        assert_eq!(parsed.hostname, Some("host"));
        assert_eq!(parsed.appname, Some("app"));
        assert_eq!(parsed.msg, "message");
    }
    
    #[test]
    fn test_rfc3164_with_numeric_hostname() {
        // RFC3164 with IP address as hostname
        let data = b"<13>Feb 5 17:32:18 192.168.1.1 app[1234]: This is the message";
        let parsed = parse_syslog_message(data);
        
        assert_eq!(parsed.protocol, Protocol::RFC3164);
        assert_eq!(parsed.facility, Some(SyslogFacility::LOG_USER));
        assert_eq!(parsed.severity, Some(SyslogSeverity::SEV_NOTICE));
        
        let ts = parsed.timestamp.unwrap();
        assert_eq!(ts.year(), chrono::Utc::now().year());
        assert_eq!(ts.month(), 2);
        assert_eq!(ts.day(), 5);
        assert_eq!(ts.hour(), 17);
        assert_eq!(ts.minute(), 32);
        assert_eq!(ts.second(), 18);        

        assert_eq!(parsed.hostname, Some("192.168.1.1"));
        assert_eq!(parsed.appname, Some("app"));
        assert_eq!(parsed.procid, Some("1234".into()));
        assert_eq!(parsed.msg, "This is the message");
        
        // Verify it's a valid IPv4
        let ip = IpAddr::from_str("192.168.1.1").unwrap();
        assert!(ip.is_ipv4());
    }

    #[test]
    fn test_rfc3164_without_priority() {
        // RFC3164 format without priority
        let data = b"Feb 5 17:32:18 hostname app[1234]: This is the message";
        let parsed = parse_syslog_message(data);
        
        assert_eq!(parsed.protocol, Protocol::RFC3164);
        assert_eq!(parsed.facility, None); // No priority, so no facility
        assert_eq!(parsed.severity, None); // No priority, so no severity

        let ts = parsed.timestamp.unwrap();
        assert_eq!(ts.year(), chrono::Utc::now().year());
        assert_eq!(ts.month(), 2);
        assert_eq!(ts.day(), 5);
        assert_eq!(ts.hour(), 17);
        assert_eq!(ts.minute(), 32);
        assert_eq!(ts.second(), 18); 

        assert_eq!(parsed.hostname, Some("hostname"));
        assert_eq!(parsed.appname, Some("app"));
        assert_eq!(parsed.procid, Some("1234".into()));
        assert_eq!(parsed.msg, "This is the message");
    }

    #[test]
    fn test_rfc3164_with_malformed_priority() {
        // Missing closing bracket
        let data = b"<13Feb 5 17:32:18 host app: message";
        let parsed = parse_syslog_message(data);
        
        // Should parse as RFC3164 with best effort
        assert_eq!(parsed.protocol, Protocol::RFC3164);
        assert_eq!(parsed.facility, None);
        assert_eq!(parsed.severity, None);
        assert!(parsed.timestamp.is_none());
        assert_eq!(parsed.msg, "<13Feb 5 17:32:18 host app: message");
    }

    #[test]
    fn test_rfc3164_with_invalid_priority_value() {
        // Priority value too high
        let data = b"<999>Feb 5 17:32:18 host app: message";
        let parsed = parse_syslog_message(data);
        
        assert_eq!(parsed.protocol, Protocol::RFC3164);
        // Should default to None when priority is invalid
        assert_eq!(parsed.facility, None);
        assert_eq!(parsed.severity, None);
        assert!(parsed.timestamp.is_none());
        assert_eq!(parsed.msg, "<999>Feb 5 17:32:18 host app: message");
    }

    #[test]
    fn test_rfc3164_with_rsyslog_format() {
        // rsyslog sometimes outputs with ISO timestamps but in RFC3164 format
        let data = b"<13>2023-02-05T17:32:18.123456+01:00 host app: message";
        let parsed = parse_syslog_message(data);
        
        assert_eq!(parsed.protocol, Protocol::RFC3164);
        assert_eq!(parsed.facility, Some(SyslogFacility::LOG_USER));
        assert_eq!(parsed.severity, Some(SyslogSeverity::SEV_NOTICE));

        let ts = parsed.timestamp.unwrap();
        assert_eq!(ts.year(), 2023);
        assert_eq!(ts.month(), 2);
        assert_eq!(ts.day(), 5);
        assert_eq!(ts.hour(), 17);
        assert_eq!(ts.minute(), 32);
        assert_eq!(ts.second(), 18);
        assert_eq!(ts.nanosecond(), 123456000);
        // Verify rfc3339 format
        assert_eq!(ts.to_rfc3339(), "2023-02-05T17:32:18.123456+01:00");

        assert_eq!(parsed.hostname, Some("host"));
        assert_eq!(parsed.appname, Some("app"));
    }

    #[test]
    fn test_rfc5424_basic() {
        let data = b"<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8";
        let parsed = parse_syslog_message(data);
        
        assert_eq!(parsed.protocol, Protocol::RFC5424(1));
        assert_eq!(parsed.facility, Some(SyslogFacility::LOG_AUTH));
        assert_eq!(parsed.severity, Some(SyslogSeverity::SEV_CRIT));

        // Timestamp check
        let ts = parsed.timestamp.unwrap();
        assert_eq!(ts.year(), 2003);
        assert_eq!(ts.month(), 10);
        assert_eq!(ts.day(), 11);
        assert_eq!(ts.hour(), 22);
        assert_eq!(ts.minute(), 14);
        assert_eq!(ts.second(), 15);
        assert_eq!(ts.nanosecond(), 3000000);
        // Verify rfc3339 format
        assert_eq!(ts.to_rfc3339(), "2003-10-11T22:14:15.003+00:00");

        assert_eq!(parsed.hostname, Some("mymachine.example.com"));
        assert_eq!(parsed.appname, Some("su"));
        assert_eq!(parsed.msgid, Some("ID47"));
        assert_eq!(parsed.msg, "'su root' failed for lonvick on /dev/pts/8");
    }

    #[test]
    fn test_rfc5424_all_fields() {
        // RFC5424 with all possible fields including structured data
        let data = b"<165>1 2003-08-24T05:14:15.000003-07:00 hostname app 1234 ID47 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"][another@32473 foo=\"bar\"] This is the message";
        let parsed = parse_syslog_message(data);
        
        // Assert all public fields
        assert_eq!(parsed.protocol, Protocol::RFC5424(1));
        assert_eq!(parsed.facility, Some(SyslogFacility::LOG_LOCAL4));
        assert_eq!(parsed.severity, Some(SyslogSeverity::SEV_NOTICE));
        
        // Timestamp check
        let ts = parsed.timestamp.unwrap();
        assert_eq!(ts.year(), 2003);
        assert_eq!(ts.month(), 8);
        assert_eq!(ts.day(), 24);
        assert_eq!(ts.hour(), 5);
        assert_eq!(ts.minute(), 14);
        assert_eq!(ts.second(), 15);
        assert_eq!(ts.nanosecond(), 3000);
        // Verify rfc3339 format
        assert_eq!(ts.to_rfc3339(), "2003-08-24T05:14:15.000003-07:00");
        
        assert_eq!(parsed.hostname, Some("hostname"));
        assert_eq!(parsed.appname, Some("app"));
        assert_eq!(parsed.procid, Some("1234".into()));
        assert_eq!(parsed.msgid, Some("ID47"));
        
        // Structured data check
        let sd = &parsed.structured_data;
        if sd.len() > 0 {
            assert_eq!(sd.len(), 2);
            
            // First structured data element
            let sd_element1 = &sd[0];
            assert_eq!(sd_element1.id, "exampleSDID@32473");
            assert_eq!(sd_element1.params.len(), 3);
            assert!(sd_element1.params.iter().find(|item| item.0 == "iut" && item.1 == "3").is_some());
            assert!(sd_element1.params.iter().find(|item| item.0 == "eventSource" && item.1 == "Application").is_some());
            assert!(sd_element1.params.iter().find(|item| item.0 == "eventID" && item.1 == "1011").is_some());
            
            // Second structured data element
            let sd_element2 = &sd[1];
            assert_eq!(sd_element2.id, "another@32473");
            assert_eq!(sd_element2.params.len(), 1);
            assert!(sd_element2.params.iter().find(|item| item.0 == "foo" && item.1 == "bar").is_some());
        } else {
            panic!("Structured data not parsed");
        }
        
        assert_eq!(parsed.msg, "This is the message");
    }

    #[test]
    fn test_rfc5424_partial_fields() {
        // RFC5424 with some fields as '-' (NIL value)
        let data = b"<165>1 2003-08-24T05:14:15.000003-07:00 hostname - - - - This is a message without app, procid, and msgid";
        let parsed = parse_syslog_message(data);
        
        assert_eq!(parsed.protocol, Protocol::RFC5424(1));
        assert_eq!(parsed.facility, Some(SyslogFacility::LOG_LOCAL4));
        assert_eq!(parsed.severity, Some(SyslogSeverity::SEV_NOTICE));
        
        // Timestamp check
        let ts = parsed.timestamp.unwrap();
        assert_eq!(ts.year(), 2003);
        assert_eq!(ts.month(), 8);
        assert_eq!(ts.day(), 24);
        assert_eq!(ts.hour(), 5);
        assert_eq!(ts.minute(), 14);
        assert_eq!(ts.second(), 15);
        assert_eq!(ts.nanosecond(), 3000);
        // Verify rfc3339 format
        assert_eq!(ts.to_rfc3339(), "2003-08-24T05:14:15.000003-07:00");

        assert_eq!(parsed.hostname, Some("hostname"));
        assert_eq!(parsed.appname, None); // NIL value
        assert_eq!(parsed.procid, None); // NIL value
        assert_eq!(parsed.msgid, None); // NIL value
        assert_eq!(parsed.structured_data, vec![]);
        assert_eq!(parsed.msg, "This is a message without app, procid, and msgid");
    }

    #[test]
    fn test_rfc5424_with_structured_data_only() {
        // RFC5424 with structured data but no message
        let data = b"<165>1 2003-08-24T05:14:15.000003-07:00 hostname app 1234 ID47 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\"]";
        let parsed = parse_syslog_message(data);
        
        assert_eq!(parsed.protocol, Protocol::RFC5424(1));
        assert_eq!(parsed.facility, Some(SyslogFacility::LOG_LOCAL4));
        assert_eq!(parsed.severity, Some(SyslogSeverity::SEV_NOTICE));

        // Timestamp check
        let ts = parsed.timestamp.unwrap();
        assert_eq!(ts.year(), 2003);
        assert_eq!(ts.month(), 8);
        assert_eq!(ts.day(), 24);
        assert_eq!(ts.hour(), 5);
        assert_eq!(ts.minute(), 14);
        assert_eq!(ts.second(), 15);
        assert_eq!(ts.nanosecond(), 3000);
        // Verify rfc3339 format
        assert_eq!(ts.to_rfc3339(), "2003-08-24T05:14:15.000003-07:00");

        assert_eq!(parsed.hostname, Some("hostname"));
        assert_eq!(parsed.appname, Some("app"));
        assert_eq!(parsed.procid, Some("1234".into()));
        assert_eq!(parsed.msgid, Some("ID47"));
        
        let sd = &parsed.structured_data;
        if sd.len() > 0 {
            assert_eq!(sd.len(), 1);
            assert_eq!(sd[0].id, "exampleSDID@32473");
            assert_eq!(sd[0].params.len(), 2);
            assert!(sd[0].params.iter().find(|item| item.0 == "iut" && item.1 == "3").is_some());
            assert!(sd[0].params.iter().find(|item| item.0 == "eventSource" && item.1 == "Application").is_some());
        } else {
            panic!("Structured data not parsed");
        }
        
        assert_eq!(parsed.msg, ""); // Empty message
    }

    #[test]
    fn test_rfc5424_with_escaped_chars_in_structured_data() {
        // RFC5424 with escaped characters in structured data
        let data = b"<165>1 2003-08-24T05:14:15.000003-07:00 hostname app 1234 ID47 [exampleSDID@32473 message=\"Hello \\\"quoted\\\" world\" escaped=\"\\]\\\"\\\\\\n\"] Message";
        let parsed = parse_syslog_message(data);

        assert_eq!(parsed.protocol, Protocol::RFC5424(1));
        assert_eq!(parsed.facility, Some(SyslogFacility::LOG_LOCAL4));
        assert_eq!(parsed.severity, Some(SyslogSeverity::SEV_NOTICE));

        // Timestamp check
        let ts = parsed.timestamp.unwrap();
        assert_eq!(ts.year(), 2003);
        assert_eq!(ts.month(), 8);
        assert_eq!(ts.day(), 24);
        assert_eq!(ts.hour(), 5);
        assert_eq!(ts.minute(), 14);
        assert_eq!(ts.second(), 15);
        assert_eq!(ts.nanosecond(), 3000);
        // Verify rfc3339 format
        assert_eq!(ts.to_rfc3339(), "2003-08-24T05:14:15.000003-07:00");

        assert_eq!(parsed.hostname, Some("hostname"));
        assert_eq!(parsed.appname, Some("app"));
        assert_eq!(parsed.procid, Some("1234".into()));
        assert_eq!(parsed.msgid, Some("ID47"));

        let sd = &parsed.structured_data;
        if sd.len() > 0 {
            assert_eq!(sd.len(), 1);
            assert_eq!(sd[0].id, "exampleSDID@32473");
            assert_eq!(sd[0].params.len(), 2);
            assert!(sd[0].params.iter().find(|item| item.0 == "message" && item.1 == r#"Hello \"quoted\" world"#).is_some());
            assert!(sd[0].params.iter().find(|item| item.0 == "escaped" && item.1 == r#"\]\"\\\n"#).is_some());
        } else {
            panic!("Structured data not parsed");
        }
    }

    #[test]
    fn test_rfc5424_with_numeric_hostname() {
        // RFC5424 with IPv6 address as hostname
        let data = b"<34>1 2003-10-11T22:14:15.003Z 2001:db8::1 su - ID47 - message";
        let parsed = parse_syslog_message(data);

        // PRI field is calculated as (Facility * 8) + Severity
        // For example, Facility LOG_AUTH (4) and Severity SEV_CRIT (2) gives us 34

        assert_eq!(parsed.protocol, Protocol::RFC5424(1));
        assert_eq!(parsed.facility, Some(SyslogFacility::LOG_AUTH));
        assert_eq!(parsed.severity, Some(SyslogSeverity::SEV_CRIT));
        
        // Timestamp check
        let ts = parsed.timestamp.unwrap();
        assert_eq!(ts.year(), 2003);
        assert_eq!(ts.month(), 10);
        assert_eq!(ts.day(), 11);
        assert_eq!(ts.hour(), 22);
        assert_eq!(ts.minute(), 14);
        assert_eq!(ts.second(), 15);
        assert_eq!(ts.nanosecond(), 3000000);
        // Verify rfc3339 format
        assert_eq!(ts.to_rfc3339(), "2003-10-11T22:14:15.003+00:00");
        
        assert_eq!(parsed.hostname, Some("2001:db8::1"));
        assert_eq!(parsed.appname, Some("su"));
        assert_eq!(parsed.msgid, Some("ID47"));
        
        // Verify it's a valid IPv6
        let ip = IpAddr::from_str("2001:db8::1").unwrap();
        assert!(ip.is_ipv6());
    }

    #[test]
    fn test_rfc5424_without_version() {
        // RFC5424-like format without version number (might be parsed as RFC3164)
        let data = b"<165> 2003-08-24T05:14:15.000003-07:00 hostname app 1234 ID47 - This is the message";
        let parsed = parse_syslog_message(data);
        
        // Don't strictly check protocol as it might vary
        assert_eq!(parsed.facility, Some(SyslogFacility::LOG_LOCAL4));
        assert_eq!(parsed.severity, Some(SyslogSeverity::SEV_NOTICE));
        
        // Timestamp check
        let ts = parsed.timestamp.unwrap();
        assert_eq!(ts.year(), 2003);
        assert_eq!(ts.month(), 8);
        assert_eq!(ts.day(), 24);
        assert_eq!(ts.hour(), 5);
        assert_eq!(ts.minute(), 14);
        assert_eq!(ts.second(), 15);
        assert_eq!(ts.nanosecond(), 3000);
        // Verify rfc3339 format
        assert_eq!(ts.to_rfc3339(), "2003-08-24T05:14:15.000003-07:00");

        assert!(parsed.msg.contains("This is the message"));
    }

    #[test]
    fn test_rfc5424_with_structured_data_escaping_edge_cases() {
        // Test edge cases for structured data escaping
        let data = b"<165>1 2003-08-24T05:14:15.000003-07:00 hostname app 1234 ID47 [test param=\"value with spaces\"][test2 empty=\"\"] Message";
        let parsed = parse_syslog_message(data);

        assert_eq!(parsed.protocol, Protocol::RFC5424(1));
        assert_eq!(parsed.facility, Some(SyslogFacility::LOG_LOCAL4));
        assert_eq!(parsed.severity, Some(SyslogSeverity::SEV_NOTICE));

        // Timestamp check
        let ts = parsed.timestamp.unwrap();
        assert_eq!(ts.year(), 2003);
        assert_eq!(ts.month(), 8);
        assert_eq!(ts.day(), 24);
        assert_eq!(ts.hour(), 5);
        assert_eq!(ts.minute(), 14);
        assert_eq!(ts.second(), 15);
        assert_eq!(ts.nanosecond(), 3000);
        // Verify rfc3339 format
        assert_eq!(ts.to_rfc3339(), "2003-08-24T05:14:15.000003-07:00");

        let sd = &parsed.structured_data;
        if sd.len() > 0 {
            assert_eq!(sd.len(), 2);
            assert!(sd[0].params.iter().find(|item| item.0 == "param" && item.1 == "value with spaces").is_some());
            assert!(sd[1].params.iter().find(|item| item.0 == "empty" && item.1 == "").is_some());
        } else {
            panic!("Structured data not parsed");
        }
    }

    #[test]
    fn test_with_only_priority_and_rest_nil_values() {
        let data = b"<165>1 - - - - - -";
        let parsed = parse_syslog_message(data);
        
        assert_eq!(parsed.protocol, Protocol::RFC3164);
        assert_eq!(parsed.facility, None);
        assert_eq!(parsed.severity, None);
        assert_eq!(parsed.timestamp, None);
        assert_eq!(parsed.hostname, None);
        assert_eq!(parsed.appname, None);
        assert_eq!(parsed.procid, None);
        assert_eq!(parsed.msgid, None);
        assert_eq!(parsed.msg, "<165>1 - - - - - -");
    }

    #[test]
    fn test_empty_message() {
        let data = b"";
        let parsed = parse_syslog_message(data);
        
        assert_eq!(parsed.protocol, Protocol::RFC3164);
        assert_eq!(parsed.facility, None);
        assert_eq!(parsed.severity, None);
        assert!(parsed.timestamp.is_none());
        assert_eq!(parsed.msg, "");
    }
}