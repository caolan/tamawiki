(function() {var implementors = {};
implementors["arrayvec"] = [{text:"impl&lt;A:&nbsp;<a class=\"trait\" href=\"arrayvec/trait.Array.html\" title=\"trait arrayvec::Array\">Array</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"arrayvec/struct.IntoIter.html\" title=\"struct arrayvec::IntoIter\">IntoIter</a>&lt;A&gt;",synthetic:false,types:["arrayvec::IntoIter"]},{text:"impl&lt;'a, A:&nbsp;<a class=\"trait\" href=\"arrayvec/trait.Array.html\" title=\"trait arrayvec::Array\">Array</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"arrayvec/struct.Drain.html\" title=\"struct arrayvec::Drain\">Drain</a>&lt;'a, A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A::<a class=\"type\" href=\"arrayvec/trait.Array.html#associatedtype.Item\" title=\"type arrayvec::Array::Item\">Item</a>: 'a,&nbsp;</span>",synthetic:false,types:["arrayvec::Drain"]},];
implementors["generic_array"] = [{text:"impl&lt;T, N&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"generic_array/iter/struct.GenericArrayIter.html\" title=\"struct generic_array::iter::GenericArrayIter\">GenericArrayIter</a>&lt;T, N&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;N: <a class=\"trait\" href=\"generic_array/trait.ArrayLength.html\" title=\"trait generic_array::ArrayLength\">ArrayLength</a>&lt;T&gt;,&nbsp;</span>",synthetic:false,types:["generic_array::iter::GenericArrayIter"]},];
implementors["http"] = [{text:"impl&lt;'a, T:&nbsp;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"http/header/struct.ValueIter.html\" title=\"struct http::header::ValueIter\">ValueIter</a>&lt;'a, T&gt;",synthetic:false,types:["http::header::map::ValueIter"]},{text:"impl&lt;'a, T:&nbsp;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"http/header/struct.ValueIterMut.html\" title=\"struct http::header::ValueIterMut\">ValueIterMut</a>&lt;'a, T&gt;",synthetic:false,types:["http::header::map::ValueIterMut"]},];
implementors["indexmap"] = [{text:"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"indexmap/set/struct.IntoIter.html\" title=\"struct indexmap::set::IntoIter\">IntoIter</a>&lt;T&gt;",synthetic:false,types:["indexmap::set::IntoIter"]},{text:"impl&lt;'a, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"indexmap/set/struct.Iter.html\" title=\"struct indexmap::set::Iter\">Iter</a>&lt;'a, T&gt;",synthetic:false,types:["indexmap::set::Iter"]},{text:"impl&lt;'a, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"indexmap/set/struct.Drain.html\" title=\"struct indexmap::set::Drain\">Drain</a>&lt;'a, T&gt;",synthetic:false,types:["indexmap::set::Drain"]},{text:"impl&lt;'a, T, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"indexmap/set/struct.Difference.html\" title=\"struct indexmap::set::Difference\">Difference</a>&lt;'a, T, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a>,&nbsp;</span>",synthetic:false,types:["indexmap::set::Difference"]},{text:"impl&lt;'a, T, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"indexmap/set/struct.Intersection.html\" title=\"struct indexmap::set::Intersection\">Intersection</a>&lt;'a, T, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a>,&nbsp;</span>",synthetic:false,types:["indexmap::set::Intersection"]},{text:"impl&lt;'a, T, S1, S2&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"indexmap/set/struct.SymmetricDifference.html\" title=\"struct indexmap::set::SymmetricDifference\">SymmetricDifference</a>&lt;'a, T, S1, S2&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;S1: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;S2: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a>,&nbsp;</span>",synthetic:false,types:["indexmap::set::SymmetricDifference"]},{text:"impl&lt;'a, T, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"indexmap/set/struct.Union.html\" title=\"struct indexmap::set::Union\">Union</a>&lt;'a, T, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a>,&nbsp;</span>",synthetic:false,types:["indexmap::set::Union"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"indexmap/map/struct.Keys.html\" title=\"struct indexmap::map::Keys\">Keys</a>&lt;'a, K, V&gt;",synthetic:false,types:["indexmap::map::Keys"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"indexmap/map/struct.Values.html\" title=\"struct indexmap::map::Values\">Values</a>&lt;'a, K, V&gt;",synthetic:false,types:["indexmap::map::Values"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"indexmap/map/struct.ValuesMut.html\" title=\"struct indexmap::map::ValuesMut\">ValuesMut</a>&lt;'a, K, V&gt;",synthetic:false,types:["indexmap::map::ValuesMut"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"indexmap/map/struct.Iter.html\" title=\"struct indexmap::map::Iter\">Iter</a>&lt;'a, K, V&gt;",synthetic:false,types:["indexmap::map::Iter"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"indexmap/map/struct.IterMut.html\" title=\"struct indexmap::map::IterMut\">IterMut</a>&lt;'a, K, V&gt;",synthetic:false,types:["indexmap::map::IterMut"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"indexmap/map/struct.IntoIter.html\" title=\"struct indexmap::map::IntoIter\">IntoIter</a>&lt;K, V&gt;",synthetic:false,types:["indexmap::map::IntoIter"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"indexmap/map/struct.Drain.html\" title=\"struct indexmap::map::Drain\">Drain</a>&lt;'a, K, V&gt;",synthetic:false,types:["indexmap::map::Drain"]},];
implementors["linked_hash_map"] = [{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"linked_hash_map/struct.Iter.html\" title=\"struct linked_hash_map::Iter\">Iter</a>&lt;'a, K, V&gt;",synthetic:false,types:["linked_hash_map::Iter"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"linked_hash_map/struct.IterMut.html\" title=\"struct linked_hash_map::IterMut\">IterMut</a>&lt;'a, K, V&gt;",synthetic:false,types:["linked_hash_map::IterMut"]},{text:"impl&lt;K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"linked_hash_map/struct.IntoIter.html\" title=\"struct linked_hash_map::IntoIter\">IntoIter</a>&lt;K, V&gt;",synthetic:false,types:["linked_hash_map::IntoIter"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"linked_hash_map/struct.Keys.html\" title=\"struct linked_hash_map::Keys\">Keys</a>&lt;'a, K, V&gt;",synthetic:false,types:["linked_hash_map::Keys"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"linked_hash_map/struct.Values.html\" title=\"struct linked_hash_map::Values\">Values</a>&lt;'a, K, V&gt;",synthetic:false,types:["linked_hash_map::Values"]},];
implementors["lru_cache"] = [{text:"impl&lt;K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"lru_cache/struct.IntoIter.html\" title=\"struct lru_cache::IntoIter\">IntoIter</a>&lt;K, V&gt;",synthetic:false,types:["lru_cache::IntoIter"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"lru_cache/struct.Iter.html\" title=\"struct lru_cache::Iter\">Iter</a>&lt;'a, K, V&gt;",synthetic:false,types:["lru_cache::Iter"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"lru_cache/struct.IterMut.html\" title=\"struct lru_cache::IterMut\">IterMut</a>&lt;'a, K, V&gt;",synthetic:false,types:["lru_cache::IterMut"]},];
implementors["memchr"] = [{text:"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"memchr/struct.Memchr.html\" title=\"struct memchr::Memchr\">Memchr</a>&lt;'a&gt;",synthetic:false,types:["memchr::Memchr"]},];
implementors["phf"] = [{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"phf/map/struct.Entries.html\" title=\"struct phf::map::Entries\">Entries</a>&lt;'a, K, V&gt;",synthetic:false,types:["phf::map::Entries"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"phf/map/struct.Keys.html\" title=\"struct phf::map::Keys\">Keys</a>&lt;'a, K, V&gt;",synthetic:false,types:["phf::map::Keys"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"phf/map/struct.Values.html\" title=\"struct phf::map::Values\">Values</a>&lt;'a, K, V&gt;",synthetic:false,types:["phf::map::Values"]},{text:"impl&lt;'a, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"phf/set/struct.Iter.html\" title=\"struct phf::set::Iter\">Iter</a>&lt;'a, T&gt;",synthetic:false,types:["phf::set::Iter"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"phf/ordered_map/struct.Entries.html\" title=\"struct phf::ordered_map::Entries\">Entries</a>&lt;'a, K, V&gt;",synthetic:false,types:["phf::ordered_map::Entries"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"phf/ordered_map/struct.Keys.html\" title=\"struct phf::ordered_map::Keys\">Keys</a>&lt;'a, K, V&gt;",synthetic:false,types:["phf::ordered_map::Keys"]},{text:"impl&lt;'a, K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"phf/ordered_map/struct.Values.html\" title=\"struct phf::ordered_map::Values\">Values</a>&lt;'a, K, V&gt;",synthetic:false,types:["phf::ordered_map::Values"]},{text:"impl&lt;'a, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"phf/ordered_set/struct.Iter.html\" title=\"struct phf::ordered_set::Iter\">Iter</a>&lt;'a, T&gt;",synthetic:false,types:["phf::ordered_set::Iter"]},];
implementors["regex"] = [{text:"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"regex/struct.SetMatchesIntoIter.html\" title=\"struct regex::SetMatchesIntoIter\">SetMatchesIntoIter</a>",synthetic:false,types:["regex::re_set::unicode::SetMatchesIntoIter"]},{text:"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"regex/struct.SetMatchesIter.html\" title=\"struct regex::SetMatchesIter\">SetMatchesIter</a>&lt;'a&gt;",synthetic:false,types:["regex::re_set::unicode::SetMatchesIter"]},{text:"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"regex/bytes/struct.SetMatchesIntoIter.html\" title=\"struct regex::bytes::SetMatchesIntoIter\">SetMatchesIntoIter</a>",synthetic:false,types:["regex::re_set::bytes::SetMatchesIntoIter"]},{text:"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"regex/bytes/struct.SetMatchesIter.html\" title=\"struct regex::bytes::SetMatchesIter\">SetMatchesIter</a>&lt;'a&gt;",synthetic:false,types:["regex::re_set::bytes::SetMatchesIter"]},];
implementors["serde_json"] = [{text:"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"serde_json/map/struct.Iter.html\" title=\"struct serde_json::map::Iter\">Iter</a>&lt;'a&gt;",synthetic:false,types:["serde_json::map::Iter"]},{text:"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"serde_json/map/struct.IterMut.html\" title=\"struct serde_json::map::IterMut\">IterMut</a>&lt;'a&gt;",synthetic:false,types:["serde_json::map::IterMut"]},{text:"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"serde_json/map/struct.IntoIter.html\" title=\"struct serde_json::map::IntoIter\">IntoIter</a>",synthetic:false,types:["serde_json::map::IntoIter"]},{text:"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"serde_json/map/struct.Keys.html\" title=\"struct serde_json::map::Keys\">Keys</a>&lt;'a&gt;",synthetic:false,types:["serde_json::map::Keys"]},{text:"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"serde_json/map/struct.Values.html\" title=\"struct serde_json::map::Values\">Values</a>&lt;'a&gt;",synthetic:false,types:["serde_json::map::Values"]},{text:"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"serde_json/map/struct.ValuesMut.html\" title=\"struct serde_json::map::ValuesMut\">ValuesMut</a>&lt;'a&gt;",synthetic:false,types:["serde_json::map::ValuesMut"]},];
implementors["smallvec"] = [{text:"impl&lt;'a, T:&nbsp;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"smallvec/struct.Drain.html\" title=\"struct smallvec::Drain\">Drain</a>&lt;'a, T&gt;",synthetic:false,types:["smallvec::Drain"]},{text:"impl&lt;A:&nbsp;<a class=\"trait\" href=\"smallvec/trait.Array.html\" title=\"trait smallvec::Array\">Array</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"smallvec/struct.IntoIter.html\" title=\"struct smallvec::IntoIter\">IntoIter</a>&lt;A&gt;",synthetic:false,types:["smallvec::IntoIter"]},];
implementors["unic_char_range"] = [{text:"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"unic_char_range/struct.CharIter.html\" title=\"struct unic_char_range::CharIter\">CharIter</a>",synthetic:false,types:["unic_char_range::iter::CharIter"]},];
implementors["unic_segment"] = [{text:"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"unic_segment/struct.GraphemeIndices.html\" title=\"struct unic_segment::GraphemeIndices\">GraphemeIndices</a>&lt;'a&gt;",synthetic:false,types:["unic_segment::grapheme::GraphemeIndices"]},{text:"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"unic_segment/struct.Graphemes.html\" title=\"struct unic_segment::Graphemes\">Graphemes</a>&lt;'a&gt;",synthetic:false,types:["unic_segment::grapheme::Graphemes"]},{text:"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"unic_segment/struct.Words.html\" title=\"struct unic_segment::Words\">Words</a>&lt;'a&gt;",synthetic:false,types:["unic_segment::word::Words"]},{text:"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"unic_segment/struct.WordBoundIndices.html\" title=\"struct unic_segment::WordBoundIndices\">WordBoundIndices</a>&lt;'a&gt;",synthetic:false,types:["unic_segment::word::WordBoundIndices"]},{text:"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/trait.DoubleEndedIterator.html\" title=\"trait core::iter::traits::DoubleEndedIterator\">DoubleEndedIterator</a> for <a class=\"struct\" href=\"unic_segment/struct.WordBounds.html\" title=\"struct unic_segment::WordBounds\">WordBounds</a>&lt;'a&gt;",synthetic:false,types:["unic_segment::word::WordBounds"]},];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
