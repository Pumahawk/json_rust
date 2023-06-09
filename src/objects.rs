use std::collections::HashMap;

const NULL_JSON: TypeJson = TypeJson::Null;

pub enum TypeJson {
    Object(ObjectJson),
    List(ListJson),
    Text(String),
    Number(Number),
    Boolean(bool),
    Null,
}

impl TypeJson {
    pub fn as_object(&self) -> Option<&ObjectJson> {
        match self {
            TypeJson::Object(obj) => Some(obj),
            _ => None,
        }
    }
    pub fn as_object_mut(&mut self) -> Option<&mut ObjectJson> {
        match self {
            TypeJson::Object(obj) => Some(obj),
            _ => None,
        }
    }
    pub fn as_list(&self) -> Option<&ListJson> {
        match self {
            TypeJson::List(list) => Some(list),
            _ => None,
        }
    }
    pub fn as_list_mut(&mut self) -> Option<&mut ListJson> {
        match self {
            TypeJson::List(list) => Some(list),
            _ => None,
        }
    }
    pub fn as_text(&self) -> Option<&str> {
        match self {
            TypeJson::Text(txt) => Some(txt),
            _ => None,
        }
    }
    pub fn as_text_mut(&mut self) -> Option<&mut str> {
        match self {
            TypeJson::Text(txt) => Some(txt),
            _ => None,
        }
    }
    pub fn as_number(&self) -> Option<&Number> {
        match self {
            TypeJson::Number(num) => Some(num),
            _ => None,
        }
    }
    pub fn as_number_mut(&mut self) -> Option<&mut Number> {
        match self {
            TypeJson::Number(num) => Some(num),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<&bool> {
        match self {
            TypeJson::Boolean(boolean) => Some(boolean),
            _ => None,
        }
    }
    pub fn as_bool_mut(&mut self) -> Option<&mut bool> {
        match self {
            TypeJson::Boolean(boolean) => Some(boolean),
            _ => None,
        }
    }
    pub fn is_null(&self) -> bool {
        match self {
            TypeJson::Null => true,
            _ => false,
        }
    }

    pub fn traverse(self, path: &str) -> Result<TypeJson, String> {
        let mut ret = self.into();
        let mut chars = path.chars();
        let mut automa = crate::automa::KeyParseQueryAutoma::new(&mut chars);
        while let Some(token) = automa.next() {
            match token {
                crate::automa::KeyParseQueryToken::Key(key) => ret = match ret {
                    TypeJson::Object(mut obj) => obj.remove(&key).ok_or("Unable to traverse")?,
                    _ => TypeJson::Null, 
                },
                crate::automa::KeyParseQueryToken::Index(i) => ret = match ret {
                    TypeJson::List(mut list) => list.remove(i),
                    _ => TypeJson::Null, 
                },
                crate::automa::KeyParseQueryToken::Error(msg) => return Err(msg),
            };
        }
        Ok(ret)
    }
}

impl ToString for TypeJson {
    fn to_string(&self) -> String {
        match self {
            TypeJson::Object(object) => object.to_string(),
            TypeJson::List(list) => list.to_string(),
            TypeJson::Text(txt) => string_to_json_escape(txt),
            TypeJson::Number(num) => num.to_string(),
            TypeJson::Boolean(b) => b.to_string(),
            TypeJson::Null => String::from("null"),
        }
    }
}

fn string_to_json_escape(txt: &str) -> String {
    return std::iter::once('"')
        .chain(txt
            .chars()
            .flat_map(|c| {
                let iter: Box<dyn Iterator<Item = char>>;
                match c {
                    '"' => iter = to_iter("\\\""),
                    '\n' => iter = to_iter("\\n"),
                    '\r' => iter = to_iter("\\r"),
                    '\t' => iter = to_iter("\\t"),
                    other => iter = Box::new(std::iter::once(other)),
                }
                iter
            })
        )
        .chain(std::iter::once('"'))
        .collect();
    
    fn to_iter(txt: &'static str) -> Box<dyn Iterator<Item = char>> {
        Box::new(txt.chars())
    }
}

pub struct NumberExponent {
    number: i32,
}

impl NumberExponent {
    pub fn new(value: i32) -> NumberExponent {
        NumberExponent {
            number: value,
        }
    }
}

impl ToString for NumberExponent {
    fn to_string(&self) -> String {
        format!("e{}", self.number.to_string())
    }
}

impl From<NumberExponent> for f32 {
    
    fn from(value: NumberExponent) -> Self {
        (&value).into()
    }
}

impl From<&NumberExponent> for f32 {
    
    fn from(value: &NumberExponent) -> Self {
        10_f32.powi(value.number)
    }
}

pub struct Number {
    number: f32,
    exponent: Option<NumberExponent>,
}

impl Number {
    pub fn new(value: f32, exponent: Option<NumberExponent>) -> Self {
        Number {
            number: value,
            exponent,
        }
    }
}

impl ToString for Number {
    fn to_string(&self) -> String {
        format!("{}{}", self.number.to_string(), self.exponent
            .as_ref()
            .map(|exp| exp.to_string())
            .unwrap_or("".to_string()))
    }
}

impl From<Number> for f32 {
    
    fn from(value: Number) -> Self {
        (&value).into()
    }
}

impl From<&Number> for f32 {
    
    fn from(value: &Number) -> Self {
        value.number as f32 * value.exponent.as_ref().map(|exp| exp.into()).unwrap_or(1.0)
    }
}

impl From<f32> for Number {
    fn from(value: f32) -> Self {
        Number::new(value, None)
    }
}

impl From<Number> for TypeJson {
    fn from(value: Number) -> Self {
        TypeJson::Number(value)
    }
}

pub struct ObjectJson {
    parameters: HashMap<String, TypeJson>,
}

impl ObjectJson {

    fn new() -> ObjectJson {
        ObjectJson {
            parameters: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, obj: impl Into<TypeJson>) {
        self.parameters.insert(String::from(key), obj.into());
    }

    pub fn object(&mut self, key: &str) -> &mut ObjectJson {
        self.set(key, ObjectJson::new());
        match self.get_mut(key) {
            Some(TypeJson::Object(obj)) => obj,
            _ => unreachable!(),
        }
    }

    pub fn list(&mut self, key: &str) -> &mut ListJson {
        self.set(key, ListJson::new());
        match self.get_mut(key) {
            Some(TypeJson::List(list)) => list,
            _ => unreachable!(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&TypeJson> {
        self.parameters
            .get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut TypeJson> {
        self.parameters
            .get_mut(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<TypeJson> {
        self.parameters.remove(key)
    }

    pub fn iter(&self) -> impl Iterator<Item=(&String, &TypeJson)> {
        self.parameters.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=(&String, &mut TypeJson)> {
        self.parameters.iter_mut()
    }

    pub fn keys(&self) -> impl Iterator<Item=&String> {
        self.iter().map(|(key, _)|key)
    }
}

impl ToString for ObjectJson {
    fn to_string(&self) -> String {
        std::iter::once('{')
        .chain(self
            .iter()
            .map(|(key, obj)| format!("\"{}\":{}", key, obj.to_string()))
            .collect::<Vec<_>>()
            .join(",")
            .chars()
        )
        .chain(std::iter::once('}'))
        .collect()
    }
}

impl From<ObjectJson> for TypeJson {
    fn from(object: ObjectJson) -> TypeJson {
        TypeJson::Object(object)
    }
}

pub struct ListJson {
    list: Vec<TypeJson>,
}

impl ListJson {
    pub fn new() -> ListJson {
        ListJson {
            list: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn add(&mut self, obj: impl Into<TypeJson>) {
        self.list.push(obj.into());
    }

    pub fn pop(&mut self) -> Option<TypeJson> {
        self.list.pop()
    }

    pub fn remove(&mut self, index: usize) -> TypeJson {
        self.list.remove(index)
    }

    pub fn object(&mut self) -> &mut ObjectJson {
        self.add(ObjectJson::new());
        match self.list.last_mut() {
            Some(TypeJson::Object(node)) => node,
            _ => unreachable!(),
        }
    }

    pub fn list(&mut self) -> &mut ListJson {
        self.add(ListJson::new());
        match self.list.last_mut() {
            Some(TypeJson::List(node)) => node,
            _ => unreachable!(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&TypeJson> {
        self.list.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut TypeJson> {
        self.list.get_mut(index)
    }

    pub fn iter(&self) -> impl Iterator<Item=&TypeJson> {
        self.list.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut TypeJson> {
        self.list.iter_mut()
    }
}

impl std::iter::IntoIterator for ListJson {
    type Item = TypeJson;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.list.into_iter()
    }
}

impl <'a> std::iter::IntoIterator for &'a ListJson {
    type Item = &'a TypeJson;
    type IntoIter = std::slice::Iter<'a, TypeJson>;

    fn into_iter(self) -> Self::IntoIter {
        self.list[..].iter()
    }
}

impl <'a> std::iter::IntoIterator for &'a mut ListJson {
    type Item = &'a mut TypeJson;
    type IntoIter = std::slice::IterMut<'a, TypeJson>;

    fn into_iter(self) -> Self::IntoIter {
        self.list[..].iter_mut()
    }
}

impl ToString for ListJson {
    fn to_string(&self)  -> String {
        std::iter::once('[')
            .chain(self
                .iter()
                .map(|obj|obj.to_string())
                .collect::<Vec<_>>()
                .join(",")
                .chars()
            )
            .chain(std::iter::once(']'))
            .collect()
    }
}

impl From<ListJson> for TypeJson {
    fn from(list: ListJson) -> TypeJson {
        TypeJson::List(list)
    }
}

impl From<String> for TypeJson {
    fn from(value: String) -> Self {
        TypeJson::Text(value)
    }
}

impl From<&str> for TypeJson {
    fn from(value: &str) -> Self {
        TypeJson::Text(value.to_string())
    }
}

impl From<f32> for TypeJson {
    fn from(value: f32) -> Self {
        Number::from(value).into()
    }
}

impl From<bool> for TypeJson {
    fn from(value: bool) -> Self {
        TypeJson::Boolean(value)
    }
}

pub struct NullJson;

impl NullJson {
    pub fn new() -> NullJson {
        NullJson {}
    }
}

impl From<NullJson> for TypeJson {
    fn from(_: NullJson) -> TypeJson {
        TypeJson::Null
    }
}

pub fn object() -> ObjectJson {
    ObjectJson::new()
}

pub fn array() -> ListJson {
    ListJson::new()
}

pub fn null() -> NullJson {
    NullJson::new()
}

pub struct ReaderJson<'a> {
    root: Option<&'a TypeJson>,
}

impl <'a> ReaderJson<'a> {
    pub fn new(root: &'a TypeJson) -> ReaderJson {
        ReaderJson {
            root: Some(root),
        }
    }

    fn empty() -> ReaderJson<'a> {
        ReaderJson {
            root: None,
        }
    }

    pub fn field(&self, key: &str) -> ReaderJson<'a> {
        match self.root {
            Some(TypeJson::Object(obj)) => match obj.get(key) {
                Some(node) => ReaderJson::new(node),
                _ => ReaderJson::empty(),
            },
            _ => ReaderJson::empty(),
        }
    }

    pub fn index(&self, i: usize) -> ReaderJson<'a> {
        match self.root {
            Some(TypeJson::List(list)) => match list.get(i) {
                Some(node) =>  ReaderJson::new(node),
                _ => ReaderJson::empty(),
            }
            _ => ReaderJson::empty(),
        }
    }

    
    pub fn path(&self, path: &str) -> ReaderJson<'a> {
        match self.path_check(path) {
            Ok(reader) => reader,
            _ => ReaderJson::empty(),
        }
    }
    
    pub fn path_check(&self, path: &str) -> Result<ReaderJson<'a>, String> {
        if let Some(root) = self.root {
            let mut ret = ReaderJson::new(root);
            let mut path = path.chars();
            let mut automa = crate::automa::KeyParseQueryAutoma::new(&mut path);
            while let Some(token) = automa.next() {
                match token {
                    crate::automa::KeyParseQueryToken::Key(key) => ret = ret.field(&key),
                    crate::automa::KeyParseQueryToken::Index(i) => ret = ret.index(i),
                    crate::automa::KeyParseQueryToken::Error(msg) => return Err(msg),
                };
            }
            Ok(ret)
        } else {
            Ok(ReaderJson::empty())
        }
        
    }

    pub fn json(&self) -> &'a TypeJson {
        match self.root {
            Some(node) => node,
            _ => &NULL_JSON,
        }
    }
}



#[cfg(test)]
mod tests {
    use crate::objects::*;
    #[test]
    fn edit_objects() {
        let mut root = object();
        let mut obj1 = object();
        let mut obj_sub_1 = object();
        
        obj1.set("field", "hello World");
        root.set("key1", obj1);
        {
            let obj2 = root.object("key2");
            obj_sub_1.set("field", "hello World 2");
            obj2.set("key-sub-1", obj_sub_1);
        }

        match root.get("key1") {
            Some(TypeJson::Object(obj)) => {
                match obj.get("field") {
                    Some(TypeJson::Text(msg)) => assert_eq!(msg, "hello World"),
                    _ => assert!(false),
                }
            },
            _ => assert!(false),
        }

        match root
            .get("key2")
            .and_then(|obj|obj.as_object())
            .and_then(|obj|obj.get("key-sub-1"))
            .and_then(|obj|obj.as_object())
            .and_then(|obj|obj.get("field")) {
                Some(TypeJson::Text(msg)) => assert_eq!(msg, "hello World 2"),
                _ => assert!(false), 
            }
        
        assert_eq!("hello World 2", root
            .get("key2")
            .and_then(|obj|obj.as_object())
            .and_then(|obj|obj.get("key-sub-1"))
            .and_then(|obj|obj.as_object())
            .and_then(|obj|obj.get("field"))
            .and_then(|obj|obj.as_text())
            .unwrap()
        );

    }
    #[test]
    fn edit_list() {
        let mut root = array();
        let mut obj1 = object();
        obj1.set("key1", "value1");

        root.add(obj1);
        root.add("second");
        assert!(root.get(0).is_some());

        let mut iter = root.iter_mut();
        match iter.next() {
            Some(TypeJson::Object(obj)) => match obj.get("key1") {
                Some(TypeJson::Text(msg)) => assert_eq!(msg, "value1"),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }

        match iter.next() {
            Some(TypeJson::Text(msg)) => assert_eq!(msg, "second"),
            _ => assert!(false),
        }
    }
    #[test]
    fn null_number() {
        let mut root = object();
        root.set("key1", null());

        match root.get("key1") {
            Some(TypeJson::Null) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn reader_json() {
        let mut root = object();
        let node = root.object("k1");
        node.set("n1", "value1");
        node.set("n2", "value2");
        let node = root.object("k2");
        node.set("n3", "value1");
        node.set("n4", "value2");
        let node = node.object("k3");
        node.set("n5", "value-sub1");
        let list = node.list("n6");
        list.add("message-1");
        list.add("message-2");
        let obj = list.object();
        obj.set("k1", "v1");
        let array = list.list();
        array.add("v1");

        let root = root.into();
        let reader = ReaderJson::new(&root);
        assert_eq!(Some("value-sub1"), reader.field("k2").field("k3").field("n5").json().as_text());
        assert_eq!(Some("message-2"), reader.field("k2").field("k3").field("n6").index(1).json().as_text());

        assert_eq!(Some("message-2"), reader.path(".k2.k3.n6[1]").json().as_text());
    }

    #[test]
    fn json_to_string() {
        
        let mut root = object();
        let node = root.object("k1");
        node.set("n1", "value1");

        assert_eq!(
            "{\"k1\":{\"n1\":\"value1\"}}",
            root.to_string());
        
        let mut root = object();
        let node = root.list("k1");
        node.add(12.1);

        assert_eq!(
            "{\"k1\":[12.1]}",
            root.to_string());
    }

    #[test]
    fn bool_json() {
        let mut root = object();
        root.set("k", true);
        match root.get("k").unwrap() {
            TypeJson::Boolean(bl) => assert!(bl),
            _ => assert!(false),
        }
    }

    #[test]
    fn traverse_test() {
        let mut root = object();
        let node = root.object("k1");
        node.set("n1", "value1");
        node.set("n2", "value2");
        let node = root.object("k2");
        node.set("n3", "value1");
        node.set("n4", "value2");
        let node = node.object("k3");
        node.set("n5", "value-sub1");
        let list = node.list("n6");
        list.add("message-1");
        list.add("message-2");
        let obj = list.object();
        obj.set("k1", "v1");
        let array = list.list();
        array.add("v1");

        assert_eq!("message-2", TypeJson::from(root).traverse(".k2.k3.n6[1]").unwrap().as_text().unwrap())
    }

    #[test]
    fn number_from() {
        let num = Number::new(10.0, None);
        assert_eq!(10_f32, num.into());

        let num = Number::new(10.1, None);
        assert_eq!(10.1_f32, num.into());

        let exp = NumberExponent::new(2);
        assert_eq!(100_f32, exp.into());

        let exp = NumberExponent::new(-2);
        assert_eq!(0.01_f32, exp.into());

        let num = Number::new(11.234, Some(NumberExponent::new(2)));
        assert_eq!(1123.4_f32, num.into());

        let num = Number::new(11.234, Some(NumberExponent::new(-2)));
        assert_eq!(0.11234_f32, num.into());
    }
}