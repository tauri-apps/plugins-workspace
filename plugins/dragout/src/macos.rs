#![allow(deprecated)]
// macOS native drag-out implementation using `NSFilePromiseProvider`.
// Минимальная рабочая реализация: создаёт `NSFilePromiseProvider`, запускает
// `beginDraggingSession` и копирует файл в destination в колбэке `writePromiseToURL`.

use cocoa::appkit::NSApp;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSArray, NSString, NSURL, NSAutoreleasePool, NSPoint, NSRect, NSSize};
use cocoa::appkit::NSPasteboard;
use objc::{class, msg_send, sel, sel_impl};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use dispatch::Queue;
use once_cell::sync::OnceCell;

const NS_DRAG_OPERATION_COPY: u64 = 1;

pub fn init() {
    println!("[dragout] macOS drag-out initialised");
}

/// Запускает drag-сессию для одного файла.
pub fn start_drag(archive_path: &str, file_path: &str) -> Result<(), String> {
    println!("[dragout] start_drag called: archive='{}' file='{}'", archive_path, file_path);
    // На macOS все UI-операции должны выполняться в главном потоке.
    // Если мы вызываемся из фонового таури-потока, перекинем задачу
    // в main queue и вернём Ok без ожидания.
    unsafe {
        let is_main: bool = msg_send![class!(NSThread), isMainThread];
        if !is_main {
            let arch = archive_path.to_string();
            let path = file_path.to_string();
            Queue::main().exec_async(move || {
                let _ = start_drag(&arch, &path);
            });
            return Ok(());
        }

        let pool = NSAutoreleasePool::new(nil);

        // Получаем active contentView
        let app: id = NSApp();
        if app == nil {
            pool.drain();
            return Err("NSApp is nil".into());
        }
        let mut window: id = msg_send![app, keyWindow];
        if window == nil {
            // Fallback: take first ordered window
            let ordered: id = msg_send![app, orderedWindows];
            if ordered != nil {
                let count: usize = msg_send![ordered, count];
                if count > 0 {
                    window = msg_send![ordered, objectAtIndex:0];
                }
            }
        }
        if window == nil {
            pool.drain();
            return Err("No key window".into());
        }
        let view: id = msg_send![window, contentView];
        if view == nil {
            pool.drain();
            return Err("No contentView".into());
        }

        // Делегат
        let delegate_cls = get_delegate_class();
        let delegate_inst: id = msg_send![delegate_cls, new];
        let ns_archive = NSString::alloc(nil).init_str(archive_path);
        let ns_path = NSString::alloc(nil).init_str(file_path);
        (*delegate_inst).set_ivar("path", ns_path);
        (*delegate_inst).set_ivar("archive", ns_archive);

        // NSFilePromiseProvider
        // Определяем UTI файла для лучшей совместимости Finder
        let ws: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let uti: id = msg_send![ws, typeOfFile:ns_path error:nil];
        let uti = if uti == nil {
            NSString::alloc(nil).init_str("public.data")
        } else { uti };
        let fp: id = msg_send![class!(NSFilePromiseProvider), alloc];
        let fp: id = msg_send![fp, initWithFileType:uti delegate:delegate_inst];
        // Retain provider и делегат, чтобы их не освободили после drain()
        let _: id = msg_send![fp, retain];
        let _: id = msg_send![delegate_inst, retain];

        // Current NSEvent
        let event: id = msg_send![app, currentEvent];

        // NSDraggingItem
        let item: id = msg_send![class!(NSDraggingItem), alloc];
        let item: id = msg_send![item, initWithPasteboardWriter:fp];
        let win_point: NSPoint = msg_send![event, locationInWindow];
        let view_point: NSPoint = msg_send![view, convertPoint:win_point fromView:nil];
        let frame = NSRect::new(view_point, NSSize::new(1.0, 1.0));
        // Добавляем иконку файла, чтобы macOS отображал превью и зелёный «плюс» при копировании
        let ws: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let icon: id = msg_send![ws, iconForFile:ns_path];
        let _: () = msg_send![icon, setSize:NSSize::new(64.0, 64.0)];
        let _: () = msg_send![item, setDraggingFrame:frame contents:icon];
        let items = NSArray::arrayWithObject(nil, item);
        println!("[dragout] beginDraggingSession call");
        let session: id = msg_send![view, beginDraggingSessionWithItems:items event:event source:delegate_inst];
        println!("[dragout] beginDraggingSession result {}", if session == nil { "nil" } else { "non-nil" });
        if session == nil {
            // Fallback: copy file URL to NSPasteboard so user can paste in Finder
            let pasteboard: id = NSPasteboard::generalPasteboard(nil);
            pasteboard.clearContents();
            let url: id = NSURL::fileURLWithPath_(nil, ns_path);
            let written: bool = msg_send![pasteboard, writeObjects: NSArray::arrayWithObject(nil, url)];
            pool.drain();
            if !written {
                return Err("beginDraggingSession failed and fallback pasteboard write failed".into());
            } else {
                println!("[dragout] beginDraggingSession failed, but URL copied to pasteboard as fallback");
                return Ok(());
            }
        }
        pool.drain();
    }
    Ok(())
}

