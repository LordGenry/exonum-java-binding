// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use exonum::storage::map_index::{MapIndexIter, MapIndexKeys, MapIndexValues};
use exonum::storage::{Fork, MapIndex, Snapshot};
use jni::objects::{JClass, JObject, JString};
use jni::sys::{jboolean, jbyteArray, jobject};
use jni::JNIEnv;

use std::panic;
use std::ptr;

use storage::db::{Key, Value, View, ViewRef};
use utils::{self, Handle, PairIter};

type Index<T> = MapIndex<T, Key, Value>;

enum IndexType {
    SnapshotIndex(Index<&'static Snapshot>),
    ForkIndex(Index<&'static mut Fork>),
}

type Iter<'a> = PairIter<MapIndexIter<'a, Key, Value>>;

const JAVA_ENTRY_FQN: &str = "com/exonum/binding/storage/indices/MapEntryInternal";

/// Returns a pointer to the created `MapIndex` object.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeCreate(
    env: JNIEnv,
    _: JClass,
    name: JString,
    view_handle: Handle,
) -> Handle {
    let res = panic::catch_unwind(|| {
        let name = utils::convert_to_string(&env, name)?;
        Ok(utils::to_handle(
            match *utils::cast_handle::<View>(view_handle).get() {
                ViewRef::Snapshot(snapshot) => {
                    IndexType::SnapshotIndex(Index::new(name, &*snapshot))
                }
                ViewRef::Fork(ref mut fork) => IndexType::ForkIndex(Index::new(name, fork)),
            },
        ))
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Returns a pointer to the created `MapIndex` instance in an index family (= group).
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeCreateInGroup(
    env: JNIEnv,
    _: JClass,
    group_name: JString,
    map_id: jbyteArray,
    view_handle: Handle,
) -> Handle {
    let res = panic::catch_unwind(|| {
        let group_name = utils::convert_to_string(&env, group_name)?;
        let map_id = env.convert_byte_array(map_id)?;
        let view_ref = utils::cast_handle::<View>(view_handle).get();
        Ok(utils::to_handle(match *view_ref {
            ViewRef::Snapshot(snapshot) => {
                IndexType::SnapshotIndex(Index::new_in_family(group_name, &map_id, &*snapshot))
            }
            ViewRef::Fork(ref mut fork) => {
                IndexType::ForkIndex(Index::new_in_family(group_name, &map_id, fork))
            }
        }))
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Destroys the underlying `MapIndex` object and frees memory.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeFree(
    env: JNIEnv,
    _: JClass,
    map_handle: Handle,
) {
    utils::drop_handle::<IndexType>(&env, map_handle);
}

/// Returns value identified by the `key`. Null pointer is returned if value is not found.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeGet(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
    key: jbyteArray,
) -> jbyteArray {
    let res = panic::catch_unwind(|| {
        let key = env.convert_byte_array(key)?;
        let val = match *utils::cast_handle::<IndexType>(map_handle) {
            IndexType::SnapshotIndex(ref map) => map.get(&key),
            IndexType::ForkIndex(ref map) => map.get(&key),
        };
        match val {
            Some(val) => env.byte_array_from_slice(&val),
            None => Ok(ptr::null_mut()),
        }
    });
    utils::unwrap_exc_or(&env, res, ptr::null_mut())
}

/// Returns `true` if the map contains a value for the specified key.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeContainsKey(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
    key: jbyteArray,
) -> jboolean {
    let res = panic::catch_unwind(|| {
        let key = env.convert_byte_array(key)?;
        Ok(match *utils::cast_handle::<IndexType>(map_handle) {
            IndexType::SnapshotIndex(ref map) => map.contains(&key),
            IndexType::ForkIndex(ref map) => map.contains(&key),
        } as jboolean)
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Returns the pointer to the iterator over a map keys and values.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeCreateEntriesIter(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
) -> Handle {
    let res = panic::catch_unwind(|| {
        let iter = match *utils::cast_handle::<IndexType>(map_handle) {
            IndexType::SnapshotIndex(ref map) => map.iter(),
            IndexType::ForkIndex(ref map) => map.iter(),
        };
        let iter = Iter::new(&env, iter, JAVA_ENTRY_FQN)?;
        Ok(utils::to_handle(iter))
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Returns a pointer to the iterator over map keys.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeCreateKeysIter(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
) -> Handle {
    let res = panic::catch_unwind(|| {
        Ok(utils::to_handle(
            match *utils::cast_handle::<IndexType>(map_handle) {
                IndexType::SnapshotIndex(ref map) => map.keys(),
                IndexType::ForkIndex(ref map) => map.keys(),
            },
        ))
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Returns a pointer to the iterator over map values.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeCreateValuesIter(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
) -> Handle {
    let res = panic::catch_unwind(|| {
        Ok(utils::to_handle(
            match *utils::cast_handle::<IndexType>(map_handle) {
                IndexType::SnapshotIndex(ref map) => map.values(),
                IndexType::ForkIndex(ref map) => map.values(),
            },
        ))
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Returns the pointer to the iterator over a map keys and values starting at the given key.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeCreateIterFrom(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
    key: jbyteArray,
) -> Handle {
    let res = panic::catch_unwind(|| {
        let key = env.convert_byte_array(key)?;
        let iter = match *utils::cast_handle::<IndexType>(map_handle) {
            IndexType::SnapshotIndex(ref map) => map.iter_from(&key),
            IndexType::ForkIndex(ref map) => map.iter_from(&key),
        };
        let iter = Iter::new(&env, iter, JAVA_ENTRY_FQN)?;
        Ok(utils::to_handle(iter))
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Returns a pointer to the iterator over map keys starting at the given key.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeKeysFrom(
    env: JNIEnv,
    _: JClass,
    map_handle: Handle,
    key: jbyteArray,
) -> Handle {
    let res = panic::catch_unwind(|| {
        let key = env.convert_byte_array(key)?;
        Ok(utils::to_handle(
            match *utils::cast_handle::<IndexType>(map_handle) {
                IndexType::SnapshotIndex(ref map) => map.keys_from(&key),
                IndexType::ForkIndex(ref map) => map.keys_from(&key),
            },
        ))
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Returns a pointer to the iterator over map values starting at the given key.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeValuesFrom(
    env: JNIEnv,
    _: JClass,
    map_handle: Handle,
    key: jbyteArray,
) -> Handle {
    let res = panic::catch_unwind(|| {
        let key = env.convert_byte_array(key)?;
        Ok(utils::to_handle(
            match *utils::cast_handle::<IndexType>(map_handle) {
                IndexType::SnapshotIndex(ref map) => map.values_from(&key),
                IndexType::ForkIndex(ref map) => map.values_from(&key),
            },
        ))
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Sets `value` identified by the `key` into the index.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativePut(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
    key: jbyteArray,
    value: jbyteArray,
) {
    let res = panic::catch_unwind(|| match *utils::cast_handle::<IndexType>(map_handle) {
        IndexType::SnapshotIndex(_) => {
            panic!("Unable to modify snapshot.");
        }
        IndexType::ForkIndex(ref mut map) => {
            let key = env.convert_byte_array(key)?;
            let value = env.convert_byte_array(value)?;
            map.put(&key, value);
            Ok(())
        }
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Removes value identified by the `key` from the index.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeRemove(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
    key: jbyteArray,
) {
    let res = panic::catch_unwind(|| match *utils::cast_handle::<IndexType>(map_handle) {
        IndexType::SnapshotIndex(_) => {
            panic!("Unable to modify snapshot.");
        }
        IndexType::ForkIndex(ref mut map) => {
            let key = env.convert_byte_array(key)?;
            map.remove(&key);
            Ok(())
        }
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Clears the index, removing all values.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeClear(
    env: JNIEnv,
    _: JObject,
    map_handle: Handle,
) {
    let res = panic::catch_unwind(|| match *utils::cast_handle::<IndexType>(map_handle) {
        IndexType::SnapshotIndex(_) => {
            panic!("Unable to modify snapshot.");
        }
        IndexType::ForkIndex(ref mut map) => {
            map.clear();
            Ok(())
        }
    });
    utils::unwrap_exc_or_default(&env, res)
}

/// Returns the next value from the iterator. Returns null pointer when iteration is finished.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeEntriesIterNext(
    env: JNIEnv,
    _: JObject,
    iter_handle: Handle,
) -> jobject {
    let res = panic::catch_unwind(|| {
        let iterWrapper = utils::cast_handle::<Iter>(iter_handle);
        match iterWrapper.iter.next() {
            Some(val) => {
                let key: JObject = env.byte_array_from_slice(&val.0)?.into();
                let value: JObject = env.byte_array_from_slice(&val.1)?.into();
                Ok(env
                    .new_object_by_id(
                        &iterWrapper.element_class,
                        iterWrapper.constructor_id,
                        &[key.into(), value.into()],
                    )?.into_inner())
            }
            None => Ok(ptr::null_mut()),
        }
    });
    utils::unwrap_exc_or(&env, res, ptr::null_mut())
}

/// Destroys the underlying `MapIndex` iterator object and frees memory.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeEntriesIterFree(
    env: JNIEnv,
    _: JObject,
    iter_handle: Handle,
) {
    utils::drop_handle::<Iter>(&env, iter_handle);
}

/// Returns the next value from the keys-iterator. Returns null pointer when iteration is finished.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeKeysIterNext(
    env: JNIEnv,
    _: JObject,
    iter_handle: Handle,
) -> jbyteArray {
    let res = panic::catch_unwind(|| {
        let iter = utils::cast_handle::<MapIndexKeys<Key>>(iter_handle);
        match iter.next() {
            Some(val) => env.byte_array_from_slice(&val),
            None => Ok(ptr::null_mut()),
        }
    });
    utils::unwrap_exc_or(&env, res, ptr::null_mut())
}

/// Destroys the underlying `MapIndex` keys-iterator object and frees memory.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeKeysIterFree(
    env: JNIEnv,
    _: JObject,
    iter_handle: Handle,
) {
    utils::drop_handle::<MapIndexKeys<Key>>(&env, iter_handle);
}

/// Return next value from the values-iterator. Returns null pointer when iteration is finished.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeValuesIterNext(
    env: JNIEnv,
    _: JObject,
    iter_handle: Handle,
) -> jbyteArray {
    let res = panic::catch_unwind(|| {
        let iter = utils::cast_handle::<MapIndexValues<Value>>(iter_handle);
        match iter.next() {
            Some(val) => env.byte_array_from_slice(&val),
            None => Ok(ptr::null_mut()),
        }
    });
    utils::unwrap_exc_or(&env, res, ptr::null_mut())
}

/// Destroys the underlying `MapIndex` values-iterator object and frees memory.
#[no_mangle]
pub extern "system" fn Java_com_exonum_binding_storage_indices_MapIndexProxy_nativeValuesIterFree(
    env: JNIEnv,
    _: JObject,
    iter_handle: Handle,
) {
    utils::drop_handle::<MapIndexValues<Value>>(&env, iter_handle);
}
