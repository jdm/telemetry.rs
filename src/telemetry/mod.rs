extern crate rustc_serialize;
use self::rustc_serialize::json::Json;

extern crate vec_map;
use self::vec_map::VecMap;

use std::marker::PhantomData;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::mem::size_of;
use std::sync::Arc;
use std::cell::Cell;

//
// Telemetry is a mechanism used to capture metrics in an application,
// and either store the data locally or upload to a server for
// statistical analysis.
//
// Examples of usage:
// - capturing the speed of an operation;
// - finding out if users are actually using a feature;
// - finding out how the duration of a session;
// - determine the operating system on which the application is executed;
// - determining the configuration of the application;
// - capturing the operations that slow down the application;
// - determining the amount of I/O performed by the application;
// - ...
//
// The abstraction used by this library is the Histogram. Each
// Histogram serves to capture a specific measurement, store it
// locally and/or upload it to the server. Several types of Histograms
// are provided, suited to distinct kinds of measures.
//
//
// Memory note: the memory used by a histogram is recollected only
// when its instance of `telemetry` is garbage-collected. In other words,
// if a histogram goes out of scope for some reason, its data remains
// in telemetry and will be stored and/or uploaded in accordance with the
// configuration of this telemetry instance.
//

//
// A software version, e.g. [2015, 10, 10, 0]
//
pub type Version = [u32;4];

//
// Metadata on a histogram.
//
pub struct Metadata {
    // A key used to identify the histogram. Must be unique to the instance
    // of `telemetry`.
    pub key: String,

    // Optionally, a version of the product at which this histogram expires.
    pub expires: Option<Version>,
}

//
// A single histogram.
//
trait Histogram<T> {
    //
    // Record a value in this histogram.
    //
    // The value is recorded only if all of the following conditions are met:
    // - telemetry is activated; and
    // - this histogram has not expired; and
    // - the histogram is active.
    //
    fn record(&self, value: T) {
        self.record_cb(|| Some(value))
    }

    //
    // Record a value in this histogram, as provided by a callback.
    //
    // The callback is triggered only if all of the following conditions are met:
    // - telemetry is activated; and
    // - this histogram has not expired; and
    // - the histogram is active.
    //
    // If the callback returns `None`, no value is recorded.
    //
    fn record_cb<F>(&self, _: F) where F: FnOnce() -> Option<T>;
}

//
// A family of histograms, indexed by some value. Use these to
// monitor families of values that cannot be determined at
// compile-time, e.g. add-ons, programs, etc.
//
trait HistogramMap<K, T> {
    //
    // Record a value in this histogram.
    //
    // The value is recorded only if all of the following conditions are met:
    // - telemetry is activated; and
    // - this histogram has not expired; and
    // - the histogram is active.
    //
    fn record(&self, key: K, value: T) {
        self.record_cb(|| Some((key, value)))
    }

    //
    // Record a value in this histogram, as provided by a callback.
    //
    // The callback is triggered only if all of the following conditions are met:
    // - `telemetry` is activated; and
    // - this histogram has not expired; and
    // - the histogram is active.
    //
    // If the callback returns `None`, no value is recorded.
    //
    fn record_cb<F>(&self, _: F) where F: FnOnce() -> Option<(K, T)>;
}


pub trait Flatten<T> {
    fn as_u32(&self) -> u32;
}

impl Flatten<u32> for u32 {
    fn as_u32(&self) -> u32 {
        *self
    }
}

//
//
// Flag histograms.
//
// This histogram type allows you to record a single value. This type
// is useful if you need to track whether a feature was ever used
// during a session. You only need to add a single line of code which
// sets the flag when the feature is used because the histogram is
// initialized with a default value of false (flag not set).
//
//

// Single histogram, good for recording a single value.
pub type FlagSingle = FlagFront<Flat>;

// Map histogram, good for recording the presence of a set of values,
// when the set cannot be known at compile-time. If the set is known
// at compile-time, you should prefer several instances of
// `FlagSingle`.
pub type FlagMap<T> = FlagFront<Keyed<T>>;


struct FlagFront<K> {
    back_end: BackEnd<K>,
}

struct FlagStorage {
    // `true` once we have called `record`, `false` until then.
    encountered: bool
}


impl RawStorage for FlagStorage {
    fn store(&mut self, _: u32) {
        self.encountered = true;
    }
    fn serialize(&self) -> Json {
        unreachable!() // FIXME: Implement
    }
}

impl Histogram<()> for FlagSingle {
    fn record_cb<F>(&self, cb: F) where F: FnOnce() -> Option<()>  {
        if let Some(k) = self.back_end.get_key() {
            match cb() {
                None => {}
                Some(()) => self.back_end.raw_record(&k, 0)
            }
        }
    }
}


