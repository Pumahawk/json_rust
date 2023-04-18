use crate::objects as json;

#[derive(Debug)]
pub enum AutomaError {
    Parser(ParserError),
}

impl std::fmt::Display for AutomaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            AutomaError::Parser(error) => write!(f, "{}", error.message())
        }
    }
}

impl std::error::Error for AutomaError {
}

type AutomaResult<T> = Result<T, AutomaError>;

#[derive(Debug)]
pub struct ParserError {
    message: String,
}

impl ParserError {
    pub fn new(message: String) -> ParserError {
        ParserError {
            message,
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.message())
    }
}

impl From<&str> for ParserError {
    fn from(message: &str) -> Self {
        ParserError {
            message: String::from(message),
        }
    }
}

impl From<ParserError> for AutomaError {
    fn from(value: ParserError) -> Self {
        AutomaError::Parser(value)
    }
}

pub trait Automa {
    type Input;
    type Output;
    fn can_start(&self, input: Self::Input) -> bool;
    fn start(&self, iter: &mut dyn Iterator<Item=Self::Input>) -> AutomaResult<Self::Output>;

    fn process(&self, first: Self::Input, iter: &mut  dyn Iterator<Item=Self::Input>) -> AutomaResult<Self::Output> {
        let mut iter = std::iter::once(first).chain(iter);
        self.start(&mut iter)
    }

}

enum StrAtm {
    N1, N2, N3,
}

struct StrAutoma {   
}

impl StrAutoma {
    fn new() -> StrAutoma {
        StrAutoma {
        }
    }
}

impl Automa for StrAutoma {
    type Input = char;
    type Output = String;

    fn can_start(&self, input: Self::Input) -> bool {
        input == '"'
    }

    fn start(&self, iter: &mut dyn Iterator<Item=Self::Input>) -> AutomaResult<Self::Output> {
        let mut status = StrAtm::N1;
        let mut chars = Vec::new();
        for c in iter {
            match status {
                StrAtm::N1 => {
                    match c {
                        '"' => {
                            status = StrAtm::N2;
                        },
                        _ => return Err(ParserError::from("invalid").into()),
                    }
                },
                StrAtm::N2 => {
                    match c {
                        '"' => return Ok(chars.iter().collect()),
                        '\\' => status = StrAtm::N3,
                        c => chars.push(c),
                    }
                },
                StrAtm::N3 => {
                    match c {
                        '\\' => escape(&mut status, &mut chars, '\\'),
                        'n' => escape(&mut status, &mut chars, '\n'),
                        'r' => escape(&mut status, &mut chars, '\r'),
                        't' => escape(&mut status, &mut chars, '\t'),
                        '"' => escape(&mut status, &mut chars, '"'),
                        _ => return Err(ParserError::from("Invalid escape").into()),
                    }

                    fn escape(status: &mut StrAtm, chars: &mut Vec<char>, c: char) {
                        chars.push(c);
                        *status = StrAtm::N2;
                    }
                }
            }
        }
        Err(ParserError::from("unable to retrieve str").into())
    }
}

struct StringAutoma {
    value: String,
}

impl StringAutoma {
}

impl std::convert::From<String> for StringAutoma {
    fn from(value: String) -> Self {
        StringAutoma {
            value,
        }
    }
}

impl std::convert::From<&str> for StringAutoma {
    fn from(value: &str) -> Self {
        StringAutoma::from(String::from(value))
    }
}

impl Automa for StringAutoma {
    type Input = char;
    type Output = ();
    fn can_start(&self, input: Self::Input) -> bool {
        if let Some(v) =self.value.chars().next() { v == input} else { false }
    }
    fn start(&self, iter: &mut dyn Iterator<Item=Self::Input>) -> AutomaResult<Self::Output> {
        let mut i = 0;
        while let Some(c) = iter.next() {
            if self.value.as_bytes()[i] == c as u8 {
                i += 1;
                if i == self.value.len() {
                    return Ok(());
                }
            } else {
                return Err(ParserError::from("Erro String match. Char not equal").into());
            }
        }
        Err(ParserError::from("Invalid StringAutoma parse.").into())
    }
}

enum NumberAtm {
    N1,
    N2,
    N3,
}

