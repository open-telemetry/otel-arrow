import sys

content = open('crates/engine/src/context.rs').read()
old_test_code = '''
        let mut custom_attrs = HashMap::new();
        custom_attrs.insert(
            "custom.identity.foo".to_string(),
            TelemetryAttribute::new(AttributeValue::String("bar".to_string())),
        );
'''

new_test_code = '''
        let mut custom_attrs = HashMap::new();
        let _ = custom_attrs.insert(
            "custom.identity.foo".to_string(),
            TelemetryAttribute::new(AttributeValue::String("bar".to_string())),
        );
'''

content = content.replace(old_test_code.strip(), new_test_code.strip())
open('crates/engine/src/context.rs', 'w').write(content)
print('Patched context.rs')