impl FlagSingle {
    pub fn new(feature: &Feature, meta: Metadata) -> FlagSingle {
        let storage = Box::new(FlagStorage { encountered: false });
        let key = feature.telemetry.register_flat(meta, storage);
        FlagFront {
            back_end: BackEnd::new(feature, key),
        }
    }
}

// Map histogram, good for a family of values. Note that if the family
// of values is known at compile-time, using a set of `Flag` instead of
// a single `FlagMap` is both more efficient and more type-safe.

impl<K> FlagMap<K> where K: ToString {
    pub fn new(feature: &Feature, meta: Metadata) -> FlagMap<K> {
        let storage = Box::new(FlagStorageMap { encountered: HashSet::new() });
        let key = feature.telemetry.register_keyed(meta, storage);
        FlagMap {
            back_end: BackEnd::new(feature, key),
        }
    }
}

struct FlagStorageMap {
    encountered: HashSet<String>
}

impl RawStorageMap for FlagStorageMap {
    fn store(&mut self, k: String, _: u32) {
        self.encountered.insert(k);
    }
    fn serialize(&self) -> Json {
        unreachable!() // FIXME: Implement
    }
}

impl<K> HistogramMap<K, ()> for FlagMap<K> where K: ToString {
    fn record_cb<F>(&self, cb: F) where F: FnOnce() -> Option<(K, ())>  {
        if let Some(k) = self.back_end.get_key() {
            match cb() {
                None => {}
                Some((key, ())) => self.back_end.raw_record(&k, key.to_string(), 0)
            }
        }
    }
}



//
// Linear histograms.
//
//
// Linear histograms classify numeric integer values into same-sized
// buckets. This type is typically used for percentages.
//


pub type LinearSingle<T> = LinearFront<Flat, T>;
pub type LinearMap<K, T> = LinearFront<Keyed<K>, T>;

struct LinearStorage {
    values: Vec<u32>// We cannot use an array here, as this would make the struct unsized
}

impl LinearStorage {
    fn new(capacity: usize) -> LinearStorage {
        let mut vec = Vec::with_capacity(capacity);
        unsafe {
            // Resize. In future versions of Rust, we should
            // be able to use `vec.resize`.
            vec.set_len(capacity);
            for i in 0 .. capacity - 1 {
                vec[i] = 0;
            }
        }
        LinearStorage {
            values: vec
        }
    }
}

impl RawStorage for LinearStorage {
    fn store(&mut self, index: u32) {
        self.values[index as usize] += 1;
    }
    fn serialize(&self) -> Json {
        unreachable!() // FIXME: Implement
    }
}

pub struct LinearFront<K, T> where T: Flatten<T> {
    witness: PhantomData<T>,
    back_end: BackEnd<K>,
    min: u32,
    max: u32, // Invariant: max > min
    buckets: u32 // Invariant: sizeof(u32) <= sizeof(usize)
}

impl<K, T> LinearFront<K, T> where T: Flatten<T> {
    fn get_bucket(&self, value: T) -> u32 {
        let value = value.as_u32();
        if value >= self.max {
            0
        } else if value <= self.min {
            self.buckets - 1 as u32
        } else {
            let num = value as f32 - self.min as f32;
            let den = self.max as f32 - self.min as f32;
            let res = (num / den) * self.buckets as f32;
            res as u32
        }
    }
}

impl<T> Histogram<T> for LinearSingle<T> where T: Flatten<T> {
    fn record_cb<F>(&self, cb: F) where F: FnOnce() -> Option<T>  {
        if let Some(k) = self.back_end.get_key() {
            match cb() {
                None => {}
                Some(v) => self.back_end.raw_record(&k, self.get_bucket(v))
            }
        }
    }
}

impl<T> LinearSingle<T> where T: Flatten<T> {
    fn new(feature: &Feature, meta: Metadata, min: u32, max: u32, buckets: usize) -> LinearSingle<T> {
        assert!(size_of::<u32>() <= size_of::<usize>());
        assert!(min < max);
        assert!(max - min >= buckets as u32);
        let storage = Box::new(LinearStorage::new(buckets));
        let key = feature.telemetry.register_flat(meta, storage);
        LinearFront {
            witness: PhantomData,
            back_end: BackEnd::new(feature, key),
            min: min,
            max: max,
            buckets: buckets as u32
        }
    }
}


struct LinearStorageMap {
    values: HashMap<String, Vec<u32>>,
    capacity: usize
}

impl LinearStorageMap {
    fn new(buckets: usize) -> LinearStorageMap {
        LinearStorageMap {
            values: HashMap::new(),
            capacity: buckets,
        }
    }
}

