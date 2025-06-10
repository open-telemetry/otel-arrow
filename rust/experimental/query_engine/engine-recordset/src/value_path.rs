use crate::{data::data_record_resolver::*, error::Error, primitives::any_value::AnyValue};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub(crate) enum ValueSelector {
    ArrayValueIndex(i32),
    ArrayValueInsert(Option<i32>),
    MapValueKey(Box<str>),
    Value,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct ValuePath {
    raw_value: Box<str>,
    selectors: Vec<ValueSelector>,
}

impl ValuePath {
    pub fn new(path: &str) -> Result<ValuePath, Error> {
        ValuePath::parse_path(path)
    }

    pub(crate) fn from_selectors(selectors: &[ValueSelector]) -> ValuePath {
        let mut s: Vec<ValueSelector> = Vec::new();
        s.extend_from_slice(selectors);

        if s.len() == 0 {
            s.push(ValueSelector::Value);
        } else if let ValueSelector::Value = s.last().unwrap() {
        } else {
            s.push(ValueSelector::Value);
        }

        let mut raw = String::new();

        for selector in selectors.iter().rev() {
            match selector {
                ValueSelector::ArrayValueIndex(index) => {
                    raw.push_str(format!("[{index}]").as_str())
                }
                ValueSelector::ArrayValueInsert(index) => match index {
                    Some(index) => raw.push_str(format!("[+{index}]").as_str()),
                    None => raw.push_str(format!("[+]").as_str()),
                },
                ValueSelector::MapValueKey(key) => raw.push_str(format!(":'{key}'").as_str()),
                ValueSelector::Value => {}
            }
        }

        Self {
            raw_value: raw.into(),
            selectors: s,
        }
    }

    pub fn extract_root_property_key(&self) -> Option<(&str, ValuePath)> {
        let selectors = self.get_selectors();

        match selectors.first() {
            Some(first_selector) => {
                if let ValueSelector::MapValueKey(key) = first_selector {
                    return Some((key, ValuePath::from_selectors(&selectors[1..])));
                }

                return None;
            }
            None => None,
        }
    }

    pub fn with_root_property_key(&self, key: &str) -> ValuePath {
        let mut selectors = self.get_selectors().to_vec();
        selectors.insert(0, ValueSelector::MapValueKey(key.into()));
        return ValuePath::from_selectors(&selectors);
    }

    pub fn get_raw_value(&self) -> &str {
        &self.raw_value
    }

    pub(crate) fn get_selectors(&self) -> &Vec<ValueSelector> {
        &self.selectors
    }

    pub fn is_value_selector(&self) -> bool {
        self.get_selectors().len() == 1
    }

    pub fn read<'a>(&self, root_any_value: &'a AnyValue) -> DataRecordReadAnyValueResult<'a> {
        let mut current_value = root_any_value;

        for value_selector in self.selectors.iter() {
            match value_selector {
                ValueSelector::ArrayValueInsert(_) => {
                    return DataRecordReadAnyValueResult::NotFound;
                }
                ValueSelector::ArrayValueIndex(index) => {
                    if let AnyValue::ArrayValue(array_value) = current_value {
                        let mut final_index = *index;

                        if final_index < 0 {
                            final_index = (array_value.get_values().len() as i32) + final_index;
                        }

                        match array_value.get_values().get(final_index as usize) {
                            Some(any_value) => {
                                current_value = any_value;
                            }
                            None => {
                                return DataRecordReadAnyValueResult::NotFound;
                            }
                        }
                    } else {
                        return DataRecordReadAnyValueResult::NotFound;
                    }
                }
                ValueSelector::MapValueKey(key) => {
                    if let AnyValue::MapValue(map_value) = current_value {
                        let k: &str = key;
                        match map_value.get_values().get(k) {
                            Some(any_value) => {
                                current_value = any_value;
                            }
                            None => {
                                return DataRecordReadAnyValueResult::NotFound;
                            }
                        }
                    } else {
                        return DataRecordReadAnyValueResult::NotFound;
                    }
                }
                ValueSelector::Value => {
                    return DataRecordReadAnyValueResult::Found(current_value);
                }
            }
        }

        return DataRecordReadAnyValueResult::NotFound;
    }

    pub fn read_mut<'a>(
        &self,
        root_any_value: &'a mut AnyValue,
    ) -> DataRecordReadMutAnyValueResult<'a> {
        let mut current_value = root_any_value;

        for value_selector in self.selectors.iter() {
            match value_selector {
                ValueSelector::ArrayValueInsert(_) => {
                    return DataRecordReadMutAnyValueResult::NotFound;
                }
                ValueSelector::ArrayValueIndex(index) => {
                    if let AnyValue::ArrayValue(array_value) = current_value {
                        let mut final_index = *index;

                        if final_index < 0 {
                            final_index = (array_value.get_values().len() as i32) + final_index;
                        }

                        match array_value.get_values_mut().get_mut(final_index as usize) {
                            Some(any_value) => {
                                current_value = any_value;
                            }
                            None => {
                                return DataRecordReadMutAnyValueResult::NotFound;
                            }
                        }
                    } else {
                        return DataRecordReadMutAnyValueResult::NotFound;
                    }
                }
                ValueSelector::MapValueKey(key) => {
                    if let AnyValue::MapValue(map_value) = current_value {
                        let k: &str = key;
                        match map_value.get_values_mut().get_mut(k) {
                            Some(any_value) => {
                                current_value = any_value;
                            }
                            None => {
                                return DataRecordReadMutAnyValueResult::NotFound;
                            }
                        }
                    } else {
                        return DataRecordReadMutAnyValueResult::NotFound;
                    }
                }
                ValueSelector::Value => {
                    return DataRecordReadMutAnyValueResult::Found(current_value);
                }
            }
        }

        return DataRecordReadMutAnyValueResult::NotFound;
    }

    pub fn set(
        &self,
        root_any_value: &mut AnyValue,
        any_value_to_set: AnyValue,
    ) -> DataRecordSetAnyValueResult {
        let mut current_value = root_any_value;
        let mut iter = self.selectors.iter().peekable();

        while let Some(value_selector) = iter.next() {
            match value_selector {
                ValueSelector::ArrayValueInsert(index) => {
                    if let AnyValue::ArrayValue(array_value) = current_value {
                        if index.is_none() {
                            array_value.get_values_mut().push(any_value_to_set);
                        } else {
                            let mut final_index = index.unwrap();

                            if final_index < 0 {
                                final_index = (array_value.get_values().len() as i32) + final_index;
                            }

                            if final_index < 0
                                || final_index >= array_value.get_values().len() as i32
                            {
                                return DataRecordSetAnyValueResult::NotFound;
                            }

                            array_value
                                .get_values_mut()
                                .insert(final_index as usize, any_value_to_set);
                        }

                        return DataRecordSetAnyValueResult::Created;
                    } else {
                        return DataRecordSetAnyValueResult::NotFound;
                    }
                }
                ValueSelector::ArrayValueIndex(index) => {
                    if let AnyValue::ArrayValue(array_value) = current_value {
                        let mut final_index = *index;

                        if final_index < 0 {
                            final_index = (array_value.get_values().len() as i32) + final_index;
                        }

                        match array_value.get_values_mut().get_mut(final_index as usize) {
                            Some(any_value) => {
                                current_value = any_value;
                            }
                            None => {
                                return DataRecordSetAnyValueResult::NotFound;
                            }
                        }
                    } else {
                        return DataRecordSetAnyValueResult::NotFound;
                    }
                }
                ValueSelector::MapValueKey(key) => {
                    if let AnyValue::MapValue(map_value) = current_value {
                        let k: &str = key;
                        let values = map_value.get_values_mut();

                        match iter.peek() {
                            Some(v) => {
                                if let ValueSelector::Value = v {
                                    if any_value_to_set.is_null() {
                                        let old_value = values.remove(k);
                                        match old_value {
                                            Some(old_value) => {
                                                return DataRecordSetAnyValueResult::Updated(
                                                    old_value,
                                                );
                                            }
                                            None => return DataRecordSetAnyValueResult::NotFound,
                                        }
                                    } else {
                                        let old_value = values.insert(k.into(), any_value_to_set);
                                        match old_value {
                                            Some(old_value) => {
                                                return DataRecordSetAnyValueResult::Updated(
                                                    old_value,
                                                );
                                            }
                                            None => return DataRecordSetAnyValueResult::Created,
                                        }
                                    }
                                }
                            }
                            None => {}
                        }

                        let value = values.get_mut(k);
                        if !value.is_none() {
                            current_value = value.unwrap();
                            continue;
                        }

                        return DataRecordSetAnyValueResult::NotFound;
                    } else {
                        return DataRecordSetAnyValueResult::NotFound;
                    }
                }
                ValueSelector::Value => {
                    let old_value = current_value.clone();
                    *current_value = any_value_to_set;

                    return DataRecordSetAnyValueResult::Updated(old_value);
                }
            }
        }

        return DataRecordSetAnyValueResult::NotFound;
    }

    pub fn remove(&self, root_any_value: &mut AnyValue) -> DataRecordRemoveAnyValueResult {
        let mut current_value = root_any_value;
        let mut iter = self.selectors.iter().peekable();

        while let Some(value_selector) = iter.next() {
            match value_selector {
                ValueSelector::ArrayValueInsert(_) => {
                    return DataRecordRemoveAnyValueResult::NotFound;
                }
                ValueSelector::ArrayValueIndex(index) => {
                    if let AnyValue::ArrayValue(array_value) = current_value {
                        let mut final_index = *index;

                        if final_index < 0 {
                            final_index = (array_value.get_values().len() as i32) + final_index;
                        }

                        let next = iter.peek();
                        if !next.is_none() {
                            if let ValueSelector::Value = next.unwrap() {
                                if final_index < 0
                                    || final_index >= array_value.get_values().len() as i32
                                {
                                    return DataRecordRemoveAnyValueResult::NotFound;
                                }

                                let old_value =
                                    array_value.get_values_mut().remove(final_index as usize);

                                return DataRecordRemoveAnyValueResult::Removed(old_value);
                            }
                        }

                        match array_value.get_values_mut().get_mut(final_index as usize) {
                            Some(any_value) => {
                                current_value = any_value;
                            }
                            None => {
                                return DataRecordRemoveAnyValueResult::NotFound;
                            }
                        }
                    } else {
                        return DataRecordRemoveAnyValueResult::NotFound;
                    }
                }
                ValueSelector::MapValueKey(key) => {
                    if let AnyValue::MapValue(map_value) = current_value {
                        let k: &str = key;

                        let next = iter.peek();
                        if !next.is_none() {
                            if let ValueSelector::Value = next.unwrap() {
                                let result = map_value.get_values_mut().remove(k);
                                match result {
                                    Some(old_value) => {
                                        return DataRecordRemoveAnyValueResult::Removed(old_value);
                                    }
                                    None => {
                                        return DataRecordRemoveAnyValueResult::NotFound;
                                    }
                                }
                            }
                        }

                        match map_value.get_values_mut().get_mut(k) {
                            Some(any_value) => {
                                current_value = any_value;
                            }
                            None => {
                                return DataRecordRemoveAnyValueResult::NotFound;
                            }
                        }
                    } else {
                        return DataRecordRemoveAnyValueResult::NotFound;
                    }
                }
                ValueSelector::Value => {
                    return DataRecordRemoveAnyValueResult::NotFound;
                }
            }
        }

        return DataRecordRemoveAnyValueResult::NotFound;
    }

    fn parse_path(path: &str) -> Result<ValuePath, Error> {
        let mut results: Vec<ValueSelector> = Vec::new();

        let mut position = 0;
        let mut chars = path.chars();
        let mut terminate = false;

        'outer: loop {
            match chars.next() {
                Some(mut c) => 'inner: loop {
                    if terminate {
                        return Err(Error::PathParseError(format!("Path at position '{position}' contained selectors after an array or map insert operation").to_string()));
                    }
                    if c == '[' {
                        let value_selector = Self::parse_array_selector(&mut position, &mut chars)?;
                        if let ValueSelector::ArrayValueInsert(_) = value_selector {
                            terminate = true;
                        }
                        results.push(value_selector);
                        continue 'outer;
                    } else {
                        let (char_to_retry, value_selector) =
                            Self::parse_key_selector(c, &mut position, &mut chars)?;
                        results.push(value_selector);
                        match char_to_retry {
                            Some(retry_char) => {
                                c = retry_char;
                                continue 'inner;
                            }
                            None => continue 'outer,
                        }
                    }
                },
                None => break,
            }
        }

        if !terminate {
            results.push(ValueSelector::Value);
        }

        return Ok(ValuePath {
            raw_value: path.into(),
            selectors: results,
        });
    }

    fn parse_key_selector(
        first_char: char,
        position: &mut i32,
        chars: &mut std::str::Chars<'_>,
    ) -> Result<(Option<char>, ValueSelector), Error> {
        let start_position = *position;
        *position = *position + 1;

        let mut key = String::new();

        let mut in_quote = false;

        if first_char == '\'' {
            in_quote = true;
        } else if first_char != ':' {
            key.push(first_char);
        }

        loop {
            match chars.next() {
                Some(c) => {
                    *position = *position + 1;
                    if c == '\'' {
                        if in_quote {
                            return build_map_value_key(start_position, &key, None);
                        }
                        if key.len() != 0 {
                            return Err(Error::PathParseError(format!(
                                "Property expression beginning at position '{start_position}' contained an invalid character at position '{}': {c}",
                                *position
                            )));
                        }
                        in_quote = true;
                    } else if in_quote {
                        key.push(c);
                    } else if c == ':' || c == '[' {
                        return build_map_value_key(start_position, &key, Some(c));
                    } else {
                        key.push(c);
                    }
                }
                None => return build_map_value_key(start_position, &key, None),
            }
        }

        fn build_map_value_key(
            start_position: i32,
            key: &str,
            char_to_retry: Option<char>,
        ) -> Result<(Option<char>, ValueSelector), Error> {
            if key.len() == 0 {
                return Err(Error::PathParseError(format!(
                    "Property expression beginning at position '{start_position}' was empty"
                )));
            }

            return Ok((char_to_retry, ValueSelector::MapValueKey(key.into())));
        }
    }

    fn parse_array_selector(
        position: &mut i32,
        chars: &mut std::str::Chars<'_>,
    ) -> Result<ValueSelector, Error> {
        let start_position = *position;
        *position = *position + 1;

        let mut index = String::new();
        let mut insert = false;

        loop {
            match chars.next() {
                Some(c) => {
                    if c == '+' && index.len() == 0 {
                        *position = *position + 1;
                        insert = true;
                    } else if c == ']' {
                        *position = *position + 1;
                        if insert {
                            if index.len() == 0 {
                                return Ok(ValueSelector::ArrayValueInsert(None));
                            } else {
                                match index.parse::<i32>() {
                                    Ok(index) => {
                                        return Ok(ValueSelector::ArrayValueInsert(Some(index)));
                                    }
                                    Err(_) => {
                                        return Err(Error::PathParseError(format!(
                                            "Array index '{index}' for insert expression beginning at position '{start_position}' could not be parsed into int"
                                        )));
                                    }
                                }
                            }
                        } else {
                            match index.parse::<i32>() {
                                Ok(index) => return Ok(ValueSelector::ArrayValueIndex(index)),
                                Err(_) => {
                                    return Err(Error::PathParseError(format!(
                                        "Array index '{index}' for expression beginning at position '{start_position}' could not be parsed into int"
                                    )));
                                }
                            }
                        }
                    } else {
                        index.push(c);
                        *position = *position + 1;
                    }
                }
                None => {
                    return Err(Error::PathParseError(format!(
                        "Array expression beginning at position '{start_position}' did not have an ending bracket"
                    )));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ValuePath, value_path::ValueSelector};

    #[test]
    fn test() {
        assert!(ValuePath::parse_path("[a").is_err());
        assert!(ValuePath::parse_path("[").is_err());
        assert!(ValuePath::parse_path("[a]").is_err());
        assert!(ValuePath::parse_path("'hello:world':[0]").is_err());
        assert!(ValuePath::parse_path("hel'lo.world").is_err());
        assert!(ValuePath::parse_path(":").is_err());

        let validate = |path: &str, expected_selectors: &[ValueSelector]| {
            let result = ValuePath::parse_path(path);
            assert!(result.is_ok());
            let selectors = result.as_ref().unwrap().get_selectors();
            assert_eq!(expected_selectors.len(), selectors.len());
        };

        validate("", &[ValueSelector::Value]);
        validate(
            "[0]",
            &[ValueSelector::ArrayValueIndex(-0), ValueSelector::Value],
        );
        validate(
            "[-1]",
            &[ValueSelector::ArrayValueIndex(-1), ValueSelector::Value],
        );
        validate("[+]", &[ValueSelector::ArrayValueInsert(None)]);
        validate("[+0]", &[ValueSelector::ArrayValueInsert(Some(0))]);
        validate("[+-2]", &[ValueSelector::ArrayValueInsert(Some(-2))]);
        validate(
            "hello.world",
            &[
                ValueSelector::MapValueKey("hello.world".into()),
                ValueSelector::Value,
            ],
        );
        validate(
            "hello:world",
            &[
                ValueSelector::MapValueKey("hello".into()),
                ValueSelector::MapValueKey("world".into()),
                ValueSelector::Value,
            ],
        );
        validate(
            "'hello:world'",
            &[
                ValueSelector::MapValueKey("hello.world".into()),
                ValueSelector::Value,
            ],
        );
        validate(
            "'hello:world'[0]",
            &[
                ValueSelector::MapValueKey("hello.world".into()),
                ValueSelector::ArrayValueIndex(0),
                ValueSelector::Value,
            ],
        );
        validate(
            "'hello:world':'two':array[0]",
            &[
                ValueSelector::MapValueKey("hello.world".into()),
                ValueSelector::MapValueKey("two".into()),
                ValueSelector::MapValueKey("array".into()),
                ValueSelector::ArrayValueIndex(0),
                ValueSelector::Value,
            ],
        );
    }
}
