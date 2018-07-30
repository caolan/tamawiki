initSidebarItems({"fn":[["compile_time_assert_clone","`compile_time_assert_clone::<T>();` fails to compile if `T` doesn't implement `Clone`."],["compile_time_assert_copy","`compile_time_assert_copy::<T>();` fails to compile if `T` doesn't implement `Copy`."],["compile_time_assert_debug","`compile_time_assert_debug::<T>();` fails to compile if `T` doesn't implement `Debug`."],["compile_time_assert_send","`compile_time_assert_send::<T>();` fails to compile if `T` doesn't implement `Send`."],["compile_time_assert_sync","`compile_time_assert_sync::<T>();` fails to compile if `T` doesn't implement `Sync`."],["from_file","Reads test cases out of the file with the path given by `test_data_relative_file_path`, calling `f` on each vector until `f` fails or until all the test vectors have been read. `f` can indicate failure either by returning `Err()` or by panicking."],["from_hex","Decode an string of hex digits into a sequence of bytes. The input must have an even number of digits."],["ring_src_path","Returns the path for ring source code root."]],"mod":[["rand","Deterministic implementations of `ring::rand::SecureRandom`."]],"struct":[["TestCase","A test case. A test case consists of a set of named attributes. Every attribute in the test case must be consumed exactly once; this helps catch typos and omissions."]]});