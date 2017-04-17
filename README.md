# Why

To learn rust!

- This library is used for console based `ID3 tagging` tool [Markdang](https://github.com/freestrings/markdang).

# Usage

This is `ID3` read and write library.

- [Add dependency](#add-dependency)
- [Reding: How to read `ID3` information](#reding-how-to-read-id3-information)
- [Writing: How to write `ID3` information](#writing-how-to-write-id3-information)
- [Rewrite: How to rewrite all `ID3` information to version 4](#rewrite-how-to-rewrite-a-id3-information-to-version-4)
- [Getting information of a frame body without property name.](#getting-information-of-a-frame-body-without-property-name)

other usecases [See tests](./tests/metadata.rs).

## Add dependency

This can be used by adding `rtag` to your dependencies in your project's `Cargo.toml`

```toml
[dependencies]
rtag = "0.3.5"
```
and this to your crate root:

```rust
extern crate rtag;
```

## Reding: How to read `ID3` information

To read a `ID3` metadata, you use a [MetadataReader](./src/metadata.rs#L50) and a [Unit](./src/metadata.rs#L36) enum. 


and the `MetadataReader` is implementing the [std::iter::Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html) trait, 
you can use [filter](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.filter), [map](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.map), [fold](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.fold).. and so on.

### Example

```rust
// read a frame1
for m in MetadataReader::new("./test-resources/v1-v2.mp3").unwrap() {
    match m {
        Unit::FrameV1(frame) => {
            debug!("v1: {:?}", frame);
            assert_eq!("Artist", frame.artist);
            assert_eq!("!@#$", frame.comment);
            assert_eq!("1", frame.track);
            assert_eq!("137", frame.genre);
        }
        _ => (),
    }
}

// filter a frame v3 having `compression flag`
let mut i = MetadataReader::new(path).unwrap().filter(|m| match m {
    &Unit::FrameV2(FrameHeader::V23(ref header), _) => {
        header.has_flag(FrameHeaderFlag::Compression)
    }
    _ => false,
});

// fold a frame v2
let new_data = MetadataReader::new(path)
    .unwrap()
    .fold(Vec::new(), |mut vec, unit| {
        if let Unit::FrameV2(frame_head, frame_body) = unit {
            let new_frame_body = if let FrameBody::TALB(ref frame) = frame_body {
                let mut new_frame = frame.clone();
                new_frame.text = "Album!".to_string();
                FrameBody::TALB(new_frame)
            } else {
                frame_body.clone()
            };

            vec.push(Unit::FrameV2(frame_head, new_frame_body));
        } else {
            vec.push(unit);
        }

        vec
    });
```

## Writing: How to write `ID3` information 

To write a `ID3` metadata, you pass [FrameHeader](./src/frame.rs#L267) and [FrameBody](./src/frame.rs#L2022) to [MetadataWriter](./src/metadata.rs#L366) via [std::vec::Vec](https://doc.rust-lang.org/std/vec/struct.Vec.html).

### Example

```rust
let new_data = MetadataReader::new(path)
    .unwrap()
    .fold(Vec::new(), |mut vec, unit| {
        if let Unit::FrameV2(frame_head, frame_body) = unit {
            let new_frame_body = ...
            vec.push(Unit::FrameV2(frame_head, new_frame_body));
        }

        vec
    });

let _ = MetadataWriter::new(path).unwrap().write(new_data, false);
```

## Rewrite: How to rewrite a `ID3` information to version 4

To rewrite all the frames to version 4, it is same to above example but second parameter is `true`. 
> Note: the frame v1 information is ignored and some frames that are ignored.
> - In 2.2 'CRM', 'PIC'. 
> - In 2.3 'EQUA', 'IPLS', 'RVAD', 'TDAT', 'TIME', 'TORY', 'TRDA', 'TSIZ', 'TYER'

```rust
// collect frames having version 2
let frames = MetadataReader::new(path).unwrap().collect::<Vec<Unit>>();
// rewrite to version 4
let _ = MetadataWriter::new(path).unwrap().write(frames, true);
// read a version 4
for unit in MetadataReader::new(path).unwrap() {
    match unit {
        Unit::FrameV2(FrameHeader::V24(head), frame_body) => {
            ...
        },
        _ => (),
    }
}
```

## Getting information of a frame body without property name.

To read value of frame without property name, FrameBody support `to_map` and `inside`.

### Example

```rust
for unit in MetadataReader::new(path).unwrap() {
    match unit {
        Unit::FrameV2(_, ref frame_body) => {
            
            // 1. using to_map();
            let map = frame_body.to_map();
            //{
            //    <key1:&str>: <value1:String>
            //    ...
            //}

            // 2. using inside
            frame_body.inside(|key, value| {
                // key<&str>, value<String>
                ...

                true // if true, look inside next.
            })

        },
        _ => (),
    }
}
```