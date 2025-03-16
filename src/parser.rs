//! TOML parser utils.

use {
    serde::{
        de::{SeqAccess, Visitor},
        Deserialize, Deserializer,
    },
    std::{collections::HashMap, fmt, marker::PhantomData},
};

pub trait TomlNamedElement {
    fn name(&self) -> &String;
}

pub fn map_of_keyed_elements<'de, D, T>(deserializer: D) -> Result<HashMap<String, T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + fmt::Debug + TomlNamedElement,
{
    struct HashMapVisitor<T> {
        marker: PhantomData<T>,
    }

    impl<'de, T> Visitor<'de> for HashMapVisitor<T>
    where
        T: Deserialize<'de> + fmt::Debug + TomlNamedElement,
    {
        type Value = HashMap<String, T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of objects with a 'name' field")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut map = HashMap::with_capacity(seq.size_hint().unwrap_or(0));

            while let Some(value) = seq.next_element::<T>()? {
                map.insert(value.name().clone(), value);
            }
            Ok(map)
        }
    }

    deserializer.deserialize_seq(HashMapVisitor {
        marker: PhantomData,
    })
}