struct NumberAutoma;

impl NumberAutoma {
    pub fn new() -> NumberAutoma {
        NumberAutoma {
        }
    }
}

impl Automa for NumberAutoma {
    type Input=char;
    type Output=(f32, Option<Self::Input>);

    fn can_start(&self, input: Self::Input) -> bool {
        is_number(input)
    }

    fn start(&self, iter: &mut dyn Iterator<Item=Self::Input>) -> AutomaResult<Self::Output> {
        let mut status = NumberAtm::N1;
        let mut number_chars = Vec::new();
        while let Some(c) = iter.next() {
            match status {
                NumberAtm::N1 => {
                    match c {
                        c if is_number(c) => {
                            number_chars.push(c);
                            status = NumberAtm::N2;
                        }
                        _ => return Err(ParserError::from("Unable to read first number").into()),
                    }
                },
                NumberAtm::N2 => {
                    match c {
                        c if is_number(c) => {
                            number_chars.push(c);
                        },
                        '.' => {
                            number_chars.push(c);
                            status = NumberAtm::N3;
                        },
                        c => {
                            return Ok((number_chars.iter().collect::<String>().parse().unwrap(), Some(c)));
                        }
                    }
                },
                NumberAtm::N3 => {
                    match c {
                        c if is_number(c) => {
                            number_chars.push(c);
                        },
                        c => {
                            return Ok((number_chars.iter().collect::<String>().parse().unwrap(), Some(c)));
                        }
                    }
                },
            }
        }
        match number_chars.iter().collect::<String>().parse() {
            Ok(number) => Ok((number, None)),
            _ => Err(ParserError::from("Unable to retrieve number").into()),
        }
    }
}

enum ObjectAtm {
    N1, N2, N3, N4, N5,
}

struct ObjectAutoma {
}

impl ObjectAutoma {
    fn new() -> ObjectAutoma {
        ObjectAutoma {
        }
    }
}

impl Automa for ObjectAutoma {
    type Input = char;
    type Output = json::ObjectJson;

    fn can_start(&self, input: Self::Input) -> bool {
        input == '{'
    }

