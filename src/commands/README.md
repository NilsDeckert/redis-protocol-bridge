# Commands

In this folder, every file - except [parse](parse.rs) - contains the code to
parse and handle RESP commands of the same name.

All commands have a `default_handler` to mock the behaviour of a minimal server.
Commands with rather straight-forward implementations (e.g `SET`, `GET`) also have a `handler`
function, that returns a proper reply by using supplied arguments (e.g. a HashMap).

## TODO
Some commands that might be useful to implement for integration with other applications:

```Rust
match command.to_uppercase().as_str() {

    "EXISTS" => {
        reply = 0.as_frame();
    }

    "TYPE" => {
        reply = OwnedFrame::SimpleString {
            data: "string".into(),
            attributes: None,
        }
    }

    "TTL" => {
        reply = (-1).as_frame();
    }

    "MEMORY" => { /* USAGE */
        reply = 64.as_frame();
    }

    /* Return empty scan */
    "SCAN" => {
        reply = OwnedFrame::Array {
            data: vec![
                0.as_frame(),       /* Cursor */
                OwnedFrame::Array { /* Keys found */
                    data: vec![],
                    attributes: None,
                }
            ],
            attributes: None,
        }
    }

    "CLIENT" => {
        reply = ok_frame();
    }
}
```