impl RawStorageMap for LinearStorageMap {
    fn store(&mut self, key: String, index: u32) {
        match self.values.entry(key) {
            Occupied(mut e) => {
                e.get_mut()[index as usize] += 1;
            }
            Vacant(e) => {
                let mut vec = Vec::with_capacity(self.capacity);
                unsafe {
                    // Resize. In future versions of Rust, we should
                    // be able to use `vec.resize`.
                    vec.set_len(self.capacity);
                    for i in 0 .. self.capacity - 1 {
                        vec[i] = 0;
                    }
                }
                vec[index as usize] += 1;
                e.insert(vec);
            }
        }
    }
    fn serialize(&self) -> Json {
        unreachable!() // FIXME: Implement
    }
}


impl<K, T> LinearMap<K, T> where K: ToString, T: Flatten<T> {
    fn new(feature: &Feature, meta: Metadata, min: u32, max: u32, buckets: usize) -> LinearMap<K, T> {
        assert!(size_of::<u32>() <= size_of::<usize>());
        assert!(min < max);
        assert!(max - min >= buckets as u32);
        let storage = Box::new(LinearStorageMap::new(buckets));
        let key = feature.telemetry.register_keyed(meta, storage);
        LinearFront {
            witness: PhantomData,
            back_end: BackEnd::new(feature, key),
            min: min,
            max: max,
            buckets: buckets as u32
        }
    }
}

impl<K, T> HistogramMap<K, T> for LinearMap<K, T> where K: ToString, T: Flatten<T> {
    fn record_cb<F>(&self, cb: F) where F: FnOnce() -> Option<(K, T)>  {
        if let Some(k) = self.back_end.get_key() {
            match cb() {
                None => {}
                Some((key, v)) => self.back_end.raw_record(&k, key.to_string(), self.get_bucket(v))
            }
        }
    }
}

//
// A group of histograms observed by Telemetry.
//
impl Feature {
    //
    // Create a new feature.
    //
    // New features are deactivated by default.
    //
    pub fn new(telemetry: &Arc<Telemetry>) -> Feature {
        Feature {
            is_active: Arc::new(Cell::new(false)),
            sender: telemetry.sender.clone(),
            telemetry: telemetry.clone(),
        }
    }
}

//
// The Telemetry service.
//
// Generally, an application will have only a single instance of this
// service but may have any number of instances of `Feature` which may
// be activated and deactivated individually.
//
impl Telemetry {
    pub fn new(version: Version) -> Telemetry {
        let (sender, receiver) = channel();
        thread::spawn(|| {
            let mut data = TelemetryTask::new();
            for msg in receiver {
                match msg {
                    Op::RegisterFlat(index, storage) => {
                        data.flat.insert(index, storage);
                    }
                    Op::RegisterKeyed(index, storage) => {
                        data.keyed.insert(index, storage);
                    }
                    Op::RecordFlat(index, value) => {
                        let ref mut storage = data.flat.get_mut(&index).unwrap();
                        storage.store(value);
                    }
                    Op::RecordKeyed(index, key, value) => {
                        let ref mut storage = data.keyed.get_mut(&index).unwrap();
                        storage.store(key, value);
                    }
                    Op::Serialize(_) => {
                        unreachable!() // Implement
                    }
                }
            }
        });
        Telemetry {
            keys_flat: KeyGenerator::new(),
            keys_keyed: KeyGenerator::new(),
            version: version,
            sender: sender,
        }
    }

    fn register_flat(&self, meta: Metadata, storage: Box<RawStorage>) -> Option<Key<Flat>> {
        // Don't bother adding the histogram if it is expired.
        match meta.expires {
            Some(v) if v <= self.version => return None,
            _ => {}
        }

        let key = self.keys_flat.next();
        self.sender.send(Op::RegisterFlat(key.index, storage)).unwrap();
        Some(key)
    }

    fn register_keyed<T>(&self, meta: Metadata, storage: Box<RawStorageMap>) -> Option<Key<Keyed<T>>> {
        // Don't bother adding the histogram if it is expired.
        match meta.expires {
            Some(v) if v <= self.version => return None,
            _ => {}
        }

        let key = self.keys_keyed.next();
        self.sender.send(Op::RegisterKeyed(key.index, storage)).unwrap();
        Some(key)
    }
}

pub struct Telemetry {
    // The version of the product. Some histograms may be limited to
    // specific versions of the product.
    version: Version,

    // A key generator for registration of new histograms. Uses atomic
    // to avoid the use of &mut.
    keys_flat: KeyGenerator<Flat>,
    keys_keyed: KeyGenerator<Map>,

    // Connection to the thread holding all the storage of this
    // instance of telemetry.
    sender: Sender<Op>,
}