    fn start(&self, iter: &mut dyn Iterator<Item=Self::Input>) -> AutomaResult<Self::Output> {
        let mut iter: Box<dyn Iterator<Item=char>> = Box::new(std::iter::empty().chain(iter));
        let mut status = ObjectAtm::N1;
        let mut key = None;
        let mut json_object = json::object();
        while let Some(c) = iter.next() {
            match status {
                ObjectAtm::N1 => {
                    match c {
                        '{' => {
                            status = ObjectAtm::N2;
                        },
                        _ => return Err(ParserError::from("invalid from node: N1").into())
                    }
                },
                ObjectAtm::N2 => {
                    let str_automa = StrAutoma::new();
                    match c {
                        c if is_space(c) => continue,
                        c if str_automa.can_start(c) => {
                            let result = str_automa.process(c, &mut iter);
                            match result {
                                Ok(k) => {
                                    key = Some(k);
                                }
                                Err(msg) => return Err(msg),
                            }
                            status = ObjectAtm::N3;
                        },
                        _ => return Err(ParserError::from("invalid from node: N2").into())
                    }
                },
                ObjectAtm::N3 => {
                    match c {
                        c if is_space(c) => continue,
                        ':' => {
                            status = ObjectAtm::N4;
                        },
                        other => return Err(ParserError::new(format!("invalid from node: N3. Value: {other}")).into())
                    }
                },
                ObjectAtm::N4 => {
                    let str_automa = StrAutoma::new();
                    let json_automa = ObjectAutoma::new();
                    let array_automa = ArrayAutoma::new();
                    let number_automa = NumberAutoma::new();
                    let null_automa = StringAutoma::from("null");
                    let true_automa = StringAutoma::from("true");
                    let false_automa = StringAutoma::from("false");
                    match c {
                        c if is_space(c) => continue,
                        c if json_automa.can_start(c) => {
                            let result = json_automa.process(c, &mut iter);
                            match result {
                                Ok(value) => {
                                    json_object.set(&key.take().unwrap(), value);
                                },
                                Err(msg) => return Err(msg),
                            }
                            status = ObjectAtm::N5;
                        },
                        c if array_automa.can_start(c) => {
                            let result = array_automa.process(c, &mut iter);
                            match result {
                                Ok(value) => {
                                    json_object.set(&key.take().unwrap(), value);
                                },
                                Err(msg) => return Err(msg),
                            }
                            status = ObjectAtm::N5;
                        },
                        c if null_automa.can_start(c) => {
                            let result = null_automa.process(c, &mut iter);
                            match result {
                                Ok(_) => {
                                    json_object.set(&key.take().unwrap(), json::null());
                                    status = ObjectAtm::N5;
                                },
                                Err(msg) => return Err(msg),
                            }
                        },
                        c if true_automa.can_start(c) => {
                            let result = true_automa.process(c, &mut iter);
                            match result {
                                Ok(_) => {
                                    json_object.set(&key.take().unwrap(), true);
                                    status = ObjectAtm::N5;
                                },
                                Err(msg) => return Err(msg),
                            }
                        },
                        c if false_automa.can_start(c) => {
                            let result = false_automa.process(c, &mut iter);
                            match result {
                                Ok(_) => {
                                    json_object.set(&key.take().unwrap(), false);
                                    status = ObjectAtm::N5;
                                },
                                Err(msg) => return Err(msg),
                            }
                        },
                        c if str_automa.can_start(c) => {
                            let result = str_automa.process(c, &mut iter);
                            match result {
                                Ok(value) => {
                                    json_object.set(&key.take().unwrap(), value);
                                },
                                Err(msg) => return Err(msg),
                            }
                            status = ObjectAtm::N5;
                        },
                        c if number_automa.can_start(c) => {
                            let result = number_automa.process(c, &mut iter);
                            match result {
                                Ok((number, c)) => {
                                    json_object.set(&key.take().unwrap(), number);
                                    status = ObjectAtm::N5;
                                    if let Some(c) = c {
                                        iter = Box::new(std::iter::once(c).chain(iter));
                                    }
                                },
                                Err(msg) => return Err(msg),
                            }
                        }
                        _ => return Err(ParserError::from("invalid from node: N4").into())
                    }
                },
                ObjectAtm::N5 => {
                    match c {
                        c if is_space(c) => continue,
                        '}' => return Ok(json_object),
                        ',' => {
                            status = ObjectAtm::N2;
                        }
                        _ => return Err(ParserError::from("invalid from node: N5").into())
                    }
                },
            }
        }
        Err(ParserError::from("invalid json automa").into())
    }
}

enum ArrayAtm {
    N1,
    N2,
    N3,
}

pub struct ArrayAutoma;

impl ArrayAutoma {
    pub fn new() -> ArrayAutoma {
        ArrayAutoma {}
    }
}

impl Automa for ArrayAutoma {
    type Input = char;
    type Output = json::ListJson;

    fn can_start(&self, input: Self::Input) -> bool {
        input == '['
    }

