import sys

content = open('crates/engine/src/context.rs').read()
old_test_code = '''
        let has_custom_attr = registry.visit_entity(entity_key, |attrs| {
            let mut found = false;
            for (key, _val) in attrs.iter_attributes() {
                if key == "custom.identity.foo" {
                    found = true;
                }
            }
            found
        }).unwrap_or(false);
'''

new_test_code = '''
        let has_custom_attr = registry.visit_entity(entity_key, |attrs| {
            let mut found = false;
            for (key, val) in attrs.iter_attributes() {
                if key == "custom" {
                    if format!("{:?}", val).contains("custom.identity.foo") {
                        found = true;
                    }
                }
            }
            found
        }).unwrap_or(false);
'''

content = content.replace(old_test_code.strip(), new_test_code.strip())
open('crates/engine/src/context.rs', 'w').write(content)
print('Patched context.rs for test assertion')
