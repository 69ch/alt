use std::{ collections::HashMap, fmt::{Debug, Write}, hash::Hash, mem, rc::Rc, vec::IntoIter };

/// HashMap that follows insertion order for values. \
/// Note that replacement of values in keys through 'insert' does not update order of that value in value vec.
#[derive(Clone)]
pub struct InsordMap<K: Clone, V: Clone> {
    values: Vec<(Rc<K>, V)>,
    map: HashMap<Rc<K>, usize>
}

pub struct Values<'a, K, V> {
    data: &'a [(Rc<K>, V)],
    current: usize
}

pub struct IntoValues<K, V> {
    data: IntoIter<(Rc<K>, V)>
}

pub struct Iter<'a, K, V> {
    data: &'a [(Rc<K>, V)],
    current: usize
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;
    fn next (&mut self) -> Option<Self::Item> {
        let (_, v) = self.data.get(self.current)?;
        self.current += 1;
        Some(v)
    }
}

impl<K, V: Default> Iterator for IntoValues<K, V> {
    type Item = V;
    fn next (&mut self) -> Option<Self::Item> {
        let (_, v) = self.data.next()?;
        Some(v)
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = &'a (Rc<K>, V);
    fn next (&mut self) -> Option<Self::Item> {
        let kv = self.data.get(self.current)?;
        self.current += 1;
        Some(kv)
    }
}

impl<K: Hash + Eq + Clone, V: Clone> InsordMap<K, V> {
    pub fn new () -> Self {
        Self {
            values: vec![],
            map: HashMap::new()
        }
    }

    /// Maps value for access by key. \
    /// If something already was mapped to key, returns previous value.
    pub fn insert (&mut self, key: K, value: V) -> Option<V> {
        let key = Rc::new(key);
        if let Some(p) = self.map.get(&key) {
            let rv = mem::replace(&mut self.values[*p], (key.clone(), value));
            self.map.insert(key, *p);
            Some(rv.1)
        }
        else {
            self.values.push((key.clone(), value));
            self.map.insert(key, self.values.len() - 1);
            None
        }
    }

    pub fn get (&self, key: &K) -> Option<&V> where {
        let p = self.map.get(key)?;
        Some(&self.values[*p].1)
    }
    pub fn get_w_p (&self, key: &K) -> Option<(&V, usize)> where {
        let p = self.map.get(key)?;
        Some((&self.values[*p].1, *p))
    }

    pub fn kv (&self) -> Values<K, V> {
        Values {
            data: &self.values,
            current: 0
        }
    }
    pub fn values (&self) -> Values<K, V> {
        Values {
            data: &self.values,
            current: 0
        }
    }
    pub fn into_values (self) -> IntoValues<K, V> {
        IntoValues {
            data: self.values.into_iter()
        }
    }
    pub fn iter (&self) -> Iter<K, V> {
        Iter {
            data: &self.values,
            current: 0
        }
    }
    pub fn clone_kv (&self) -> Vec<(Rc<K>, V)> {
        self.values.clone()
    }
}

impl<K: Debug + Clone, V: Debug + Clone> Debug for InsordMap<K, V> {
    fn fmt (&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('{')?;
        if self.values.len() == 0 { return f.write_char('}') }
        if f.alternate() { f.write_char('\n')?; }
        let mut i = 0;
        while i <= self.values.len() - 1 {
            let (k, v) = &self.values[i];
            if f.alternate() {
                let vv = format!("{v:#?}").replace("\n", "\n    ");
                writeln!(f, "    {k:#?}: {vv},")?;
            }
            else { write!(f, "{k:?}: {v:?}{}", if self.values.len() > 1 && i < self.values.len() - 2 { ", " } else { "" })?; }
            i += 1;
        }
        f.write_char('}')
    }
}