    fn start(&self, iter: &mut dyn Iterator<Item=Self::Input>) -> AutomaResult<Self::Output> {
        let mut iter: Box<dyn Iterator<Item=char>> = Box::new(std::iter::empty().chain(iter));
        let mut status = ArrayAtm::N1;
        let mut json_array = json::array();
        while let Some(c) = iter.next() {
            match status {
                ArrayAtm::N1 => {
                    match c {
                        '[' => {
                            status = ArrayAtm::N2;
                        },
                        _ => return Err(ParserError::from("Invalid ArrayAtm::N1").into()),
                    }
                },
                ArrayAtm::N2 => {
                    let string_automa = StrAutoma::new();
                    let number_automa = NumberAutoma::new();
                    let object_automa = ObjectAutoma::new();
                    let array_automa = ArrayAutoma::new();
                    let null_automa = StringAutoma::from("null");
                    let false_automa = StringAutoma::from("false");
                    let true_automa = StringAutoma::from("true");
                    match c {
                        ']' => return Ok(json_array),
                        c if is_space(c) => {},
                        c if string_automa.can_start(c) => match string_automa.process(c, &mut iter) {
                            Ok(string) => {
                                json_array.add(string);
                                status = ArrayAtm::N3;
                            },
                            _ => return Err(ParserError::from("Invalid ArrayAtm::N2, string_automa").into()),
                        },
                        c if number_automa.can_start(c) => match number_automa.process(c, &mut iter) {
                            Ok((num, c)) => {
                                json_array.add(num);
                                if let Some(c) = c {
                                    iter = Box::new(std::iter::once(c).chain(iter));
                                }
                                status = ArrayAtm::N3;
                            }
                            _ => return Err(ParserError::from("Invalid ArrayAtm::N2, number_automa").into()),
                        }
                        c if object_automa.can_start(c) => match object_automa.process(c, &mut iter) {
                            Ok(object) => {
                                json_array.add(object);
                                status = ArrayAtm::N3;
                            }
                            _ => return Err(ParserError::from("Invalid ArrayAtm::N2, object_automa").into()),
                        }
                        c if array_automa.can_start(c) => match array_automa.process(c, &mut iter) {
                            Ok(array) => {
                                json_array.add(array);
                                status = ArrayAtm::N3;
                            }
                            _ => return Err(ParserError::from("Invalid ArrayAtm::N2, array_automa").into()),
                        }
                        c if null_automa.can_start(c) => match null_automa.process(c, &mut iter) {
                            Ok(_) => {
                                json_array.add(json::null());
                                status = ArrayAtm::N3;
                            }
                            _ => return Err(ParserError::from("Invalid ArrayAtm::N2, null_automa").into()),
                        }
                        c if true_automa.can_start(c) => {
                            let result = true_automa.process(c, &mut iter);
                            match result {
                                Ok(_) => {
                                    json_array.add(true);
                                    status = ArrayAtm::N3;
                                },
                                Err(msg) => return Err(msg),
                            }
                        },
                        c if false_automa.can_start(c) => {
                            let result = false_automa.process(c, &mut iter);
                            match result {
                                Ok(_) => {
                                    json_array.add(false);
                                    status = ArrayAtm::N3;
                                },
                                Err(msg) => return Err(msg),
                            }
                        },
                        _ => return Err(ParserError::from("Invalid ArrayAtm::N2").into()),
                    }
                },
                ArrayAtm::N3 => {
                    match c {
                        ']' => return Ok(json_array),
                        ',' => {
                            status = ArrayAtm::N2;
                        },
                        c if is_space(c) => {},
                        _ => return Err(ParserError::from("Invalid ArrayAtm::N3").into()),
                    }
                },
            }
        }
        Err(ParserError::from("unable to retrieve array").into())
    }

}

enum KeyParseQueryAtm {
    N1,
    N2,
    N3,
    N4,
    N5,
    N6,
}

pub enum KeyParseQueryToken {
    Key(String),
    Index(usize),
    Error(String)
}

impl From<&str> for KeyParseQueryToken {
    fn from(value: &str) -> Self {
        KeyParseQueryToken::Error(String::from(value))
    }
}

pub struct KeyParseQueryAutoma<'a, T> {
    status: KeyParseQueryAtm,
    chars: Vec<char>,
    iter: &'a mut T,
}

impl <'a, T: Iterator<Item=char>> KeyParseQueryAutoma<'a, T> {
    pub fn new(iter: &mut T) -> KeyParseQueryAutoma<T> {
        KeyParseQueryAutoma {
            status: KeyParseQueryAtm::N1,
            chars: Vec::new(),
            iter,
        }
    }

    fn collect_c(&mut self, c: char) {
        self.chars.push(c);
    }

    fn retrieve_field(&mut self) -> KeyParseQueryToken {
        KeyParseQueryToken::Key(self.chars.drain(..).collect())
    } 

    fn retrieve_number(&mut self) -> KeyParseQueryToken {
        let mut vp = Vec::new();
        vp.append(&mut self.chars);
        match vp.into_iter().collect::<String>().parse() {
            Ok(number) => KeyParseQueryToken::Index(number),
            Err(msg) => KeyParseQueryToken::Error(msg.to_string()),
        }
    } 
}

