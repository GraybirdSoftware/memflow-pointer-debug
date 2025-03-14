# memflow-pointer-debug

Simple macro used for automatically expanding structs with `Pointer<T>` fields inside

### Example output 
```
_EPROCESS {
  pcb: _KPROCESS = _KPROCESS { directory_table_base: 7ddb29000 }
  peb-> _PEB
    being_debugged: bool = false
  }
}
```


# Usage

Simply add the derive macro to each field with pointers you wish to be expanded
 - field names that contain `_pad` will be ignored
 - there is a depth limit, the default is 5 but you can set it yourself with `pointer_print_with_depth`
 - macro will match on `Pointer` in the actual typename. Ensure that you dont have pointer anywhere in your typenames for struct fields

```rs
offset_debug!(
    #[derive(PointerDerefDebugPrint)]
    struct _EPROCESS  {
        0x0 pcb: _KPROCESS,
        0x2e0 peb: Pointer64<_PEB>,
    }
);

offset_debug!(
    #[derive(PointerDerefDebugPrint)]
    struct _KPROCESS  {
        0x28 directory_table_base: Address,
    }
);

offset_debug!(
    #[derive(PointerDerefDebugPrint)]
    struct _PEB {
        0x2 being_debugged: bool,
    }
);


let eprocess = process.read::<_EPROCESS>(p.address)?;
let dtb = eprocess.pcb.directory_table_base;
let pcb = eprocess.peb.read(&mut process)?;
// call function directly
print_with_pointer_reading(&eprocess, &mut process);
// call impl on variable
eprocess.pointer_print_with_depth(&mut process, 2);
```




