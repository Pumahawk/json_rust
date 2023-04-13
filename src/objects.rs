use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

pub enum TypeJson {
    Object(ObjectJson),
    List(ListJson),
    Text(String),
    Number(f32),
    Null,
}

impl TypeJson {
    pub fn as_object(&self) -> Option<&ObjectJson> {
        match self {
            TypeJson::Object(obj) => Some(obj),
            _ => None,
        }
    }
    pub fn as_object_mut(&mut self) -> Option<&ObjectJson> {
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
    pub fn as_number(&self) -> Option<&f32> {
        match self {
            TypeJson::Number(num) => Some(num),
            _ => None,
        }
    }
    pub fn as_number_mut(&mut self) -> Option<&mut f32> {
        match self {
            TypeJson::Number(num) => Some(num),
            _ => None,
        }
    }
    pub fn is_null(&self) -> bool {
        match self {
            TypeJson::Null => true,
            _ => false,
        }
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

    pub fn create(&mut self, key: &str) -> &mut ObjectJson {
        self.set(key, ObjectJson::new());
        match self.get(key) {
            Some(TypeJson::Object(obj)) => obj,
            _ => unreachable!(),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&mut TypeJson> {
        self.parameters
            .get_mut(key)
    }
    
    pub fn as_object(&mut self, key: &str) -> Option<&mut ObjectJson> {
        match self.get(key) {
            Some(TypeJson::Object(obj)) => Some(obj),
            _ => None,
        }
    }

    pub fn as_text(&mut self, key: &str) -> Option<&mut str> {
        match self.get(key) {
            Some(TypeJson::Text(msg)) => Some(msg),
            _ => None,
        }
    }

    pub fn as_list(&mut self, key: &str) -> Option<&mut ListJson> {
        match self.get(key) {
            Some(TypeJson::List(list)) => Some(list),
            _ => None,
        }
    }

    pub fn as_number(&mut self, key: &str) -> Option<&mut f32> {
        match self.get(key) {
            Some(TypeJson::Number(number)) => Some(number),
            _ => None,
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=(&String, &mut TypeJson)> {
        self.parameters.iter_mut()
    }

    pub fn values(&mut self) -> impl Iterator<Item=&mut TypeJson> {
        self.iter_mut().map(|(_, value)| value)
    }

    pub fn keys(&mut self) -> impl Iterator<Item=&String> {
        self.iter_mut().map(|(key, _)|key)
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

    pub fn get(&mut self, index: usize) -> Option<&mut TypeJson> {
        self.list.get_mut(index)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut TypeJson> {
        self.list.iter_mut()
    }
}

impl From<ListJson> for TypeJson {
    fn from(list: ListJson) -> TypeJson {
        TypeJson::List(list)
    }
}

pub struct NumberJson {
    value: f32,
}

impl NumberJson {
    pub fn new(value: f32) -> NumberJson {
        NumberJson {
            value,
        }
    }
}

impl From<f32> for NumberJson {
    fn from(value: f32) -> Self {
        NumberJson::new(value)
    }
}

impl From<NumberJson> for TypeJson {
    fn from(value: NumberJson) -> TypeJson {
        TypeJson::Number(*value)
    }
}

impl Deref for NumberJson {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for NumberJson {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub struct TextJson {
    value: String,
}

impl TextJson {
    pub fn new(value: String) -> TextJson {
        TextJson {
            value,
        }
    }
}

impl <T: ToString> From<T> for TextJson {
    fn from(value: T) -> Self {
        Self::new(value.to_string())
    }
}

impl From<TextJson> for TypeJson {
    fn from(value: TextJson) -> TypeJson {
        TypeJson::Text(value.value)
    }
}

impl Deref for TextJson {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for TextJson {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
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

pub fn text<T: ToString>(txt: T) -> TextJson {
    TextJson::from(txt)
}

pub fn number(num: f32) -> NumberJson {
    NumberJson::from(num)
}

pub fn null() -> NullJson {
    NullJson::new()
}

#[cfg(test)]
mod tests {
    use crate::objects::*;
    #[test]
    fn edit_objects() {
        let mut root = object();
        let mut obj1 = object();
        let mut obj_sub_1 = object();
        
        obj1.set("field", text("hello World"));
        root.set("key1", obj1);
        {
            let obj2 = root.create("key2");
            obj_sub_1.set("field", text("hello World 2"));
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
            .as_object("key2")
            .and_then(|obj|obj.as_object("key-sub-1"))
            .and_then(|obj|obj.get("field")) {
                Some(TypeJson::Text(msg)) => assert_eq!(msg, "hello World 2"),
                _ => assert!(false), 
            }
        
        assert_eq!("hello World 2", root
            .as_object("key2")
            .and_then(|obj|obj.as_object("key-sub-1"))
            .and_then(|obj|obj.as_text("field"))
            .unwrap()
        );

    }
    #[test]
    fn edit_list() {
        let mut root = array();
        let mut obj1 = object();
        obj1.set("key1", text("value1"));

        root.add(obj1);
        root.add(text("second"));
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
    fn edit_text() {
        let mut text = text("message");
        assert_eq!(&*text, "message");
        *text = String::from("message edit");
        assert_eq!(&*text, "message edit");
    }
    #[test]
    fn edit_number() {
        let mut number = number(5.3);
        assert_eq!(*number, 5.3);
        *number = 6.2;
        assert_eq!(*number, 6.2);

        match TypeJson::from(number) {
            TypeJson::Number(num) => assert_eq!(num, 6.2),
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
}