impl <'a, T: Iterator<Item=char>> Iterator for KeyParseQueryAutoma<'a, T> {
    type Item=KeyParseQueryToken;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let str_automa = StrAutoma::new();
        while let Some(c) = self.iter.next() {
            match &self.status {
                KeyParseQueryAtm::N1 => match c {
                    '.' => self.status = KeyParseQueryAtm::N2,
                    '[' => self.status = KeyParseQueryAtm::N4,
                    _ => return Some("Invalid starter character. Valid: /".into()),
                },
                KeyParseQueryAtm::N2 => match c {
                    c if is_char(c) || is_number(c) => {
                        self.collect_c(c);
                        self.status = KeyParseQueryAtm::N3;
                    },
                    c if str_automa.can_start(c) => {
                        match str_automa.process(c, self.iter) {
                            Ok(key) => {
                                self.status = KeyParseQueryAtm::N1;
                                return Some(KeyParseQueryToken::Key(key));
                            },
                            Err(error) => match error {
                                AutomaError::Parser(msg) => return Some(KeyParseQueryToken::Error(format!("Error reading {}", msg))),
                            }
                        }
                    }
                    _ => return Some("Invalid key string reference. Valid: char".into()),
                },
                KeyParseQueryAtm::N3 => match c {
                    c if is_char(c) || is_number(c) => self.collect_c(c),
                    '.' => {
                        self.status = KeyParseQueryAtm::N2;
                        return Some(self.retrieve_field());
                    },
                    '[' => {
                        self.status = KeyParseQueryAtm::N4;
                        return Some(self.retrieve_field());
                    },
                    _ => return Some("Invalid key string reference.".into()),
                },
                KeyParseQueryAtm::N4 => match c {
                    c if is_number(c) => {
                        self.status = KeyParseQueryAtm::N5;
                        self.collect_c(c);
                    },
                    _ => return Some("Invalid key index reference.".into()),
                },
                KeyParseQueryAtm::N5 => match c {
                    c if is_number(c) => self.collect_c(c),
                    ']' => {
                        self.status = KeyParseQueryAtm::N1;
                        return Some(self.retrieve_number());
                    }
                    _ => return Some("Invalid key index reference.".into()),
                },
                KeyParseQueryAtm::N6 => return None,
            }
        }
        match &self.status {
            KeyParseQueryAtm::N1 => None,
            KeyParseQueryAtm::N6 => None,
            KeyParseQueryAtm::N3 => {
                self.status = KeyParseQueryAtm::N6;
                return Some(self.retrieve_field());
            },
            _ => Some("Invalid EOF status".into()),
        }
    }
}

fn is_space(c: char) -> bool {
    match c {
        ' ' | '\t' | '\n' | '\r' => true,
        _ => false,
    }
}

fn is_number(c: char) -> bool {
    c >= '0' && c <= '9'
}

fn is_char(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
}

pub fn parser(mut iter: impl Iterator<Item=char>) -> AutomaResult<json::ObjectJson> {
    ObjectAutoma::new().start(&mut iter)
}

#[cfg(test)]
mod test {

    use crate::objects::*;
    use crate::automa::*;
    use crate as json;

    #[test]
    fn is_space_test() {
        assert!(is_space(' '));
        assert!(is_space('\t'));
        assert!(is_space('\r'));
        assert!(is_space('\n'));
    }

    #[test]
    fn str_automa() {
        let str_automa = StrAutoma::new();
        let input = String::from("\"input_automa\"");

        if let Ok(msg) = str_automa.start(&mut input.chars()) {
            assert_eq!("input_automa", msg);
        } else {
            assert!(false);
        }

        let input = String::from("\"input_automa\": 1234");
        let mut iter = input.chars();
        let result = str_automa.start(&mut iter).unwrap();
        let rest: String = iter.collect();

        assert_eq!("input_automa", result);
        assert_eq!(": 1234", rest);

        let input = String::from("\"\\n\\\"\"");
        if let Ok(msg) = str_automa.start(&mut input.chars()) {
            assert_eq!("\n\"", msg);
        } else {
            assert!(false);
        }
        
    }

