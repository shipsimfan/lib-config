use crate::Error;
use std::{
    collections::{HashMap, VecDeque},
    path::Path,
};

pub struct Configuration {
    options: HashMap<String, String>,
}

impl Configuration {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file_contents = match std::fs::read_to_string(path) {
            Ok(string) => string,
            Err(error) => return Err(Error::ReadFileError(error)),
        };

        let mut current_container = String::new();
        let mut container_scope = Vec::new();
        let mut options = HashMap::new();
        let mut iter: VecDeque<char> = file_contents.chars().collect();
        let mut line = 0;
        'main_loop: loop {
            // Ignore whitespace
            'whitespace_loop: loop {
                match iter.pop_front() {
                    Some(c) => {
                        if !c.is_whitespace() {
                            iter.push_front(c);
                            break 'whitespace_loop;
                        } else if c == '\n' {
                            line += 1;
                        }
                    }
                    None => break 'whitespace_loop,
                }
            }

            // Check first character
            let c = match iter.pop_front() {
                Some(c) => c,
                None => break 'main_loop,
            };

            match c {
                '[' => {
                    // Collect container name
                    let mut name = String::new();
                    'name_loop: loop {
                        let c = match iter.pop_front() {
                            Some(c) => c,
                            None => break 'name_loop,
                        };

                        if c == '\n' {
                            break 'name_loop;
                        }

                        if c.is_whitespace() {
                            return Err(Error::InvalidContainer(line));
                        }

                        name.push(c);
                    }

                    if name == "" {
                        return Err(Error::InvalidContainer(line));
                    }

                    container_scope.push(name);
                    current_container = String::new();
                    let mut first = true;
                    for part in &container_scope {
                        if !first {
                            current_container.push('.');
                        } else {
                            first = false;
                        }

                        current_container.push_str(part);
                    }
                } // Start of container
                ']' => match container_scope.pop() {
                    Some(_) => {
                        current_container = String::new();
                        let mut first = true;
                        for part in &container_scope {
                            if !first {
                                current_container.push('.');
                            } else {
                                first = false;
                            }

                            current_container.push_str(part);
                        }
                    }
                    None => {} // Ignore mismatched brackets
                }, // End of container
                _ => {
                    // Start of option
                    let mut found_equals = false;
                    let mut name = format!("{}", c);
                    loop {
                        match iter.pop_front() {
                            Some(c) => {
                                if c == ' ' || c == '=' {
                                    if c == '=' {
                                        found_equals = true;
                                    }

                                    break;
                                } else if c == '\n' {
                                    return Err(Error::InvalidOption(line));
                                }

                                name.push(c);
                            }
                            None => return Err(Error::InvalidOption(line)),
                        }
                    }
                    name = name.trim().to_owned();

                    if !found_equals {
                        // Search for equals
                        while let Some(c) = iter.pop_front() {
                            if c == '\n' {
                                return Err(Error::InvalidOption(line));
                            }

                            if c.is_whitespace() {
                                continue;
                            }

                            if c == '=' {
                                break;
                            }
                        }
                    }

                    // Ignore whitespace
                    while let Some(c) = iter.pop_front() {
                        if c == '\n' {
                            return Err(Error::InvalidOption(line));
                        }

                        if !c.is_whitespace() {
                            iter.push_front(c);
                            break;
                        }
                    }

                    // Get value
                    let mut value = String::new();
                    'value_loop: loop {
                        let c = match iter.pop_front() {
                            Some(c) => c,
                            None => break 'value_loop,
                        };

                        if c == '\n' {
                            line += 1;
                            break;
                        }

                        value.push(c);
                    }
                    value = value.trim().to_owned();

                    if value == "" {
                        return Err(Error::InvalidOption(line - 1));
                    }

                    let name = if current_container == "" {
                        name
                    } else {
                        format!("{}.{}", current_container, name)
                    };

                    match options.insert(name.clone(), value) {
                        Some(_) => return Err(Error::RepeatedOption(name, line - 1)),
                        None => {}
                    }
                }
            }
        }

        Ok(Configuration { options })
    }

    pub fn get<S: AsRef<str>>(&self, option: S) -> Option<&str> {
        self.options.get(option.as_ref()).map(|s| s.as_str())
    }
}
