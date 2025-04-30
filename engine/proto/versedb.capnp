@0xa57efb7a533a9732;

struct KeyValue {
    key @0 :Data;
    value @1 :Data;
}

struct KeyValuePair {
    key @0 :Data;
    value @1 :Data;
}

struct KeyRange {
    start @0 :Data;
    end @1 :Data;
}

interface Versedb {
    add @0 (key :Data, value :Data) -> ();
    select @1 (key :Data) -> (value :Data);
    remove @2 (key :Data) -> ();
    selectRange @3 (range :KeyRange) -> (pairs :List(KeyValuePair));
    helloworld @4 (input :Text) -> (output :Text);
    flush @5 () -> ();
} 