    #[test]
    fn json_automa() {
        let json_autom = ObjectAutoma::new();
        let input = String::from("{\"key\":\"input_automa\"}");

        match json_autom.start(&mut input.chars()) {
            Ok(json_object) => {
                assert_eq!("input_automa", if let TypeJson::Text(msg) = json_object.get("key").unwrap() {msg} else {"none"});
            },
            Err(_) => {
                assert!(false);
            }
        }

        let input = String::from("{\"key1\":\"input_automa_1\",\"key2\":\"input_automa_2\"}");

        match json_autom.start(&mut input.chars()) {
            Ok(json_object) => {
                assert_eq!("input_automa_1", if let TypeJson::Text(msg) = json_object.get("key1").unwrap() {msg} else {"none"});
                assert_eq!("input_automa_2", if let TypeJson::Text(msg) = json_object.get("key2").unwrap() {msg} else {"none"});
            },
            Err(_) => {
                assert!(false);
            }
        }

        let input = String::from("{  \"key1\" \t : \n \"input_automa_1\"  \t,\r \"key2\":\"input_automa_2\"}");

        match json_autom.start(&mut input.chars()) {
            Ok(json_object) => {
                assert_eq!("input_automa_1", if let TypeJson::Text(msg) = json_object.get("key1").unwrap() {msg} else {"none"});
                assert_eq!("input_automa_2", if let TypeJson::Text(msg) = json_object.get("key2").unwrap() {msg} else {"none"});
            },
            Err(_) => {
                assert!(false);
            }
        }

        let input = String::from("{\"key1\":\"input_automa_1\",\"key2\":{\"key\":\"input_automa\"}}");

        match json_autom.start(&mut input.chars()) {
            Ok(json_object) => {
                assert_eq!("input_automa_1", if let TypeJson::Text(msg) = json_object.get("key1").unwrap() {msg} else {"none"});
                let jo = json_object.get("key2").unwrap();
                match jo {
                    TypeJson::Object(v) => {
                        match v.get("key") {
                            Some(TypeJson::Text(msg)) => assert_eq!("input_automa", msg),
                            _ => unreachable!(),
                        }
                    },
                    _ => unreachable!(),
                }
            },
            Err(_) => {
                assert!(false);
            }
        }

        let input = String::from("{\"key1\":\"input_automa_1\",\"key2\": null}");

        match json_autom.start(&mut input.chars()) {
            Ok(json_object) => {
                assert_eq!("input_automa_1", if let TypeJson::Text(msg) = json_object.get("key1").unwrap() {msg} else {"none"});
                assert_eq!("null", if let TypeJson::Null = json_object.get("key2").unwrap() {"null"} else {"none"});
            },
            Err(_) => {
                assert!(false);
            }
        }

        let input = String::from("{\"key1\":\"input_automa_1\",\"key2\": 33.2}");

        match json_autom.start(&mut input.chars()) {
            Ok(json_object) => {
                assert_eq!("input_automa_1", if let TypeJson::Text(msg) = json_object.get("key1").unwrap() {msg} else {"none"});
                assert_eq!(33.2, if let TypeJson::Number(num) = json_object.get("key2").unwrap() {*num} else {0.0});
            },
            Err(_) => {
                assert!(false);
            }
        }

        let input = String::from("{\"key1\":\"input_automa_1\",\"key2\": [12]}");

        match json_autom.start(&mut input.chars()) {
            Ok(json_object) => {
                match json_object.get("key2") {
                    Some(TypeJson::List(list)) => match list.get(0) {
                        Some(TypeJson::Number(num)) => assert_eq!(12.0, *num),
                        _ => assert!(false),
                    }
                    _ => assert!(false),
                }
            },
            Err(_) => {
                assert!(false);
            }
        }
    }

    #[test]
    fn string_automa() {
        let input = String::from("null");
        let string_automa = StringAutoma::from("null");

        let mut iter = input.chars();
        match string_automa.start(&mut iter) {
            Ok(_) => {
                assert!(true);
            },
            _ => assert!(false),
        }

        let mut iter = input.chars().chain(std::iter::once('c'));
        match string_automa.start(&mut iter) {
            Ok(_) => {
                assert!(true);
            },
            _ => assert!(false),
        }

        assert_eq!(Some('c'), iter.next());

    }