pub struct Feature {
    // Are measurements active for this feature?
    is_active: Arc<Cell<bool>>,
    sender: Sender<Op>,
    telemetry: Arc<Telemetry>,
}

//
// Low-level, untyped, implementation of histogram storage.
//
trait RawStorage: Send {
    fn store(&mut self, value: u32);
    fn serialize(&self) -> Json;
}
trait RawStorageMap: Send {
    fn store(&mut self, key: String, value: u32);
    fn serialize(&self) -> Json;
}

//
// Features shared by all histograms
//
struct BackEnd<K> {
    // A key used to map a histogram to its storage owned by telemetry,
    // or None if the histogram has been rejected by telemetry because
    // it has expired.
    key: Option<Key<K>>,

    // `true` unless the histogram has been deactivated by user request.
    // If `false`, no data will be recorded for this histogram.
    is_active: bool,

    sender: Sender<Op>,
    is_feature_active: Arc<Cell<bool>>,
}

impl<K> BackEnd<K> {
    fn new(feature: &Feature, key: Option<Key<K>>) -> BackEnd<K> {
        BackEnd {
            key: key,
            is_active: true,
            sender: feature.sender.clone(),
            is_feature_active: feature.is_active.clone(),
        }
    }

    fn get_key(&self) -> Option<&Key<K>>
    {
        if !self.is_active {
            return None;
        }
        if !self.is_feature_active.get() {
            return None;
        }
        match self.key {
            None => None,
            Some(ref k) => Some(k)
        }
    }
}

impl BackEnd<Flat> {
    fn raw_record(&self, k: &Key<Flat>, value: u32) {
        self.sender.send(Op::RecordFlat(k.index, value)).unwrap();
    }
}

impl<T> BackEnd<Keyed<T>> {
    fn raw_record(&self, k: &Key<Keyed<T>>, key: String, value: u32) {
        self.sender.send(Op::RecordKeyed(k.index, key, value)).unwrap();
    }
}

struct Key<T> {
    witness: PhantomData<T>,
    index: usize,
}
struct KeyGenerator<T> {
    counter: AtomicUsize,
    witness: PhantomData<T>,
}
impl<T> KeyGenerator<T> {
    fn new() -> KeyGenerator<T> {
        KeyGenerator {
            counter: AtomicUsize::new(0),
            witness: PhantomData,
        }
    }
}
impl KeyGenerator<Flat> {
    fn next(&self) -> Key<Flat> {
        Key {
            index: self.counter.fetch_add(1, Ordering::Relaxed),
            witness: PhantomData
        }
    }
}
impl KeyGenerator<Map> {
    fn next<T>(&self) -> Key<Keyed<T>> {
        Key {
            index: self.counter.fetch_add(1, Ordering::Relaxed),
            witness: PhantomData
        }
    }
}

pub struct Flat;
pub struct Map;
pub struct Keyed<T> {
    witness: PhantomData<T>
}

enum Op {
    RegisterFlat(usize, Box<RawStorage>),
    RegisterKeyed(usize, Box<RawStorageMap>),
    RecordFlat(usize, u32),
    RecordKeyed(usize, String, u32),
    Serialize(Sender<Json>),
}

struct TelemetryTask {
    flat: VecMap<Box<RawStorage>>,
    keyed: VecMap<Box<RawStorageMap>>
}

impl TelemetryTask {
    fn new() -> TelemetryTask {
        TelemetryTask {
            flat: VecMap::new(),
            keyed: VecMap::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use telemetry::{Histogram, HistogramMap};

    #[test]
    fn create_flags() {
        let telemetry = Arc::new(Telemetry::new([0, 0, 0, 0]));
        let feature = Feature::new(&telemetry);
        let flag_single = FlagSingle::new(&feature, Metadata { key: "Test linear single".to_string(), expires: None});
        let flag_map = FlagMap::new(&feature, Metadata { key: "Test flag map".to_string(), expires: None});

        flag_single.record(());
        flag_map.record("key".to_string(), ());

        feature.is_active.set(true);
        flag_single.record(());
        flag_map.record("key".to_string(), ());
    }

    #[test]
    fn create_linears() {
        let telemetry = Arc::new(Telemetry::new([0, 0, 0, 0]));
        let feature = Feature::new(&telemetry);
        let linear_single =
            LinearSingle::new(&feature,
                              Metadata {
                                  key: "Test linear single".to_string(),
                                  expires: None
                              }, 0, 100, 10);
        let linear_map =
            LinearMap::new(&feature,
                              Metadata {
                                  key: "Test linear map".to_string(),
                                  expires: None
                              }, 0, 100, 10);

        linear_single.record(0);
        linear_map.record("key".to_string(), 0);

        feature.is_active.set(true);
        linear_single.record(0);
        linear_map.record("key".to_string(), 0);
    }

}
