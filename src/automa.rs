use crate::objects as json;
use std::collections::LinkedList;

use ::automa as atm;
use atm::Linkable;
use atm::LinkProcess;

struct StoreBufferIterator<T> {
    size: usize,
    store: std::collections::LinkedList<char>,
    iterator: T,
}

impl <T: Iterator<Item=char>> StoreBufferIterator<T> {
    pub fn new(size: usize, iter: T) -> Self {
        StoreBufferIterator {
            size,
            store: std::collections::LinkedList::new(),
            iterator: iter,
        }
    }

    pub fn into_iter(self) -> impl Iterator<Item = char> {
        self.store.into_iter()
    } 
    
    fn store_c(&mut self, c: char) {
        self.store.push_back(c);
        if self.store.len() >= self.size {
            self.store.pop_front();
        }
    }
}

impl <T: Iterator<Item=char>> Iterator for StoreBufferIterator<T> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iterator.next() {
            Some(c) => {
                self.store_c(c);
                Some(c)
            },
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct AutomaError {
    message: String,
    source: DetailError,
}

impl AutomaError {
    pub fn new(message: String, source: DetailError) -> AutomaError {
        AutomaError {
            message,
            source,
        }
    }
}

impl std::fmt::Display for AutomaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AutomaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Debug)]
pub enum DetailError {
    Parser(ParserError),
}

impl std::fmt::Display for DetailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            DetailError::Parser(error) => write!(f, "{}", error.message())
        }
    }
}

impl std::error::Error for DetailError {
}