/// Регистрация делегата `NSDraggingSource` + `NSFilePromiseProviderDelegate`.
fn get_delegate_class() -> &'static Class {
    static CELL: OnceCell<&'static Class> = OnceCell::new();
    CELL.get_or_init(|| unsafe {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("BAFilePromiseDelegate", superclass).unwrap();
        // Реализуем необходимые протоколы
        decl.add_protocol(objc::runtime::Protocol::get("NSFilePromiseProviderDelegate").unwrap());
        decl.add_protocol(objc::runtime::Protocol::get("NSDraggingSource").unwrap());

        // ivar для хранения NSString пути
        decl.add_ivar::<*mut Object>("path");
        decl.add_ivar::<*mut Object>("archive");

        // filePromiseProvider:writePromiseToURL:completionHandler:
        extern "C" fn write_promise(this: &Object, _sel: Sel, _provider: id, dest_url: id, _completion: id) {
    // Оборачиваем логику в catch_unwind, чтобы паника не пересекла границу FFI
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        unsafe {
            use std::ffi::CStr;
            use std::os::raw::c_char;
            #[allow(unused_imports)]
            use std::path::{Path, PathBuf};
            use objc::runtime::Object;

            let path_ptr: *mut Object = *this.get_ivar("path");
            let arch_ptr: *mut Object = *this.get_ivar("archive");
            if path_ptr.is_null() || arch_ptr.is_null() {
                println!("[dragout][err] ivars null in write_promise");
                return;
            }
            let src_ns: id = path_ptr as id;
            let arch_ns: id = arch_ptr as id;

            let c_src: *const c_char = msg_send![src_ns, UTF8String];
            let c_arch: *const c_char = msg_send![arch_ns, UTF8String];
            let dest_path_ns: id = msg_send![dest_url, path];
            let c_dest: *const c_char = msg_send![dest_path_ns, UTF8String];

            if c_src.is_null() || c_arch.is_null() || c_dest.is_null() {
                println!("[dragout][err] got null C string in write_promise");
                return;
            }

            let rel_path = CStr::from_ptr(c_src).to_string_lossy().into_owned();
            let arch_path = CStr::from_ptr(c_arch).to_string_lossy().into_owned();
            let dest_dir = CStr::from_ptr(c_dest).to_string_lossy().into_owned();

            let rel_path_pb = PathBuf::from(&rel_path);
            let comps = rel_path_pb.components().count();
            let strip = if comps > 1 { Some((comps - 1) as u32) } else { None };

            // dest_url содержит полный путь до места назначения с именем файла
            let dest_path = PathBuf::from(&dest_dir);
            let dest_root = dest_path.parent().map(Path::to_path_buf).unwrap_or_else(|| dest_path.clone());

            if let Err(e) = std::fs::create_dir_all(&dest_root) {
                println!("[dragout][err] create_dir_all failed: {:?}", e);
            }

            println!("[dragout] write_promise: rel_path='{}' arch='{}' dest='{}' strip={:?}", rel_path, arch_path, dest_root.display(), strip);

            

            println!("[dragout] extraction logic removed in published version");
            

            // Invoke completion handler block with nil to signal success
            if !_completion.is_null() {
                #[repr(C)]
                struct BlockLiteral {
                    isa: *const std::ffi::c_void,
                    flags: i32,
                    reserved: i32,
                    invoke: extern "C" fn(*mut BlockLiteral, *mut Object),
                }
                let block_ptr = _completion as *mut BlockLiteral;
                if !block_ptr.is_null() {
                    let invoke_fn = (*block_ptr).invoke;
                    // Call with nil error
                    invoke_fn(block_ptr, std::ptr::null_mut());
                }
            }
        }
    }));

    // Завершаем файл-промис без вызова Objective-C completion-handler —
    // пока достаточно для корректной работы Finder.

    if let Err(err) = result {
        println!("[dragout][panic] write_promise caught panic while extracting: {:?}", err);
    }
}

        // Регистрируем метод делегата
        decl.add_method(sel!(filePromiseProvider:writePromiseToURL:completionHandler:), write_promise as extern "C" fn(&Object, Sel, id, id, id));
        extern "C" fn source_mask(_this: &Object, _sel: Sel, _session: id, _context: u64) -> u64 {
            NS_DRAG_OPERATION_COPY
        }
        decl.add_method(sel!(draggingSession:sourceOperationMaskForDraggingContext:), source_mask as extern "C" fn(&Object, Sel, id, u64) -> u64);
        // Support older API Finder expects
        extern "C" fn source_mask_local(_this: &Object, _sel: Sel, _local: i8) -> u64 {
            NS_DRAG_OPERATION_COPY
        }
        decl.add_method(sel!(draggingSourceOperationMaskForLocal:), source_mask_local as extern "C" fn(&Object, Sel, i8) -> u64);

        // filePromiseProvider:promiseFilenameForDestination:
        extern "C" fn promise_filename(this: &Object, _sel: Sel, _provider: id, _dest: id) -> id {
            println!("[dragout] promise_filename called");
            unsafe {
                let path_ptr: *mut Object = *this.get_ivar("path");
                if path_ptr.is_null() {
                    return nil;
                }
                let src_ns: id = path_ptr as id;
                let name: id = msg_send![src_ns, lastPathComponent];
                msg_send![name, copy]
            }
        }
        decl.add_method(sel!(filePromiseProvider:promiseFilenameForDestination:), promise_filename as extern "C" fn(&Object, Sel, id, id) -> id);
        // filePromiseProvider:fileNameForType: (alternative path used by some targets)
        extern "C" fn file_name_for_type(this:&Object,_sel:Sel,_provider:id,_file_type:id)->id{
            unsafe{
                let path_ptr:*mut Object=*this.get_ivar("path");
                if path_ptr.is_null(){return nil;}
                let src_ns:id=path_ptr as id;
                let name:id= msg_send![src_ns, lastPathComponent];
                msg_send![name, copy]
            }
        }
        decl.add_method(sel!(filePromiseProvider:fileNameForType:), file_name_for_type as extern "C" fn(&Object, Sel, id, id)->id);

        // namesOfPromisedFilesDroppedAtDestination:
        extern "C" fn names_promised(this: &Object, _sel: Sel, dest_url: id) -> id {
            use std::ffi::CStr;
            use std::os::raw::c_char;
            #[allow(unused_imports)]
            use std::path::{Path, PathBuf};
            println!("[dragout] namesOfPromisedFilesDroppedAtDestination called");
            unsafe {
                let path_ptr: *mut Object = *this.get_ivar("path");
                let arch_ptr: *mut Object = *this.get_ivar("archive");
                if path_ptr.is_null() || arch_ptr.is_null() {
                    return nil;
                }
                let src_ns: id = path_ptr as id;
                let arch_ns: id = arch_ptr as id;
                let c_src: *const c_char = msg_send![src_ns, UTF8String];
                let c_arch: *const c_char = msg_send![arch_ns, UTF8String];
                let dest_path_ns: id = msg_send![dest_url, path];
                let c_dest: *const c_char = msg_send![dest_path_ns, UTF8String];
                if c_src.is_null() || c_arch.is_null() || c_dest.is_null() {
                    return nil;
                }
                let rel_path = CStr::from_ptr(c_src).to_string_lossy().into_owned();
                let _arch_path = CStr::from_ptr(c_arch).to_string_lossy().into_owned();
                let dest_dir = CStr::from_ptr(c_dest).to_string_lossy().into_owned();

                let _rel_path_pb = PathBuf::from(&rel_path);
                let dest_root = PathBuf::from(&dest_dir);
                if let Err(e) = std::fs::create_dir_all(&dest_root) {
                    println!("[dragout][err] create_dir_all failed: {:?}", e);
                }
                let filename_ns: id = msg_send![src_ns, lastPathComponent];
                let arr = NSArray::arrayWithObject(nil, filename_ns);
                msg_send![arr, retain]
            }
        }
        decl.add_method(sel!(namesOfPromisedFilesDroppedAtDestination:), names_promised as extern "C" fn(&Object, Sel, id) -> id);

        // draggingSession:endedAt:operation:
        extern "C" fn drag_ended(_this: &Object, _sel: Sel, _session: id, _point: NSPoint, _op: u64) {
            println!("[dragout] drag ended op={}", _op);
            // Не требуется дополнительных действий
        }
        decl.add_method(sel!(draggingSession:endedAt:operation:), drag_ended as extern "C" fn(&Object, Sel, id, NSPoint, u64));

        decl.register()
    })
}



