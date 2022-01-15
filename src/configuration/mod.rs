use crate::Error;
use std::{
    collections::{HashMap, VecDeque},
    path::Path,
};

pub struct Configuration {
    options: HashMap<String, String>,
}

enum ContainerState {
    Object,
    Array(usize),
}

impl Configuration {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        if !path.as_ref().exists() {
            return Ok(Configuration {
                options: HashMap::new(),
            });
        }

        let file_contents = match std::fs::read_to_string(path) {
            Ok(string) => string,
            Err(error) => return Err(Error::ReadFileError(error)),
        };

        let mut current_container = String::new();
        let mut current_state = ContainerState::Object;
        let mut container_scope = Vec::new();
        let mut options = HashMap::new();
        let mut iter: VecDeque<char> = file_contents.chars().collect();
        let mut line = 0;
        'main_loop: loop {
            // Ignore whitespace & comments
            'whitespace_loop: loop {
                match iter.pop_front() {
                    Some(c) => {
                        if c == '#' {
                            // Loop until end of line or end of file
                            while let Some(c) = iter.pop_front() {
                                if c == '\n' {
                                    line += 1;
                                    break;
                                }
                            }
                            continue 'whitespace_loop;
                        } else if !c.is_whitespace() {
                            iter.push_front(c);
                            break 'whitespace_loop;
                        } else if c == '\n' {
                            line += 1;
                        }
                    }
                    None => break 'main_loop,
                }
            }

            match current_state {
                ContainerState::Array(array_state) => {
                    match iter.pop_front() {
                        Some(c) => match c {
                            '{' => {
                                // Object

                                // Move down scope
                                container_scope.push((
                                    current_container.clone(),
                                    ContainerState::Array(array_state + 1),
                                ));

                                current_container.push_str(&format!(
                                    "{}{}",
                                    if current_container == "" { "" } else { "." },
                                    array_state
                                ));

                                current_state = ContainerState::Object;
                            }
                            '[' => {
                                // Array

                                // Move down scope
                                container_scope.push((
                                    current_container.clone(),
                                    ContainerState::Array(array_state + 1),
                                ));

                                current_container.push_str(&format!(
                                    "{}{}",
                                    if current_container == "" { "" } else { "." },
                                    array_state
                                ));

                                current_state = ContainerState::Array(0);
                            }
                            ']' => {
                                // End of array

                                // Move scope up
                                let (old_container, old_state) = container_scope
                                    .pop()
                                    .unwrap_or((String::new(), ContainerState::Object));

                                current_container = old_container;
                                current_state = old_state;
                            }
                            _ => return Err(Error::UnexpectedCharacter(c, line)),
                        },
                        None => break 'main_loop,
                    }
                }
                ContainerState::Object => {
                    // Collect item name
                    let mut name = String::new();
                    let mut close = false;

                    while let Some(c) = iter.pop_front() {
                        match c {
                            '#' => return Err(Error::UnexpectedComment(line)),
                            '{' | '[' | ':' | '}' => {
                                close = c == '}';
                                name = name.trim().to_owned();
                                iter.push_front(c);
                                break;
                            }
                            '\n' => return Err(Error::UnexpectedEndOfLine(line)),
                            _ => name.push(c),
                        }
                    }

                    if !close && name == "" {
                        return Err(Error::EmptyKey(line));
                    } else if close && name != "" {
                        return Err(Error::UnexpectedEndOfContainer(line));
                    }

                    // Parse type
                    match iter.pop_front() {
                        Some(c) => match c {
                            '{' => {
                                // Object

                                // Move scope down
                                container_scope
                                    .push((current_container.clone(), ContainerState::Object));

                                // Update current container
                                if current_container != "" {
                                    current_container.push('.');
                                }
                                current_container.push_str(&name);
                            }
                            '[' => {
                                // Array

                                // Move scope down
                                container_scope
                                    .push((current_container.clone(), ContainerState::Object));

                                // Update current container
                                if current_container != "" {
                                    current_container.push('.');
                                }
                                current_container.push_str(&name);

                                // Update current state
                                current_state = ContainerState::Array(0);
                            }
                            ':' => {
                                // Value

                                // Build key
                                let key = if current_container == "" {
                                    name
                                } else {
                                    format!("{}.{}", current_container, name)
                                };

                                // Verify it doesn't exist
                                match options.get(&key) {
                                    Some(_) => return Err(Error::RepeatedOption(key, line)),
                                    None => {}
                                }

                                // Build value
                                let mut value = String::new();
                                while let Some(c) = iter.pop_front() {
                                    match c {
                                        '#' => {
                                            iter.push_front(c);
                                            break;
                                        }
                                        '\n' => {
                                            line += 1;
                                            break;
                                        }
                                        _ => value.push(c),
                                    }
                                }

                                value = value.trim().to_owned();

                                // Insert into options
                                options.insert(key, value);
                            }
                            '}' => {
                                // End of Object

                                // Move scope up
                                let (old_container, old_state) = container_scope
                                    .pop()
                                    .unwrap_or((String::new(), ContainerState::Object));

                                current_container = old_container;
                                current_state = old_state;
                            }
                            _ => panic!("Character is not one of '{{', '[', or ':'"),
                        },
                        None => return Err(Error::UnexpectedEndOfFile),
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