impl From<DetailError> for AutomaError {
    fn from(detail: DetailError) -> Self {
        AutomaError::new("Automa error".to_string(), detail)
    }
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

impl From<ParserError> for DetailError {
    fn from(value: ParserError) -> Self {
        DetailError::Parser(value)
    }
}

impl From<ParserError> for AutomaError {
    fn from(value: ParserError) -> Self {
        Into::<DetailError>::into(value).into()
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

        enum StrAtm {
            EndStr,
        }
        
        type StrNode = atm::ANode<(), char, Option<Result<StrAtm, &'static str>>, LinkedList::<char>>;
        
        let mut n1: StrNode = atm::node();
        let mut n2: StrNode = atm::node();
        let mut n3: StrNode = atm::node();
        let mut n4: StrNode = atm::node();

        let mut fail: StrNode = atm::node();

        n1.link(Some(&n2), atm::eq('"'));
        n2.link(Some(&n3), atm::eq('\\'));
        n2.link_function(Some(&n4), atm::eq('"'), |_,_| Some(Ok(StrAtm::EndStr)));
        n2.link_process(None, |c,_| c != &'\\', |c, key| key.push_back(c));
        n3.link_function(Some(&n2), |_,_| true, |c,key| {
            match c {
                '\\' => key.push_back('\\'),
                'n' => key.push_back('\n'),
                'r' => key.push_back('\r'),
                't' => key.push_back('\t'),
                '"' => key.push_back('"'),
                _ => return Some(Err("Invalid escape")),
            }
            None
        });

        n1.link_function(Some(&fail), |_,_| true, |_,_| Some(Err("Invalid char in node n1")));
        n2.link_function(Some(&fail), |_,_| true, |_,_| Some(Err("Invalid char in node n2")));
        n3.link_function(Some(&fail), |_,_| true, |_,_| Some(Err("Invalid char in node n3")));
        n4.link_function(Some(&fail), |_,_| true, |_,_| Some(Err("Invalid char in node n4")));
        fail.link_function(None, |_,_| true, |_,_| Some(Err("Invalid char in node fail")));

        let mut cursor = atm::Cursor::new_none(LinkedList::new(), &n1);
        
        while let Some(c) = iter.next() {
            match cursor.action(c) {
                Some(Ok(StrAtm::EndStr)) => return Ok(cursor.into_context().iter().collect()),
                Some(Err(msg)) => return Err(ParserError::new(msg.to_string()).into()),
                _ => {},
            }
        }
        
        Err(ParserError::new("End Str iterator".to_string()).into())
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

        enum NumAtm {
            End,
        }
        
        struct NumContext {
            end: bool,
            positive: bool,
            extra: Option<char>,
            num: LinkedList::<char>,
        }
        
        type Node = atm::ANode<bool, char, Option<Result<NumAtm, &'static str>>, NumContext>;

        let mut n0: Node = atm::node();
        let mut n1: Node = atm::node();
        let mut n2: Node = true.into();
        let mut n3: Node = true.into();
        let mut n4: Node = atm::node();
        let mut n5: Node = true.into();
        let mut n6: Node = atm::node();
        let mut n7: Node = atm::node();
        let mut n8: Node = true.into();

        n0.link_process(Some(&n1), atm::eq('-'), |_, ctx| ctx.positive = false);
        n0.link_process(Some(&n2), atm::eq('0'), |c, ctx| ctx.num.push_back(c));
        n0.link_process(Some(&n3), |c, _| is_number(*c), |c, ctx| ctx.num.push_back(c));

        n1.link_process(Some(&n2), atm::eq('0'), |c, ctx| ctx.num.push_back(c));
        n1.link_process(Some(&n3), |c, _| is_number(*c), |c, ctx| ctx.num.push_back(c));

        n2.link_process(Some(&n4), atm::eq('.'), |c, ctx| ctx.num.push_back(c));

        n3.link_process(None, |c, _| is_number(*c), |c, ctx| ctx.num.push_back(c));
        n3.link_process(Some(&n4), atm::eq('.'), |c, ctx| ctx.num.push_back(c));
        
        n4.link_process(Some(&n5), |c, _| is_number(*c), |c, ctx| ctx.num.push_back(c));

        n5.link_process(None, |c, _| is_number(*c), |c, ctx| ctx.num.push_back(c));

        // TODO
        
        let mut err = atm::node();
        n2.link_function(Some(&err), |_,_| true, |c, ctx| {ctx.extra = Some(c); Some(Ok(NumAtm::End))});
        n3.link_function(Some(&err), |_,_| true, |c, ctx| {ctx.extra = Some(c); Some(Ok(NumAtm::End))});
        n5.link_function(Some(&err), |_,_| true, |c, ctx| {ctx.extra = Some(c); Some(Ok(NumAtm::End))});
        n8.link_function(Some(&err), |_,_| true, |c, ctx| {ctx.extra = Some(c); Some(Ok(NumAtm::End))});

        let mut cursor = atm::Cursor::black(NumContext {
            end: false,
            positive: true,
            num: LinkedList::new(),
            extra: None,
        }, &n0, |_| Some(Err("Invalid input, default")));

        while let Some(c) = iter.next() {
            match cursor.action(c) {
                Some(Ok(NumAtm::End)) => {
                    let ctx = cursor.into_context();
                    return Ok((retrieve_num(&ctx)?, ctx.extra))
                },
                Some(Err(msg)) => return Err(ParserError::new(msg.to_string()).into()),
                _ => {},
            }
        }

        cursor.access_data(|data, ctx| ctx.end = *data.unwrap_or(&false));

        let ctx = cursor.into_context();
        if ctx.end {
            return Ok((retrieve_num(&ctx)?, None));
        } else {
            return Err(ParserError::new("Invalid number...".to_string()).into());
        }

        fn retrieve_num(ctx: &NumContext) -> Result<f32, ParserError> {
            let sign = if ctx.positive { 1f32 } else { -1f32 };
            Ok(ctx.num
                .iter()
                .collect::<String>()
                .parse()
                .map_err(|err: std::num::ParseFloatError| ParserError::new(err.to_string()))?).map(|el: f32| el * sign)
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
        let mut key_pipe = std::collections::LinkedList::new();
        let mut object_pipe = std::collections::LinkedList::new();
        let str_automa = StrAutoma::new();
        let array_automa = ArrayAutoma::new();
        let number_automa = NumberAutoma::new();
        let null_automa = StringAutoma::from("null");
        let true_automa = StringAutoma::from("true");
        let false_automa = StringAutoma::from("false");
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
                    match c {
                        c if is_space(c) => continue,
                        '{' => {
                            object_pipe.push_front(json_object);
                            key_pipe.push_front(key.take().unwrap());
                            json_object = json::object();
                            status = ObjectAtm::N2;
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
                        '}' => {
                            match object_pipe.pop_front() {
                                Some(mut obj) => {
                                    obj.set(&key_pipe.pop_front().unwrap(), json_object);
                                    json_object = obj;
                                    status = ObjectAtm::N5;
                                },
                                None => return Ok(json_object),
                            }
                        }
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
        let string_automa = StrAutoma::new();
        let number_automa = NumberAutoma::new();
        let object_automa = ObjectAutoma::new();
        let array_automa = ArrayAutoma::new();
        let null_automa = StringAutoma::from("null");
        let false_automa = StringAutoma::from("false");
        let true_automa = StringAutoma::from("true");
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
                            Err(error) => match error.source {
                                DetailError::Parser(msg) => return Some(KeyParseQueryToken::Error(format!("Error reading {}", msg))),
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

pub fn parser(iter: impl Iterator<Item=char>) -> AutomaResult<json::ObjectJson> {
    let mut buffer = StoreBufferIterator::new(10, iter);
    match ObjectAutoma::new().start(&mut buffer) {
        Ok(obj) => Ok(obj),
        Err(err) => {
            let message = format!("Stream read: {}", buffer.into_iter().collect::<String>());
            Err(AutomaError::new(message, err.source))
        }
    }
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
        match str_automa.start(&mut input.chars()) {
            Ok(msg) => assert_eq!("\n\"", msg),
            _ => assert!(false),
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
                assert_eq!(33.2, if let TypeJson::Number(num) = json_object.get("key2").unwrap() {num.into()} else {0.0});
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
                        Some(TypeJson::Number(num)) => assert_eq!(12.0_f32, num.into()),
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
        use std::error::Error;
        let number_automa = NumberAutoma::new();
        
        let input = String::from("1234.2123");
        let mut iter = input.chars();
        match number_automa.start(&mut iter) {
            Ok((number, _)) => assert_eq!(1234.2123, number),
            _ => assert!(false),
        }
        
        let input = String::from("1234..");
        let mut iter = input.chars();
        match number_automa.start(&mut iter) {
            Err(msg) => assert_eq!("Invalid input, default", msg.cause().unwrap().to_string()),
            _ => assert!(false),
        }
        
        let input = String::from("0.2123");
        let mut iter = input.chars();
        match number_automa.start(&mut iter) {
            Ok((number, _)) => assert_eq!(0.2123, number),
            _ => assert!(false),
        }
        
        let input = String::from("0");
        let mut iter = input.chars();
        match number_automa.start(&mut iter) {
            Ok((number, _)) => assert_eq!(0.0, number),
            _ => assert!(false),
        }
        
        let input = String::from("0.0");
        let mut iter = input.chars();
        match number_automa.start(&mut iter) {
            Ok((number, _)) => assert_eq!(0.0, number),
            _ => assert!(false),
        }
        
        let input = String::from("1234.002123,");
        let mut iter = input.chars();
        match number_automa.start(&mut iter) {
            Ok((number, Some(c))) => {
                assert_eq!(1234.002123, number);
                assert_eq!(',', c);
            },
            _ => assert!(false),
        }
        
        let input = String::from("1234");
        let mut iter = input.chars();
        match number_automa.start(&mut iter) {
            Ok((number, _)) => assert_eq!(1234f32, number),
            _ => assert!(false),
        }
        
        let input = String::from("-1234");
        let mut iter = input.chars();
        match number_automa.start(&mut iter) {
            Ok((number, _)) => assert_eq!(-1234f32, number),
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
            Some(TypeJson::Number(num)) => assert_eq!(2234.23_f32, num.into()),
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
            "tags": ["t1", "t2"],
            "sub": {
                "sub2": {
                    "subk": "subv"
                }
            }
        }"###);
        let mut user = json::parser(input.chars()).unwrap();
        assert_eq!("Foo", user.get("name").unwrap().as_text().unwrap());
        assert_eq!("Paa", user.get("username").unwrap().as_text().unwrap());
        assert_eq!(32.0_f32, user.get("age").unwrap().as_number().unwrap().into());
        assert_eq!(true, *user.get("valid").unwrap().as_bool().unwrap());
        assert_eq!(false, *user.get("notValid").unwrap().as_bool().unwrap());
        let tags = user.get("tags").unwrap().as_list().unwrap();
        assert_eq!("t1", tags.get(0).unwrap().as_text().unwrap());
        assert_eq!("t2", tags.get(1).unwrap().as_text().unwrap());

        match user.remove("tags") {
            Some(TypeJson::List(list)) => assert_eq!(Some("t2"), list.get(1).unwrap().as_text()),
            _ => assert!(false),
        }

        assert_eq!("subv", TypeJson::from(user).traverse(".sub.sub2.subk").unwrap().as_text().unwrap());

        let input = r##"{"key": error
        "##;
        let error = json::parser(input.chars()).err().unwrap();
        assert_eq!("Stream read: {\"key\": e", error.to_string())
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

    #[test]
    fn buffer_store() {
        let mut input = StoreBufferIterator::new(10, "text".chars());
        
        assert_eq!('t', input.next().unwrap());
        assert_eq!('e', input.next().unwrap());

        assert_eq!("te", input.into_iter().collect::<String>());
    }
}