    #[test]
    fn number_automa() {
        let number_automa = NumberAutoma::new();
        
        let input = String::from("1234.2123");
        let mut iter = input.chars();
        match number_automa.start(&mut iter) {
            Ok((number, _)) => assert_eq!(1234.2123, number),
            _ => assert!(false),
        }
        
        let input = String::from("001234.002123");
        let mut iter = input.chars();
        match number_automa.start(&mut iter) {
            Ok((number, _)) => assert_eq!(1234.002123, number),
            _ => assert!(false),
        }
        
        let input = String::from("001234.002123,");
        let mut iter = input.chars();
        match number_automa.start(&mut iter) {
            Ok((number, Some(c))) => {
                assert_eq!(1234.002123, number);
                assert_eq!(',', c);
            },
            _ => assert!(false),
        }
        
        let input = String::from("001234");
        let mut iter = input.chars();
        match number_automa.start(&mut iter) {
            Ok((number, _)) => assert_eq!(1234f32, number),
            _ => assert!(false),
        }
    }

    #[test]
    fn array_automa() {
        let array_automa = ArrayAutoma::new();

        let input = String::from("[\"Hello, World\", null, 2234.23, {\"key\": \"Value!\"}]");
        let mut iter = input.chars();
        let array = array_automa.start(&mut iter).unwrap();
        match array.get(0) {
            Some(TypeJson::Text(txt)) => assert_eq!("Hello, World", txt),
            _ => assert!(false),
        }
        match array.get(1) {
            Some(TypeJson::Null) => assert!(true),
            _ => assert!(false),
        }
        match array.get(2) {
            Some(TypeJson::Number(num)) => assert_eq!(2234.23, *num),
            _ => assert!(false),
        }
        match array.get(3) {
            Some(TypeJson::Object(obj)) => match obj.get("key").unwrap().as_text() {
                Some(msg) => assert_eq!("Value!", msg),
                _ => assert!(false),
            },
            _ => assert!(false),
        }

        let input = String::from("[]");
        let mut iter = input.chars();
        let array = array_automa.start(&mut iter).unwrap();
        assert_eq!(0, array.len());
    }

    #[test]
    fn complete_json() {
        let input = String::from(r###"{
            "name": "Foo",
            "username": "Paa",
            "age": 32.0,
            "valid": true,
            "notValid": false,
            "tags": ["t1", "t2"]
        }"###);
        let mut user = json::parser(input.chars()).unwrap();
        assert_eq!("Foo", user.get("name").unwrap().as_text().unwrap());
        assert_eq!("Paa", user.get("username").unwrap().as_text().unwrap());
        assert_eq!(32.0, *user.get("age").unwrap().as_number().unwrap());
        assert_eq!(true, *user.get("valid").unwrap().as_bool().unwrap());
        assert_eq!(false, *user.get("notValid").unwrap().as_bool().unwrap());
        let tags = user.get("tags").unwrap().as_list().unwrap();
        assert_eq!("t1", tags.get(0).unwrap().as_text().unwrap());
        assert_eq!("t2", tags.get(1).unwrap().as_text().unwrap());

        match user.remove("tags") {
            Some(TypeJson::List(list)) => assert_eq!(Some("t2"), list.get(1).unwrap().as_text()),
            _ => assert!(false),
        }
    }

    #[test]
    fn parser_query() {
        let query = ".key.field[1][2].name.field1.000[001].\"txt_!!£\"[33]";
        let mut iter = query.chars();
        let mut parser = KeyParseQueryAutoma::new(&mut iter);
        assert_key("key", &mut parser);
        assert_key("field", &mut parser);
        assert_index(1, &mut parser);
        assert_index(2, &mut parser);
        assert_key("name", &mut parser);
        assert_key("field1", &mut parser);
        assert_key("000", &mut parser);
        assert_index(1, &mut parser);
        assert_key("txt_!!£", &mut parser);
        assert_index(33, &mut parser);

        fn assert_key<T: Iterator<Item=char>>(expected: &str, parser: &mut KeyParseQueryAutoma<T>) {
            match parser.next() {
                Some(KeyParseQueryToken::Key(key)) => assert_eq!(expected, key),
                _ => assert!(false),
            }
        }

        fn assert_index<T: Iterator<Item=char>>(expected: usize, parser: &mut KeyParseQueryAutoma<T>) {
            match parser.next() {
                Some(KeyParseQueryToken::Index(index)) => assert_eq!(expected, index),
                _ => assert!(false),
            }
        }
    }
}